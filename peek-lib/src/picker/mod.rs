pub mod buffer;
pub mod file;
pub mod file_explorer;

use crate::create_window;
use crate::picker;
use mlua::prelude::*;

pub fn file_picker(lua: &Lua, config: mlua::Table) -> LuaResult<()> {
    let mappings = lua.create_table()?;
    let insert = lua.create_table()?;
    insert.set("<CR>", lua.create_function(picker::file::open_file)?)?;
    mappings.set("i", insert)?;

    config.set("filter", lua.create_function(picker::file::filter)?)?;
    config.set("to_line", lua.create_function(picker::file::to_line)?)?;
    config.set("mappings", mappings)?;
    config.set("title", "Find File")?;
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
    config.set("title", "Find Buffer")?;
    create_window(lua, config)
}

pub fn file_explorer_picker(lua: &Lua, config: mlua::Table) -> LuaResult<()> {
    let mappings = lua.create_table()?;
    let insert = lua.create_table()?;
    insert.set("<CR>", lua.create_function(picker::file_explorer::select_option)?)?;
    insert.set("<BS>", lua.create_function(picker::file_explorer::backspace)?)?;
    insert.set("<Tab>", lua.create_function(picker::file_explorer::tab)?)?;
    mappings.set("i", insert)?;
    config.set("filter", lua.create_function(picker::file_explorer::filter)?)?;
    config.set("to_line", lua.create_function(picker::file_explorer::to_line)?)?;
    config.set("mappings", mappings)?;
    config.set("title", "Find File")?;
    create_window(lua, config)
}
