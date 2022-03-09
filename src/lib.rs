mod state;
mod table;
mod value;

use neon::prelude::*;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("luaNew", state::Lua::js_new)?;
    cx.export_function("luaClose", state::Lua::js_close)?;
    cx.export_function("luaDoString", state::Lua::js_do_string)?;
    cx.export_function("luaTableForEach", table::LuaTableHandle::js_for_each)?;
    cx.export_function("luaTableToString", table::LuaTableHandle::js_to_string)?;
    Ok(())
}
