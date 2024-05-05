use crate::vim::{BufferDeleteOptions, Vim};
use mlua::prelude::*;
use mlua::Function;

pub fn select_down(lua: &Lua) -> Function {
    lua.create_function(move |lua, ()| {
        let buffer = 0;
        let vim = Vim::new(lua);
        let total: i32 = vim.nvim_buf_get_var(buffer, "peek_results_count".into())?;
        let offset: i32 = vim.nvim_buf_get_var(buffer, "peek_offset".into())?;
        let limit: i32 = vim.nvim_buf_get_var(buffer, "peek_limit".into())?;
        let cursor_position: i32 = vim.nvim_buf_get_var(buffer, "peek_cursor".into())?;
        let next = cursor_position + 1;

        if (offset + cursor_position) == total {
            return Ok(());
        }

        if next == limit {
            vim.nvim_buf_set_var(buffer, "peek_offset".into(), LuaValue::Integer((offset + 1).into()))?;
            crate::render(lua).call(())?;
            vim.nvim_buf_clear_namespace(buffer, 101, 0, -1)?;
            vim.nvim_buf_add_highlight(buffer, 101, "PeekSelection".into(), cursor_position, 0, -1)?;
            return Ok(());
        }

        vim.nvim_buf_set_var(buffer, "peek_cursor".into(), LuaValue::Integer(next.into()))?;
        vim.nvim_buf_clear_namespace(buffer, 101, 0, -1)?;
        vim.nvim_buf_add_highlight(buffer, 101, "PeekSelection".into(), next, 0, -1)?;

        Ok(())
    })
    .unwrap()
}

pub fn select_up(lua: &Lua) -> Function {
    lua.create_function(move |lua, ()| {
        let buffer = 0;
        let vim = Vim::new(lua);
        let cursor_position = vim.nvim_buf_get_var::<i32>(buffer, "peek_cursor".into())?;
        let next = std::cmp::max(cursor_position - 1, 0);
        let offset: i32 = vim.nvim_buf_get_var(buffer, "peek_offset".into())?;

        if cursor_position == 2 && offset > 0 {
            vim.nvim_buf_set_var(buffer, "peek_offset".into(), LuaValue::Integer((offset - 1).into()))?;
            crate::render(lua).call(())?;
            vim.nvim_buf_clear_namespace(buffer, 101, 0, -1)?;
            vim.nvim_buf_add_highlight(buffer, 101, "PeekSelection".into(), cursor_position, 0, -1)?;
            return Ok(());
        }

        vim.nvim_buf_set_var(buffer, "peek_cursor".into(), LuaValue::Integer(next.into()))?;
        vim.nvim_buf_clear_namespace(buffer, 101, 0, -1)?;
        vim.nvim_buf_add_highlight(buffer, 101, "PeekSelection".into(), next, 0, -1)?;

        Ok(())
    })
    .unwrap()
}

pub fn exit(lua: &Lua) -> Function {
    lua.create_function(move |lua, ()| {
        let vim = Vim::new(lua);
        let buffer = vim.nvim_get_current_buf().unwrap();
        let window = vim.nvim_get_current_win().unwrap();
        vim.nvim_win_close(window, true)?;
        lua.load("vim.cmd('stopinsert')").eval()?;
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

pub fn selected_value(lua: &Lua) -> Function {
    lua.create_function(move |lua, ()| {
        let buffer = 0;
        let vim = Vim::new(lua);
        let cursor_position: usize = vim.nvim_buf_get_var(buffer, "peek_cursor".into())?;
        let offset: usize = vim.nvim_buf_get_var(buffer, "peek_offset".into())?;
        let data: Vec<mlua::Value> = vim.nvim_buf_get_var(buffer, "peek_results".into())?;

        match data.get(offset + cursor_position - 1) {
            Some(v) => Ok(Some(v.clone())),
            None => Ok(None),
        }
    })
    .unwrap()
}

pub fn origin_window(lua: &Lua) -> Function {
    lua.create_function(move |lua, ()| {
        let buffer = 0;
        let vim = Vim::new(lua);
        vim.nvim_buf_get_var::<usize>(buffer, "peek_origin_window".into())
    })
    .unwrap()
}
