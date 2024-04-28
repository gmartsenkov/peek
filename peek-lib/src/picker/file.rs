use mlua::{FromLua, Function, Lua, LuaSerdeExt};
use serde::{Deserialize, Serialize};
use std::io::Write;

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
        let mut command = std::process::Command::new("fzf")
            .arg("--filter")
            .arg(&prompt)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        let mut stdin = command.stdin.take().expect("Failed to open stdin");
        std::thread::spawn(move || {
            let cmd = std::process::Command::new("fd").arg("-t").arg("file").output().unwrap();
            stdin.write_all(&cmd.stdout).expect("Failed to write to stdin");
        });
        let output = command.wait_with_output().unwrap();

        let search_results: Vec<File> = std::str::from_utf8(&output.stdout)
            .unwrap()
            .lines()
            .take(500)
            .map(|x| File { path: x.to_owned() })
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

pub fn to_line(lua: &Lua) -> Function {
    lua.create_function(|_lua, data: File| Ok(data.path)).unwrap()
}

pub fn mappings(lua: &Lua) -> Function {
    lua.create_function(|lua, (window, buffer): (i32, i32)| {
        let vim = Vim::new(lua);
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Normal, "<ESC>".into(), functions::exit(lua, window, buffer))
            .unwrap();
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<ESC>".into(), functions::exit(lua, window, buffer))
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
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<CR>".into(), open_file(lua, buffer, window))?;

        Ok(())
    })
    .unwrap()
}

pub fn open_file(lua: &Lua, buffer: i32, window: i32) -> Function {
    lua.create_function(move |lua, ()| {
        let selected: Option<File> = functions::selected_value(lua, buffer).call(())?;

        if let Some(file) = selected {
            let vim = Vim::new(lua);
            let origin_window: i32 = functions::origin_window(lua, buffer).call(())?;
            let inner_func = lua.create_function(move |lua, ()| {
                let vim = Vim::new(lua);
                vim.edit_file(file.path.clone()).ok();
                Ok(())
            })?;
            vim.nvim_win_call(origin_window, inner_func)?;
            vim.nvim_set_current_win(origin_window)?;
            functions::exit(lua, window, buffer).call(())?;
        }
        Ok(())
    })
    .unwrap()
}
