use mlua::prelude::*;
use mlua::{Function, Table};
use serde::Serialize;

#[allow(dead_code)]
pub struct Vim<'a> {
    lua: &'a Lua,
    api: Table<'a>,
}

pub struct BufferAttachOptions<'a> {
    pub on_lines: Option<Function<'a>>,
}

#[derive(Serialize)]
pub enum Mode {
    #[serde(rename = "n")]
    Normal,
    #[serde(rename = "i")]
    Insert,
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

impl<'a> BufferAttachOptions<'a> {
    pub fn to_lua_table(&'a self, lua: &'a Lua) -> Table {
        let table = lua.create_table().unwrap();
        if let Some(func) = &self.on_lines {
            table.set("on_lines", func).unwrap();
        }
        table
    }
}

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

    pub fn nvim_buf_attach(
        &self,
        buffer: i32,
        send_buffer: bool,
        opts: BufferAttachOptions,
    ) -> LuaResult<()> {
        let func: Function = self
            .api
            .get("nvim_buf_attach")
            .expect("can't load nvim_buf_attach");

        func.call((buffer, send_buffer, opts.to_lua_table(self.lua)))
    }

    pub fn nvim_buf_get_lines(
        &self,
        buffer: i32,
        start: i32,
        end: i32,
        strict_indexing: bool,
    ) -> LuaResult<Vec<String>> {
        let func: Function = self
            .api
            .get("nvim_buf_get_lines")
            .expect("can't load nvim_buf_get_lines");

        func.call((buffer, start, end, strict_indexing))
    }

    pub fn nvim_win_close(&self, window: i32, force: bool) -> LuaResult<()> {
        let func: Function = self.api.get("nvim_win_close").expect("nvim_win_close");
        func.call((window, force))
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

    pub fn nvim_buf_set_keymap(
        &self,
        buffer: i32,
        mode: Mode,
        lhs: String,
        callback: Function,
    ) -> LuaResult<()> {
        let func: Function = self
            .api
            .get("nvim_buf_set_keymap")
            .expect("can't load nvim_set_keymap");
        let opts = self.lua.create_table()?;
        opts.set("callback", callback)?;
        func.call((buffer, self.lua.to_value(&mode).unwrap(), lhs, "", opts))
    }
}
