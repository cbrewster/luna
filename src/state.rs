use std::{collections::HashMap, sync::mpsc, thread};

use neon::prelude::*;

use crate::{table::LuaTableHandle, value::LuaValue};

type LuaCallback = Box<dyn FnOnce(&mut LuaContext, &Channel) + Send>;

#[derive(Debug, Clone)]
pub struct Lua {
    pub tx: mpsc::Sender<LuaMessage>,
}

impl Finalize for Lua {}

pub enum LuaMessage {
    Callback(LuaCallback),
    FinalizeTable(usize),
    Close,
}

pub struct LuaContext<'lua> {
    pub ctx: rlua::Context<'lua>,
    pub tables: HashMap<usize, rlua::Table<'lua>>,
    pub next_id: usize,
    pub lua: Lua,
}

impl<'lua> LuaContext<'lua> {
    pub fn table_handle(&mut self, table: rlua::Table<'lua>) -> LuaTableHandle {
        let handle = LuaTableHandle {
            id: self.next_id,
            lua: self.lua.clone(),
        };

        self.tables.insert(handle.id, table);
        self.next_id += 1;

        handle
    }

    pub fn get_table(&self, handle: &LuaTableHandle) -> Option<&rlua::Table<'lua>> {
        self.tables.get(&handle.id)
    }
}

impl Lua {
    pub fn new<'a, C>(cx: &mut C) -> Self
    where
        C: Context<'a>,
    {
        let (tx, rx) = mpsc::channel::<LuaMessage>();
        let state = rlua::Lua::new();

        let channel = cx.channel();

        let lua = Self { tx };
        // sigh...
        let lua_clone = lua.clone();
        thread::spawn(move || {
            state.context(|lua_ctx| {
                let mut ctx = LuaContext {
                    ctx: lua_ctx,
                    tables: HashMap::new(),
                    next_id: 0,
                    lua: lua_clone,
                };

                while let Ok(message) = rx.recv() {
                    match message {
                        LuaMessage::Callback(f) => {
                            f(&mut ctx, &channel);
                        }
                        LuaMessage::Close => break,
                        LuaMessage::FinalizeTable(id) => {
                            ctx.tables.remove(&id);
                        }
                    }
                }
            });
        });

        lua
    }

    pub fn close(&self) -> Result<(), mpsc::SendError<LuaMessage>> {
        self.tx.send(LuaMessage::Close)
    }

    pub fn send(
        &self,
        callback: impl FnOnce(&mut LuaContext, &Channel) + Send + 'static,
    ) -> Result<(), mpsc::SendError<LuaMessage>> {
        self.tx.send(LuaMessage::Callback(Box::new(callback)))
    }
}

impl Lua {
    pub fn js_new(mut cx: FunctionContext) -> JsResult<JsBox<Lua>> {
        let lua = Lua::new(&mut cx);
        Ok(cx.boxed(lua))
    }

    pub fn js_close(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        cx.this()
            .downcast_or_throw::<JsBox<Lua>, _>(&mut cx)?
            .close()
            .or_else(|err| cx.throw_error(err.to_string()))?;

        Ok(cx.undefined())
    }

    pub fn js_do_string(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let code = cx.argument::<JsString>(0)?.value(&mut cx);
        let callback = cx.argument::<JsFunction>(1)?.root(&mut cx);

        let lua = cx.this().downcast_or_throw::<JsBox<Lua>, _>(&mut cx)?;

        lua.send(move |lua_ctx, channel| {
            let result = lua_ctx
                .ctx
                .load(&code)
                .eval::<rlua::Value>()
                .map(|val| LuaValue::from_lua(lua_ctx, val));

            channel.send(move |mut cx| {
                let callback = callback.into_inner(&mut cx);
                let this = cx.undefined();
                let args: Vec<Handle<JsValue>> = match result {
                    Ok(val) => vec![cx.null().upcast(), val.to_js(&mut cx)?],
                    Err(err) => vec![cx.error(err.to_string())?.upcast()],
                };
                callback.call(&mut cx, this, args)?;

                Ok(())
            });
        })
        .or_else(|err| cx.throw_error(err.to_string()))?;

        Ok(cx.undefined())
    }
}
