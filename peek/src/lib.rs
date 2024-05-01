use mlua::prelude::*;

#[mlua::lua_module]
fn peek(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("nvim_get_current_buf", lua.create_function(peek_lib::nvim_get_current_buf)?)?;

    let builtins = lua.create_table()?;
    builtins.set("find_file", lua.create_function(peek_lib::file_picker)?)?;
    builtins.set("find_buffer", lua.create_function(peek_lib::buffer_picker)?)?;

    exports.set("builtins", builtins)?;
    Ok(exports)
}
