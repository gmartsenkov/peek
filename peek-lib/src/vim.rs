use mlua::prelude::*;
use mlua::{Function, Table};
use serde::Serialize;

#[allow(dead_code)]
pub struct Vim<'a> {
    lua: &'a Lua,
    api: Table<'a>,
}

#[derive(Serialize)]
pub enum Mode {
    #[serde(rename = "n")]
    Normal,
    #[serde(rename = "i")]
    Insert
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct WindowOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split: Option<String>,
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

    pub fn nvim_buf_set_keymap(&self, buffer: i32, mode: Mode, lhs: String, callback: Function) -> LuaResult<()> {
        let func: Function = self
            .api
            .get("nvim_buf_set_keymap")
            .expect("can't load nvim_set_keymap");
        let opts = self.lua.create_table()?;
        opts.set("callback", callback)?;
        func.call((buffer, self.lua.to_value(&mode).unwrap(), lhs , "", opts))
    }
}
