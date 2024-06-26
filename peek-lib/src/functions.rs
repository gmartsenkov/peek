use crate::vim::{BufferDeleteOptions, Vim};
use crate::Config;
use mlua::prelude::*;

pub fn select_down(lua: &Lua, _: ()) -> LuaResult<()> {
    let buffer = 0;
    let vim = Vim::new(lua);
    let config = Config::new(lua);
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
        crate::highlight_selected_line(&vim, buffer, cursor_position)?;
        config.on_refresh_callback()?;
        return Ok(());
    }

    vim.nvim_buf_set_var(buffer, "peek_cursor", LuaValue::Integer(next.into()))?;
    crate::highlight_selected_line(&vim, buffer, next)?;
    config.on_refresh_callback()?;

    Ok(())
}

pub fn select_up(lua: &Lua, _: ()) -> LuaResult<()> {
    let buffer = 0;
    let vim = Vim::new(lua);
    let config = Config::new(lua);
    let cursor_position = vim.nvim_buf_get_var::<i32>(buffer, "peek_cursor")?;
    let next = std::cmp::max(cursor_position - 1, 0);
    let offset: i32 = vim.nvim_buf_get_var(buffer, "peek_offset")?;

    if cursor_position == 2 && offset > 0 {
        vim.nvim_buf_set_var(buffer, "peek_offset", LuaValue::Integer((offset - 1).into()))?;
        crate::render(lua).call(())?;

        crate::highlight_selected_line(&vim, buffer, cursor_position)?;
        config.on_refresh_callback()?;
        return Ok(());
    }

    vim.nvim_buf_set_var(buffer, "peek_cursor", LuaValue::Integer(next.into()))?;

    crate::highlight_selected_line(&vim, buffer, next)?;
    config.on_refresh_callback()?;

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

pub fn result_count(lua: &Lua, _: ()) -> LuaResult<usize> {
    let vim = Vim::new(lua);
    vim.nvim_buf_get_var(0, "peek_results_count")
}

pub fn position(lua: &Lua, _: ()) -> LuaResult<usize> {
    let vim = Vim::new(lua);
    let cursor: usize = vim.nvim_buf_get_var(0, "peek_cursor")?;
    let offset: usize = vim.nvim_buf_get_var(0, "peek_offset")?;

    Ok(cursor + offset)
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

pub fn origin_window(lua: &Lua, _: ()) -> LuaResult<usize> {
    let buffer = 0;
    let vim = Vim::new(lua);
    vim.nvim_buf_get_var(buffer, "peek_origin_window")
}
