use mlua::{FromLua, Function, Lua, LuaSerdeExt};
use serde::{Deserialize, Serialize};

use crate::functions;
use crate::vim::Vim;

#[derive(Serialize, Deserialize)]
struct File {
    path: String,
}

impl<'lua> FromLua<'lua> for File {
    fn from_lua(value: mlua::prelude::LuaValue<'lua>, lua: &'lua Lua) -> mlua::prelude::LuaResult<Self> {
        lua.from_value(value)
    }
}

pub fn filter(lua: &Lua) -> Function {
    lua.create_function(|lua, prompt: String| {
        let mut binding = std::process::Command::new("fd");
        let cmd = binding.arg("-t").arg("file").output().unwrap().stdout;
        let fzf_output = crate::search::fzf(prompt, cmd);

        let search_results: Vec<File> = fzf_output
            .iter()
            .take(500)
            .map(|x| File { path: x.to_owned() })
            .collect();
        let result = lua.to_value(&search_results)?;
        Ok(result)
    })
    .unwrap()
}

pub fn to_line(lua: &Lua) -> Function {
    lua.create_function(|_lua, data: File| Ok(data.path)).unwrap()
}

pub fn on_open(lua: &Lua) -> Function {
    lua.create_function(|lua, ()| {
        let vim = Vim::new(lua);
        let buffer = vim.nvim_get_current_buf().unwrap();
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Normal, "<ESC>", functions::exit(lua))
            .unwrap();
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<ESC>", functions::exit(lua))
            .unwrap();
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<C-j>", functions::select_down(lua))?;
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<Down>", functions::select_down(lua))?;
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<C-k>", functions::select_up(lua))?;
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<Up>", functions::select_up(lua))?;
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<CR>", open_file(lua))?;

        Ok(())
    })
    .unwrap()
}

pub fn open_file(lua: &Lua) -> Function {
    lua.create_function(move |lua, ()| {
        let selected: Option<File> = functions::selected_value(lua).call(())?;

        if let Some(file) = selected {
            let vim = Vim::new(lua);
            let origin_window: usize = functions::origin_window(lua).call(())?;
            let inner_func = lua.create_function(move |lua, ()| {
                let vim = Vim::new(lua);
                vim.edit_file(file.path.as_str()).ok();
                Ok(())
            })?;
            functions::exit(lua).call(())?;
            vim.nvim_win_call(origin_window, inner_func)?;
            vim.nvim_set_current_win(origin_window)?;
        }
        Ok(())
    })
    .unwrap()
}
