use std::sync::mpsc;

use neon::prelude::*;

use crate::{state::{Lua, LuaMessage}, value::LuaValue};

#[derive(Debug, Clone)]
pub struct LuaTableHandle {
    pub id: usize,
    pub lua: Lua,
}

impl Finalize for LuaTableHandle {
    fn finalize<'a, C: Context<'a>>(self, _: &mut C) {
        let _ = self.lua.tx.send(LuaMessage::FinalizeTable(self.id));
    }
}

impl LuaTableHandle {
    pub fn js_get(mut cx: FunctionContext) -> JsResult<JsValue> {
        let key = cx.argument::<JsValue>(0).and_then(|key| LuaValue::from_js(&mut cx, key))?;

        let handle: LuaTableHandle = (**cx
            .this()
            .downcast_or_throw::<JsBox<LuaTableHandle>, _>(&mut cx)?)
        .clone();
        let lua = handle.lua.clone();

        let value = lua.request::<rlua::Result<LuaValue>, _>(move |lua_ctx, _| {
            let lua_key = key.to_lua(lua_ctx)?;
            let table = lua_ctx.get_table(&handle).expect("stale table handle");
            let lua_val = table.get(lua_key)?;
            Ok(LuaValue::from_lua(lua_ctx, lua_val))
        })
        .or_else(|err| cx.throw_error(err.to_string()))?
        .or_else(|err| cx.throw_error(err.to_string()))?;

        value.to_js(&mut cx)
    }
    
    pub fn js_set(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let key = cx.argument::<JsValue>(0).and_then(|key| LuaValue::from_js(&mut cx, key))?;
        let val = cx.argument::<JsValue>(1).and_then(|val| LuaValue::from_js(&mut cx, val))?;

        let handle: LuaTableHandle = (**cx
            .this()
            .downcast_or_throw::<JsBox<LuaTableHandle>, _>(&mut cx)?)
        .clone();
        let lua = handle.lua.clone();

        lua.request::<rlua::Result<_>, _>(move |lua_ctx, _| {
            let lua_key = key.to_lua(lua_ctx)?;
            let lua_val = val.to_lua(lua_ctx)?;
            let table = lua_ctx.get_table(&handle).expect("stale table handle");
            table.set(lua_key, lua_val)?;
            Ok(())
        })
        .or_else(|err| cx.throw_error(err.to_string()))?
        .or_else(|err| cx.throw_error(err.to_string()))?;

        Ok(cx.undefined())
    }
    
    pub fn js_for_each(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);

        let handle: LuaTableHandle = (**cx
            .this()
            .downcast_or_throw::<JsBox<LuaTableHandle>, _>(&mut cx)?)
        .clone();
        let lua = handle.lua.clone();

        // TODO: Make this actually async instead of blocking main thread.
        let pairs = lua.request(move |lua_ctx, _| {
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

            pairs
        })
        .or_else(|err| cx.throw_error(err.to_string()))?;

        let callback = callback.into_inner(&mut cx);
        let this = cx.undefined();

        for (key, val) in pairs {
            let args = vec![key.to_js(&mut cx)?, val.to_js(&mut cx)?];
            callback.call(&mut cx, this, args)?;
        }

        Ok(cx.undefined())
    }

    pub fn js_to_string(mut cx: FunctionContext) -> JsResult<JsString> {
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
