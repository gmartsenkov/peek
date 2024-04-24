use crate::vim::{BufferDeleteOptions, Vim};
use mlua::prelude::*;
use mlua::Function;

pub fn select_down(lua: &Lua, buffer: i32) -> Function {
    lua.create_function(move |lua, ()| {
        let vim = Vim::new(lua);
        let cursor_position = vim.nvim_buf_get_var::<i32>(buffer, "peek_cursor".into())?;
        let next = cursor_position + 1;
        vim.nvim_buf_set_var(buffer, "peek_cursor".into(), LuaValue::Integer(next.into()))?;
        vim.nvim_buf_clear_namespace(buffer, 101, 0, -1)?;
        vim.nvim_buf_add_highlight(buffer, 101, "PeekSelection".into(), next, 0, -1)?;

        Ok(())
    })
    .unwrap()
}

pub fn select_up(lua: &Lua, buffer: i32) -> Function {
    lua.create_function(move |lua, ()| {
        let vim = Vim::new(lua);
        let cursor_position = vim.nvim_buf_get_var::<i32>(buffer, "peek_cursor".into())?;
        let next = std::cmp::max(cursor_position - 1, 0);
        vim.nvim_buf_set_var(buffer, "peek_cursor".into(), LuaValue::Integer(next.into()))?;
        vim.nvim_buf_clear_namespace(buffer, 101, 0, -1)?;
        vim.nvim_buf_add_highlight(buffer, 101, "PeekSelection".into(), next, 0, -1)?;

        Ok(())
    })
    .unwrap()
}

pub fn exit(lua: &Lua, window: i32, buffer: i32) -> Function {
    lua.create_function(move |lua, ()| {
        let vim = Vim::new(lua);
        vim.nvim_win_close(window, true)?;
        vim.nvim_buf_delete(
            buffer,
            BufferDeleteOptions {
                force: Some(true),
                unload: None,
            },
        )
    })
    .unwrap()
}
