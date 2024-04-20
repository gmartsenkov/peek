use mlua::prelude::*;
use mlua::{Function, Table};
use serde::Serialize;

#[allow(dead_code)]
pub struct Vim<'a> {
    lua: &'a Lua,
    api: Table<'a>,
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct WindowOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    split: Option<String>,
}

impl mlua::UserData for WindowOptions {}

impl<'a> Vim<'a> {
    pub fn new(lua: &'a Lua) -> Vim<'a> {
        let globals = lua.globals();
        let vim: Table = globals.get("vim").expect("can't load vim");
        let api: Table = vim.get("api").expect("can't load api");
        Vim { lua, api }
    }

    pub fn nvim_get_current_buf(&self) -> LuaResult<i32> {
        let func: Function = self
            .api
            .get("nvim_get_current_buf")
            .expect("can't load nvim_get_current_buf");
        func.call::<_, i32>(())
    }

    pub fn nvim_create_buffer(&self, listed: bool, scratch: bool) -> LuaResult<i32> {
        let func: Function = self
            .api
            .get("nvim_create_buf")
            .expect("can't load nvim_create_buf");
        func.call::<_, i32>((listed, scratch))
    }

    pub fn nvim_open_win(
        &self,
        buffer: i32,
        enter_window: bool,
        opts: WindowOptions,
    ) -> LuaResult<i32> {
        let func: Function = self
            .api
            .get("nvim_open_win")
            .expect("can't load nvim_open_win");

        func.call::<_, i32>((buffer, enter_window, self.lua.to_value(&opts).unwrap()))
    }
}

pub fn nvim_get_current_buf(lua: &Lua, _: ()) -> LuaResult<()> {
    let vim = Vim::new(lua);
    let buf = vim.nvim_get_current_buf()?;
    Ok(())
}

pub fn create_window(lua: &Lua, _: ()) -> LuaResult<()> {
    let vim = Vim::new(lua);
    let buf = vim.nvim_create_buffer(false, true)?;
    let win = vim.nvim_open_win(
        buf,
        true,
        WindowOptions {
            width: None,
            height: Some(15),
            split: Some("below".into()),
        },
    )?;

    Ok(())
}
