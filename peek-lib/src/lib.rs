pub mod vim;

use mlua::prelude::*;
use vim::{Vim, WindowOptions};

pub fn nvim_get_current_buf(lua: &Lua, _: ()) -> LuaResult<i32> {
    let vim = Vim::new(lua);
    vim.nvim_get_current_buf()
}

pub fn create_window(lua: &Lua, _: ()) -> LuaResult<()> {
    let vim = Vim::new(lua);
    let buffer = vim.nvim_create_buffer(false, true)?;
    let win = vim.nvim_open_win(
        buffer,
        true,
        WindowOptions {
            width: None,
            height: Some(15),
            split: Some("below".into()),
        },
    )?;

    let rhs = lua.create_function(move |lua, ()| {
        let vim = Vim::new(lua);
        vim.nvim_win_close(win.clone(), true)
    })?;

    lua.load("vim.cmd('file Peek')").eval()?;
    vim.nvim_buf_set_keymap(buffer, vim::Mode::Normal, "<ESC>".into(), rhs)
}
