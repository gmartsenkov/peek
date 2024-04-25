pub mod functions;
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

    // Window/Buffer config
    lua.load("vim.cmd('file Peek')").eval()?;
    lua.load("require('cmp').setup.buffer { enabled = false }").eval()?;
    vim.nvim_buf_set_var(buffer, "peek_cursor".into(), LuaValue::Integer(0))?;
    vim.nvim_buf_set_keymap(buffer, vim::Mode::Normal, "<ESC>".into(), functions::exit(lua, win, buffer))?;
    vim.nvim_buf_set_keymap(buffer, vim::Mode::Insert, "<C-j>".into(), functions::select_down(lua, buffer))?;
    vim.nvim_buf_set_keymap(buffer, vim::Mode::Insert, "<Down>".into(), functions::select_down(lua, buffer))?;
    vim.nvim_buf_set_keymap(buffer, vim::Mode::Insert, "<C-k>".into(), functions::select_up(lua, buffer))?;
    vim.nvim_buf_set_keymap(buffer, vim::Mode::Insert, "<Up>".into(), functions::select_up(lua, buffer))?;
    vim.nvim_set_hl(
        0,
        "PeekSelection".into(),
        vim::HighlightOptions {
            bg: Some("purple".into()),
            fg: Some("white".into()),
        },
    )?;

    let buff_attach_function = lua.create_function(
        move |lua, (_lines, buffer, _changed_tick, first_line_changed): (String, i32, bool, i32)| {
            // Only care about changes to the first line (ie. the prompt)
            if first_line_changed > 0 {
                return Ok(false);
            }
            log::info!("First line changed {:?}", first_line_changed);

            let vim = Vim::new(lua);

            if first_line_changed != 0 {
                return Ok(false);
            }

            let lines = vim.nvim_buf_get_lines(buffer, 0, 1, false)?;
            let prompt = lines.first().unwrap();
            let command = std::process::Command::new("fd").arg(prompt).output().unwrap();
            let search_results: Vec<String> = std::str::from_utf8(&command.stdout)
                .unwrap()
                .lines()
                .map(|x| x.to_owned())
                .take(10)
                .collect();

            log::info!("{:?}", prompt);

            let callback = lua.create_function(move |lua, ()| {
                let vim = Vim::new(lua);
                vim.nvim_buf_set_var(buffer, "peek_results".into(), lua.to_value(&search_results).unwrap())?;
                vim.nvim_buf_set_lines(buffer, 1, -1, false, search_results.clone())?;

                Ok(())
            })?;
            vim.vim_schedule(callback)?;

            Ok(false)
        },
    )?;

    let buf_attach_options = vim::BufferAttachOptions {
        on_lines: Some(buff_attach_function),
    };
    vim.nvim_buf_attach(buffer, false, buf_attach_options)
}
