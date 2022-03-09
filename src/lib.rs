mod state;
mod table;
mod value;

use neon::prelude::*;

use crate::{state::Lua, table::LuaTableHandle};

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("luaNew", Lua::js_new)?;
    cx.export_function("luaClose", Lua::js_close)?;
    cx.export_function("luaDoString", Lua::js_do_string)?;
    cx.export_function("luaNewTable", Lua::js_new_table)?;
    
    cx.export_function("luaTableGet", LuaTableHandle::js_get)?;
    cx.export_function("luaTableSet", LuaTableHandle::js_set)?;
    cx.export_function("luaTableForEach", LuaTableHandle::js_for_each)?;
    cx.export_function("luaTableToString", LuaTableHandle::js_to_string)?;
    
    Ok(())
}
