use crate::vim::{BufferDeleteOptions, Vim};
use crate::Config;
use mlua::prelude::*;

pub fn select_down(lua: &Lua, _: ()) -> LuaResult<()> {
    let buffer = 0;
    let vim = Vim::new(lua);
    let total: i32 = vim.nvim_buf_get_var(buffer, "peek_results_count")?;
    let offset: i32 = vim.nvim_buf_get_var(buffer, "peek_offset")?;
    let limit: i32 = vim.nvim_buf_get_var(buffer, "peek_limit")?;
    let cursor_position: i32 = vim.nvim_buf_get_var(buffer, "peek_cursor")?;
    let next = cursor_position + 1;

    if (offset + cursor_position) == total {
        return Ok(());
    }

    if next == limit {
        vim.nvim_buf_set_var(buffer, "peek_offset", LuaValue::Integer((offset + 1).into()))?;
        crate::render(lua).call(())?;
        vim.nvim_buf_clear_namespace(buffer, 101, 0, -1)?;
        vim.nvim_buf_add_highlight(buffer, 101, "PeekSelection", cursor_position, 0, -1)?;
        return Ok(());
    }

    vim.nvim_buf_set_var(buffer, "peek_cursor", LuaValue::Integer(next.into()))?;
    vim.nvim_buf_clear_namespace(buffer, 101, 0, -1)?;
    vim.nvim_buf_add_highlight(buffer, 101, "PeekSelection", next, 0, -1)?;

    Ok(())
}

pub fn select_up(lua: &Lua, _: ()) -> LuaResult<()> {
    let buffer = 0;
    let vim = Vim::new(lua);
    let cursor_position = vim.nvim_buf_get_var::<i32>(buffer, "peek_cursor")?;
    let next = std::cmp::max(cursor_position - 1, 0);
    let offset: i32 = vim.nvim_buf_get_var(buffer, "peek_offset")?;

    if cursor_position == 2 && offset > 0 {
        vim.nvim_buf_set_var(buffer, "peek_offset", LuaValue::Integer((offset - 1).into()))?;
        crate::render(lua).call(())?;
        vim.nvim_buf_clear_namespace(buffer, 101, 0, -1)?;
        vim.nvim_buf_add_highlight(buffer, 101, "PeekSelection", cursor_position, 0, -1)?;
        return Ok(());
    }

    vim.nvim_buf_set_var(buffer, "peek_cursor", LuaValue::Integer(next.into()))?;
    vim.nvim_buf_clear_namespace(buffer, 101, 0, -1)?;
    vim.nvim_buf_add_highlight(buffer, 101, "PeekSelection", next, 0, -1)?;

    Ok(())
}

pub fn exit(lua: &Lua, _: ()) -> LuaResult<()> {
    let vim = Vim::new(lua);
    let buffer = vim.nvim_get_current_buf().unwrap();
    let window = vim.nvim_get_current_win().unwrap();
    let origin_window: usize = origin_window(lua, ())?;
    vim.nvim_win_close(window, true)?;
    lua.load("vim.cmd('stopinsert')").eval()?;
    vim.nvim_buf_delete(
        buffer,
        BufferDeleteOptions {
            force: Some(true),
            unload: None,
        },
    )?;
    vim.nvim_set_current_win(origin_window)
}

pub fn selected_value(lua: &Lua, _: ()) -> LuaResult<Option<mlua::Value>> {
    let buffer = 0;
    let vim = Vim::new(lua);
    let cursor_position: usize = vim.nvim_buf_get_var(buffer, "peek_cursor")?;
    let offset: usize = vim.nvim_buf_get_var(buffer, "peek_offset")?;
    let data: Vec<mlua::Value> = vim.nvim_buf_get_var(buffer, "peek_results")?;

    match data.get(offset + cursor_position - 1) {
        Some(v) => Ok(Some(v.clone())),
        None => Ok(None),
    }
}

pub fn backspace(lua: &Lua, _: ()) -> LuaResult<()> {
    let vim = Vim::new(lua);
    let config = Config::new(lua);
    let lines = vim.nvim_buf_get_lines(0, 0, 1, false)?;
    let mut prompt = lines.first().unwrap().clone();

    if config.title.unwrap() == prompt {
        vim.nvim_win_set_cursor(0, vec![1, 1000])?;
        return Ok(());
    }

    prompt.pop();
    vim.nvim_buf_set_lines(0, 0, 1, false, vec![prompt])?;
    vim.nvim_win_set_cursor(0, vec![1, 1000])?;

    Ok(())
}

pub fn origin_window(lua: &Lua, _: ()) -> LuaResult<usize> {
    let buffer = 0;
    let vim = Vim::new(lua);
    vim.nvim_buf_get_var(buffer, "peek_origin_window")
}
