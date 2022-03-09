use neon::prelude::*;

use crate::{state::LuaContext, table::LuaTableHandle};

/// An intermediate enum representing a lua value. This is created to appease the
/// thread-safety of both JS and Lua runtimes.
#[derive(Debug)]
pub enum LuaValue {
    Nil,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(String),
    Table(LuaTableHandle),
    Unsupported,
}

impl LuaValue {
    pub fn from_lua<'lua>(ctx: &mut LuaContext<'lua>, lua_val: rlua::Value<'lua>) -> Self {
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
    pub fn to_js<'js>(&self, cx: &mut impl Context<'js>) -> JsResult<'js, JsValue> {
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