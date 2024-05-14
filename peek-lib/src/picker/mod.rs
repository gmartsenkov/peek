pub mod buffer;
pub mod file;

use crate::create_window;
use crate::picker;
use mlua::prelude::*;

pub fn file_picker(lua: &Lua, _: ()) -> LuaResult<()> {
    let config = lua.create_table()?;
    let mappings = lua.create_table()?;
    let insert = lua.create_table()?;
    insert.set("<CR>", lua.create_function(picker::file::open_file)?)?;
    mappings.set("i", insert)?;

    config.set("filter", lua.create_function(picker::file::filter)?)?;
    config.set("to_line", lua.create_function(picker::file::to_line)?)?;
    config.set("mappings", mappings)?;
    create_window(lua, config)
}

pub fn buffer_picker(lua: &Lua, config: mlua::Table) -> LuaResult<()> {
    let mappings = lua.create_table()?;
    let insert = lua.create_table()?;
    insert.set("<CR>", lua.create_function(picker::buffer::open_buffer)?)?;
    mappings.set("i", insert)?;

    config.set("filter", lua.create_function(picker::buffer::filter)?)?;
    config.set("to_line", lua.create_function(picker::buffer::to_line)?)?;
    config.set("mappings", mappings)?;
    create_window(lua, config)
}
