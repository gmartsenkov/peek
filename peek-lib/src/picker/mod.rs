pub mod buffer;
pub mod file;

use crate::create_window;
use crate::picker;
use mlua::prelude::*;

pub fn file_picker(lua: &Lua, _: ()) -> LuaResult<()> {
    let config = lua.create_table()?;
    config.set("initial_data", picker::file::initial_data(lua))?;
    config.set("filter", picker::file::filter(lua))?;
    config.set("to_line", picker::file::to_line(lua))?;
    config.set("on_open", picker::file::on_open(lua))?;
    create_window(lua, config)
}

pub fn buffer_picker(lua: &Lua, config: mlua::Table) -> LuaResult<()> {
    config.set("initial_data", picker::buffer::initial_data(lua))?;
    config.set("filter", picker::buffer::filter(lua))?;
    config.set("to_line", picker::buffer::to_line(lua))?;
    config.set("on_open", picker::buffer::on_open(lua))?;
    create_window(lua, config)
}
