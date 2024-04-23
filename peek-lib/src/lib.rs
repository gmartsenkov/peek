pub mod vim;

use log::LevelFilter;
use mlua::prelude::*;
use vim::{Vim, WindowOptions};

pub fn nvim_get_current_buf(lua: &Lua, _: ()) -> LuaResult<i32> {
    let vim = Vim::new(lua);
    vim.nvim_get_current_buf()
}

pub fn create_window(lua: &Lua, _: ()) -> LuaResult<()> {
    simple_logging::log_to_file("test.log", LevelFilter::Info).unwrap();
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
        vim.nvim_win_close(win, true)
    })?;

    let select_down = lua.create_function(move |lua, ()| {
        let vim = Vim::new(lua);
        let cursor_position = vim.nvim_buf_get_var::<i32>(buffer, "peek_cursor".into())?;
        let next = cursor_position + 1;
        vim.nvim_buf_set_var(buffer, "peek_cursor".into(), LuaValue::Integer(next.into()))?;

        Ok(())
    })?;

    let select_up = lua.create_function(move |lua, ()| {
        let vim = Vim::new(lua);
        let cursor_position = vim.nvim_buf_get_var::<i32>(buffer, "peek_cursor".into())?;
        let next = std::cmp::max(cursor_position - 1, 0);
        vim.nvim_buf_set_var(buffer, "peek_cursor".into(), LuaValue::Integer(next.into()))?;

        Ok(())
    })?;

    // Window/Buffer config
    lua.load("vim.cmd('file Peek')").eval()?;
    vim.nvim_buf_set_var(buffer, "peek_cursor".into(), LuaValue::Integer(0))?;
    vim.nvim_buf_set_keymap(buffer, vim::Mode::Normal, "<ESC>".into(), rhs)?;
    vim.nvim_buf_set_keymap(buffer, vim::Mode::Insert, "<C-j>".into(), select_down)?;
    vim.nvim_buf_set_keymap(buffer, vim::Mode::Insert, "<C-k>".into(), select_up)?;
    vim.nvim_set_hl(
        0,
        "PeekSelection".into(),
        vim::HighlightOptions {
            bg: Some("red".into()),
            fg: Some("white".into()),
        },
    )?;

    let buff_attach_function = lua.create_function(
        |lua, (_lines, buffer, _chaned_tick, first_line_changed): (String, i32, i32, i32)| {
            // Only care about changes to the first line (ie. the prompt)
            if first_line_changed != 0 {
                return Ok(false);
            }

            let vim = Vim::new(lua);
            let lines = vim.nvim_buf_get_lines(buffer, 0, 1, false)?;
            let prompt = lines.first();
            log::info!("{:?}", prompt);

            let callback = lua.create_function(move |lua, ()| {
                let vim = Vim::new(lua);
                vim.nvim_buf_set_lines(buffer, 1, -1, false, lines.clone())
            })?;
            vim.vim_schedule(callback)?;

            Ok(false)
        },
    )?;

    let buf_attach_options = vim::BufferAttachOptions {
        on_lines: Some(buff_attach_function),
    };
    vim.nvim_buf_attach(buffer, true, buf_attach_options)
}
