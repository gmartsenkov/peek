use mlua::prelude::*;
use mlua::{Function, Table};

#[allow(dead_code)]
struct Vim<'a> {
    lua: &'a Lua,
    api: Table<'a>,
}

impl<'a> Vim<'a> {
    pub fn new(lua: &'a Lua) -> Vim<'a> {
        let globals = lua.globals();
        let vim: Table = globals.get("vim").expect("can't load vim");
        let api: Table = vim.get("api").expect("can't load api");
        Vim { lua, api }
    }

    pub fn nvim_get_current_buf(&self) -> LuaResult<i32> {
        let get_buf: Function = self
            .api
            .get("nvim_get_current_buf")
            .expect("can't load nvim_get_current_buf");
        get_buf.call::<_, i32>(())
    }
}

fn nvim_get_current_buf(lua: &Lua, _: ()) -> LuaResult<()> {
    let vim = Vim::new(lua);
    let buf = vim.nvim_get_current_buf()?;
    println!("{}", buf);
    Ok(())
}

#[mlua::lua_module]
fn peek(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set(
        "nvim_get_current_buf",
        lua.create_function(nvim_get_current_buf)?,
    )?;
    Ok(exports)
}
