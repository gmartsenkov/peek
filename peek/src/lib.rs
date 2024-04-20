use mlua::prelude::*;

#[mlua::lua_module]
fn peek(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set(
        "nvim_get_current_buf",
        lua.create_function(peek_lib::nvim_get_current_buf)?,
    )?;
    exports.set(
        "create_window",
        lua.create_function(peek_lib::create_window)?,
    )?;
    Ok(exports)
}
