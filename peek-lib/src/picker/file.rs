use mlua::{Function, Lua, LuaSerdeExt};
use serde::{Deserialize, Serialize};

use crate::functions;
use crate::vim::Vim;

#[derive(Serialize, Deserialize)]
struct File {
    path: String,
}

pub fn filter(lua: &Lua) -> Function {
    lua.create_function(|lua, prompt: String| {
        let command = std::process::Command::new("fd").arg(prompt).output().unwrap();
        let search_results: Vec<File> = std::str::from_utf8(&command.stdout)
            .unwrap()
            .lines()
            .map(|x| File { path: x.to_owned() })
            .take(10)
            .collect();
        let result = lua.to_value(&search_results)?;
        Ok(result)
    })
    .unwrap()
}

pub fn initial_data(lua: &Lua) -> Function {
    lua.create_function(|lua, ()| filter(lua).call::<_, Vec<mlua::Value>>(""))
        .unwrap()
}

pub fn render(lua: &Lua) -> Function {
    lua.create_function(|lua, value: mlua::Value| {
        let data: File = lua.from_value(value)?;
        Ok(data.path)
    })
    .unwrap()
}

pub fn mappings(lua: &Lua) -> Function {
    lua.create_function(|lua, (window, buffer): (i32, i32)| {
        let vim = Vim::new(lua);
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Normal, "<ESC>".into(), functions::exit(lua, window, buffer))
            .unwrap();
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<C-j>".into(), functions::select_down(lua, buffer))?;
        vim.nvim_buf_set_keymap(
            buffer,
            crate::vim::Mode::Insert,
            "<Down>".into(),
            functions::select_down(lua, buffer),
        )?;
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<C-k>".into(), functions::select_up(lua, buffer))?;
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<Up>".into(), functions::select_up(lua, buffer))?;

        Ok(())
    })
    .unwrap()
}
