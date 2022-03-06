use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

use neon::prelude::*;

type LuaCallback = Box<dyn FnOnce(&mut LuaContext, &Channel) + Send>;

#[derive(Debug, Clone)]
struct Lua {
    tx: mpsc::Sender<LuaMessage>,
}

impl Finalize for Lua {}

enum LuaMessage {
    Callback(LuaCallback),
    FinalizeTable(usize),
    Close,
}

struct LuaContext<'lua> {
    ctx: rlua::Context<'lua>,
    tables: HashMap<usize, rlua::Table<'lua>>,
    next_id: usize,
    lua: Lua,
}

impl<'lua> LuaContext<'lua> {
    fn table_handle(&mut self, table: rlua::Table<'lua>) -> LuaTableHandle {
        let handle = LuaTableHandle {
            id: self.next_id,
            lua: self.lua.clone(),
        };

        self.tables.insert(handle.id, table);
        self.next_id += 1;

        handle
    }

    fn get_table(&self, handle: &LuaTableHandle) -> Option<&rlua::Table<'lua>> {
        self.tables.get(&handle.id)
    }
}

#[derive(Debug, Clone)]
struct LuaTableHandle {
    id: usize,
    lua: Lua,
}

impl Finalize for LuaTableHandle {
    fn finalize<'a, C: Context<'a>>(self, _: &mut C) {
        let _ = self.lua.tx.send(LuaMessage::FinalizeTable(self.id));
    }
}

impl Lua {
    fn new<'a, C>(cx: &mut C) -> Self
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

    fn close(&self) -> Result<(), mpsc::SendError<LuaMessage>> {
        self.tx.send(LuaMessage::Close)
    }

    fn send(
        &self,
        callback: impl FnOnce(&mut LuaContext, &Channel) + Send + 'static,
    ) -> Result<(), mpsc::SendError<LuaMessage>> {
        self.tx.send(LuaMessage::Callback(Box::new(callback)))
    }
}

impl Lua {
    fn js_new(mut cx: FunctionContext) -> JsResult<JsBox<Lua>> {
        let lua = Lua::new(&mut cx);
        Ok(cx.boxed(lua))
    }

    fn js_close(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        cx.this()
            .downcast_or_throw::<JsBox<Lua>, _>(&mut cx)?
            .close()
            .or_else(|err| cx.throw_error(err.to_string()))?;

        Ok(cx.undefined())
    }

    fn js_do_string(mut cx: FunctionContext) -> JsResult<JsUndefined> {
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

impl LuaTableHandle {
    fn js_for_each(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);

        let handle: LuaTableHandle = (**cx
            .this()
            .downcast_or_throw::<JsBox<LuaTableHandle>, _>(&mut cx)?)
        .clone();
        let lua = handle.lua.clone();

        let (tx, rx) = mpsc::channel();

        // TODO: Make this actually async instead of blocking main thread.
        lua.send(move |lua_ctx, _| {
            let table = lua_ctx.get_table(&handle).expect("Unknown table");
            let pairs = table
                .clone()
                .pairs::<rlua::Value, rlua::Value>()
                .map(|pair| pair.unwrap())
                .map(|(key, val)| {
                    (
                        LuaValue::from_lua(lua_ctx, key),
                        LuaValue::from_lua(lua_ctx, val),
                    )
                })
                .collect::<Vec<_>>();

            tx.send(pairs).expect("failed to send");
        })
        .or_else(|err| cx.throw_error(err.to_string()))?;

        let pairs = rx.recv().or_else(|err| cx.throw_error(err.to_string()))?;
        let callback = callback.into_inner(&mut cx);
        let this = cx.undefined();

        for (key, val) in pairs {
            let args = vec![key.to_js(&mut cx)?, val.to_js(&mut cx)?];
            callback.call(&mut cx, this, args)?;
        }

        Ok(cx.undefined())
    }

    fn js_to_string(mut cx: FunctionContext) -> JsResult<JsString> {
        let handle: LuaTableHandle = (**cx
            .this()
            .downcast_or_throw::<JsBox<LuaTableHandle>, _>(&mut cx)?)
        .clone();
        let lua = handle.lua.clone();

        let (tx, rx) = mpsc::channel();

        // TODO: Make this actually async instead of blocking main thread.
        lua.send(move |lua_ctx, _| {
            let table = lua_ctx.get_table(&handle).expect("Unknown table");

            let to_string: rlua::Function = lua_ctx.ctx.globals().get("tostring").expect("failed to get tostring");
            let table_ref =  to_string
                    .call::<_, String>(rlua::Value::Table(table.clone()))
                    .expect("tostring failed")
                    .to_owned();

            tx.send(table_ref).expect("failed to send");
        })
        .or_else(|err| cx.throw_error(err.to_string()))?;

        let table_ref = rx.recv().or_else(|err| cx.throw_error(err.to_string()))?;

        Ok(cx.string(table_ref))
    }
}

/// An intermediate enum representing a lua value. This is created to appease the
/// thread-safety of both JS and Lua runtimes.
#[derive(Debug)]
enum LuaValue {
    Nil,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(String),
    Table(LuaTableHandle),
    Unsupported,
}

impl LuaValue {
    fn from_lua<'lua>(ctx: &mut LuaContext<'lua>, lua_val: rlua::Value<'lua>) -> Self {
        match lua_val {
            rlua::Value::Nil => LuaValue::Nil,
            rlua::Value::Boolean(val) => LuaValue::Boolean(val),
            rlua::Value::Integer(val) => LuaValue::Integer(val),
            rlua::Value::Number(val) => LuaValue::Number(val),
            rlua::Value::String(val) => LuaValue::String(val.to_str().unwrap().to_owned()),
            rlua::Value::Table(table) => LuaValue::Table(ctx.table_handle(table)),

            // TODO: The rest of em!
            _ => LuaValue::Unsupported,
        }
    }
}

impl LuaValue {
    fn to_js<'js>(&self, cx: &mut impl Context<'js>) -> JsResult<'js, JsValue> {
        Ok(match self {
            LuaValue::Nil => cx.null().upcast(),
            LuaValue::Boolean(val) => cx.boolean(*val).upcast(),
            LuaValue::Integer(val) => cx.number(*val as f64).upcast(),
            LuaValue::Number(val) => cx.number(*val).upcast(),
            LuaValue::String(val) => cx.string(val.clone()).upcast(),
            LuaValue::Table(table) => {
                let obj = cx.boxed(table.clone());
                let typ = cx.string("table");
                obj.set(cx, "__type", typ)?;
                obj.upcast()
            },
            LuaValue::Unsupported => cx.throw_error("unsupported type")?,
        })
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("luaNew", Lua::js_new)?;
    cx.export_function("luaClose", Lua::js_close)?;
    cx.export_function("luaDoString", Lua::js_do_string)?;
    cx.export_function("luaTableForEach", LuaTableHandle::js_for_each)?;
    cx.export_function("luaTableToString", LuaTableHandle::js_to_string)?;
    Ok(())
}
