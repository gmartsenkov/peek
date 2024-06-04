use mlua::prelude::*;

#[mlua::lua_module]
fn peek(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;

    let builtins = lua.create_table()?;
    builtins.set("find_file", lua.create_function(peek_lib::picker::file_picker)?)?;
    builtins.set("find_buffer", lua.create_function(peek_lib::picker::buffer_picker)?)?;
    builtins.set("file_explorer", lua.create_function(peek_lib::picker::file_explorer_picker)?)?;

    let functions = lua.create_table()?;
    functions.set("result_count", lua.create_function(peek_lib::functions::result_count)?)?;
    functions.set("position", lua.create_function(peek_lib::functions::position)?)?;

    exports.set("builtins", builtins)?;
    exports.set("fn", functions)?;
    Ok(exports)
}
