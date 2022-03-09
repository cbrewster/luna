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

    pub fn from_js<'js>(cx: &mut impl Context<'js>, js_val: Handle<JsValue>) -> NeonResult<Self> {
        if let Ok(_) = js_val.downcast::<JsNull, _>(cx) {
            return Ok(LuaValue::Nil);
        }

        if let Ok(val) = js_val.downcast::<JsBoolean, _>(cx) {
            return Ok(LuaValue::Boolean(val.value(cx)));
        }
        
        if let Ok(val) = js_val.downcast::<JsNumber, _>(cx) {
            return Ok(LuaValue::Number(val.value(cx)));
        }
        
        if let Ok(val) = js_val.downcast::<JsString, _>(cx) {
            return Ok(LuaValue::String(val.value(cx)));
        }
        
        if let Ok(val) = js_val.downcast::<JsString, _>(cx) {
            return Ok(LuaValue::String(val.value(cx)));
        }

        if let Ok(val) = js_val.downcast::<JsBox<LuaTableHandle>, _>(cx) {
            return Ok(LuaValue::Table((**val).clone()));
        }

        cx.throw_error("Unsupported type")
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
            }
            LuaValue::Unsupported => cx.throw_error("unsupported type")?,
        })
    }

    pub fn to_lua<'lua>(&self, lua_ctx: &mut LuaContext<'lua>) -> rlua::Result<rlua::Value<'lua>> {
        Ok(match self {
            LuaValue::Nil => rlua::Value::Nil,
            LuaValue::Boolean(val) => rlua::Value::Boolean(*val),
            LuaValue::Integer(val) => rlua::Value::Integer(*val),
            LuaValue::Number(val) => rlua::Value::Number(*val),
            LuaValue::String(val) => rlua::Value::String(lua_ctx.ctx.create_string(val)?),
            LuaValue::Table(table) => {
                rlua::Value::Table(lua_ctx.get_table(&table).expect("stale handle").clone())
            }
            LuaValue::Unsupported => todo!(),
        })
    }
}
