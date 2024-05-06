pub mod functions;
pub mod picker;
pub mod search;
pub mod vim;

use mlua::prelude::*;
use vim::Vim;

pub fn file_picker(lua: &Lua, _: ()) -> LuaResult<()> {
    let config = lua.create_table()?;
    config.set("initial_data", picker::file::initial_data(lua))?;
    config.set("filter", picker::file::filter(lua))?;
    config.set("to_line", picker::file::to_line(lua))?;
    config.set("mappings", picker::file::mappings(lua))?;
    create_window(lua, config)
}

pub fn buffer_picker(lua: &Lua, config: mlua::Table) -> LuaResult<()> {
    config.set("initial_data", picker::buffer::initial_data(lua))?;
    config.set("filter", picker::buffer::filter(lua))?;
    config.set("to_line", picker::buffer::to_line(lua))?;
    config.set("mappings", picker::buffer::mappings(lua))?;
    create_window(lua, config)
}

pub fn create_window(lua: &Lua, config: mlua::Table) -> LuaResult<()> {
    // simple_logging::log_to_file("test.log", LevelFilter::Info).unwrap();
    let globals = lua.globals();
    let initial_data_function: mlua::Function = config.get("initial_data")?;
    let filter_function: mlua::Function = config.get("filter")?;
    let to_line_func: mlua::Function = config.get("to_line")?;
    let mappings_func: mlua::Function = config.get("mappings")?;
    globals.set("peek_filter_func", filter_function)?;
    globals.set("peek_to_line_func", to_line_func)?;

    let vim = Vim::new(lua);
    let buffer = vim.nvim_create_buffer(false, true)?;
    let origin_win = vim.nvim_get_current_win().unwrap();

    lua.load("vim.cmd('bot sp')").eval()?;
    let win = vim.win_get_id()?;
    vim.nvim_win_set_height(win, 20)?;
    vim.nvim_win_set_buf(win, buffer)?;

    // Window/Buffer config
    lua.load("vim.cmd('startinsert')").eval()?;
    lua.load(format!("vim.cmd('file {}')", "File")).eval()?;
    lua.load("require('cmp').setup.buffer { enabled = false }").eval()?;
    lua.load("vim.cmd('set nonu')").eval()?;
    vim.nvim_buf_set_var(buffer, "peek_origin_window".into(), LuaValue::Integer(origin_win.try_into().unwrap()))?;
    vim.nvim_buf_set_var(buffer, "peek_cursor".into(), LuaValue::Integer(0))?;
    vim.nvim_buf_set_var(buffer, "peek_limit".into(), LuaValue::Integer(20))?;
    vim.nvim_buf_set_var(buffer, "peek_offset".into(), LuaValue::Integer(0))?;
    vim.nvim_buf_set_var(buffer, "peek_config".into(), LuaValue::Table(config))?;

    // Assign mappings
    mappings_func.call(())?;

    // Define highlights (need to be moved outside)
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
    vim.nvim_buf_set_var(buffer, "peek_results_count".into(), lua.to_value(&initial_data.len()).unwrap())?;
    render(lua).call(())?;

    let buff_attach_function = lua.create_function(
        move |lua, (_lines, buffer, _changed_tick, first_line_changed): (String, usize, bool, i32)| {
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
                let search_results: Vec<mlua::Value> = filter_function.call(prompt.clone())?;

                let vim = Vim::new(lua);
                vim.nvim_buf_set_var(buffer, "peek_results".into(), lua.to_value(&search_results).unwrap())?;
                vim.nvim_buf_set_var(
                    buffer,
                    "peek_results_count".into(),
                    lua.to_value(&search_results.len()).unwrap(),
                )?;
                vim.nvim_buf_set_var(buffer, "peek_cursor".into(), LuaValue::Integer(1))?;
                vim.nvim_buf_set_var(buffer, "peek_offset".into(), LuaValue::Integer(0))?;
                render(lua).call(())?;
                vim.nvim_buf_add_highlight(buffer, 101, "PeekSelection".into(), 1, 0, -1)?;
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

pub fn render(lua: &Lua) -> mlua::Function {
    lua.create_function(|lua, ()| {
        let vim = Vim::new(lua);
        let buffer = vim.bufnr()?;
        let globals = lua.globals();
        let to_line_function: mlua::Function = globals.get("peek_to_line_func")?;
        let data: Vec<mlua::Value> = vim.nvim_buf_get_var(buffer, "peek_results".into())?;
        let limit: usize = vim.nvim_buf_get_var(buffer, "peek_limit".into())?;
        let offset: usize = vim.nvim_buf_get_var(buffer, "peek_offset".into())?;

        let lines: Vec<String> = data
            .iter()
            .map(|x| to_line_function.call(x).unwrap())
            .skip(offset)
            .take(limit)
            .collect();

        vim.nvim_buf_set_lines(buffer, 1, -1, false, lines)?;
        Ok(())
    })
    .unwrap()
}
