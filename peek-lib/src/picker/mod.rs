pub mod buffer;
pub mod file;

use crate::create_window;
use crate::picker;
use mlua::prelude::*;

pub fn file_picker(lua: &Lua, _: ()) -> LuaResult<()> {
    let config = lua.create_table()?;
    config.set("filter", lua.create_function(picker::file::filter)?)?;
    config.set("to_line", lua.create_function(picker::file::to_line)?)?;
    config.set("on_open", lua.create_function(picker::file::on_open)?)?;
    create_window(lua, config)
}

pub fn buffer_picker(lua: &Lua, config: mlua::Table) -> LuaResult<()> {
    config.set("filter", lua.create_function(picker::buffer::filter)?)?;
    config.set("to_line", lua.create_function(picker::buffer::to_line)?)?;
    config.set("on_open", lua.create_function(picker::buffer::on_open)?)?;
    create_window(lua, config)
}
