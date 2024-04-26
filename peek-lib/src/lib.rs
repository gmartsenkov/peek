pub mod functions;
pub mod picker;
pub mod vim;

use log::LevelFilter;
use mlua::prelude::*;
use vim::{Vim, WindowOptions};

pub fn nvim_get_current_buf(lua: &Lua, _: ()) -> LuaResult<i32> {
    let vim = Vim::new(lua);
    vim.nvim_get_current_buf()
}

pub fn file_picker(lua: &Lua, _: ()) -> LuaResult<()> {
    let config = lua.create_table()?;
    config.set("initial_data", picker::file::initial_data(lua))?;
    config.set("filter", picker::file::filter(lua))?;
    config.set("render", picker::file::render(lua))?;
    create_window(lua, config)
}

pub fn create_window(lua: &Lua, config: mlua::Table) -> LuaResult<()> {
    simple_logging::log_to_file("test.log", LevelFilter::Info).unwrap();
    let globals = lua.globals();
    let initial_data_function: mlua::Function = config.get("initial_data")?;
    let filter_function: mlua::Function = config.get("filter")?;
    let render_func: mlua::Function = config.get("render")?;
    globals.set("peek_filter_func", filter_function)?;
    globals.set("peek_render_func", render_func)?;

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
    lua.load(format!("vim.cmd('file {}')", "File")).eval()?;
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

    let initial_data: Vec<mlua::Value> = initial_data_function.call(())?;
    vim.nvim_buf_set_var(buffer, "peek_results".into(), lua.to_value(&initial_data).unwrap())?;
    let render_function: mlua::Function = globals.get("peek_render_func")?;
    let lines: Vec<String> = initial_data
        .clone()
        .iter()
        .map(|x| render_function.call(x).unwrap())
        .collect();

    vim.nvim_buf_set_lines(buffer, 1, -1, false, lines)?;

    let buff_attach_function = lua.create_function(
        move |lua, (_lines, buffer, _changed_tick, first_line_changed): (String, i32, bool, i32)| {
            // Only care about changes to the first line (ie. the prompt)
            if first_line_changed > 0 {
                return Ok(false);
            }

            let vim = Vim::new(lua);
            let lines = vim.nvim_buf_get_lines(buffer, 0, 1, false)?;
            let prompt = lines.first().unwrap().clone();

            let callback = lua.create_function(move |lua, ()| {
                let globals = lua.globals();
                let filter_function: mlua::Function = globals.get("peek_filter_func")?;
                let render_function: mlua::Function = globals.get("peek_render_func")?;
                let search_results: Vec<mlua::Value> = filter_function.call(prompt.clone())?;

                let lines: Vec<String> = search_results
                    .clone()
                    .iter()
                    .map(|x| render_function.call(x).unwrap())
                    .collect();

                let vim = Vim::new(lua);
                vim.nvim_buf_set_var(buffer, "peek_results".into(), lua.to_value(&search_results).unwrap())?;
                vim.nvim_buf_set_lines(buffer, 1, -1, false, lines)?;
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
