use std::fs::read_dir;
use std::path::Path;

use mlua::prelude::*;
use mlua::{FromLua, Lua, LuaSerdeExt};
use serde::{Deserialize, Serialize};

use crate::functions;
use crate::vim::Vim;

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    path: String,
    full_path: String,
    is_dir: bool,
}

impl<'lua> FromLua<'lua> for File {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

pub fn filter(lua: &Lua, prompt: String) -> LuaResult<LuaValue> {
    let mut x = prompt.clone();
    x.push(' ');
    let path = Path::new(&x).with_file_name("").as_path().to_owned();
    let query = prompt.split('/').last().unwrap_or("").to_string();

    if read_dir(&path).is_err() {
        let result: Vec<File> = vec![];
        return lua.to_value(&result);
    }

    let entries: Vec<File> = read_dir(&path)
        .unwrap()
        .map(|res| {
            let p = res.as_ref().map(|e| e.path()).unwrap();
            let meta = res.as_ref().map(|e| e.metadata().unwrap()).unwrap();

            let new_path = p.strip_prefix(&path).unwrap();
            File {
                full_path: p.to_str().unwrap().to_string(),
                path: new_path.as_os_str().to_str().unwrap().to_string(),
                is_dir: meta.is_dir(),
            }
        })
        .take(500)
        .collect();

    let results = entries
        .iter()
        .map(|x| x.path.to_owned())
        .collect::<Vec<String>>()
        .join("\n")
        .as_bytes()
        .to_vec();

    let matches = crate::search::fzf(query, results);
    let filtered: Vec<File> = entries.into_iter().filter(|x| matches.contains(&x.path)).collect();

    let result = lua.to_value(&filtered)?;
    Ok(result)
}

pub fn to_line(_lua: &Lua, data: File) -> LuaResult<String> {
    if data.is_dir {
        return Ok(format!("{}/", data.path));
    }
    Ok(data.path)
}

pub fn select_option(lua: &Lua, _: ()) -> LuaResult<()> {
    let vim = Vim::new(lua);
    let selected: Option<mlua::Value> = functions::selected_value(lua, ())?;

    if let Some(selected_buffer) = selected {
        let file: File = lua.from_value(selected_buffer)?;

        let file_path = Path::new(&file.full_path);
        if let Ok(m) = file_path.metadata() {
            if m.is_dir() {
                let new_path = format!("{}/", file.full_path);
                crate::update_prompt(lua, new_path)?;
            }

            if m.is_file() {
                open_file(lua, file.full_path)?;
            }
        }
    } else {
        let lines = vim.nvim_buf_get_lines(0, 0, 1, false)?;
        let prompt = lines.first().unwrap();
        open_file(lua, prompt.to_string())?;
    }

    Ok(())
}

pub fn tab(lua: &Lua, _: ()) -> LuaResult<()> {
    let selected: Option<mlua::Value> = functions::selected_value(lua, ())?;

    if let Some(selected_buffer) = selected {
        let file: File = lua.from_value(selected_buffer)?;
        let path = Path::new(&file.full_path);
        if let Ok(meta) = path.metadata() {
            if meta.is_dir() {
                crate::update_prompt(lua, format!("{}/", file.full_path))?;
            } else {
                crate::update_prompt(lua, file.full_path)?;
            }
        }
    }

    Ok(())
}

fn open_file(lua: &Lua, path: String) -> LuaResult<()> {
    let vim = Vim::new(lua);
    let origin_window: usize = functions::origin_window(lua, ())?;
    let inner_func = lua.create_function(move |lua, ()| {
        let vim = Vim::new(lua);
        vim.edit_file(path.as_str()).ok();
        Ok(())
    })?;
    functions::exit(lua, ())?;
    vim.nvim_win_call(origin_window, inner_func)?;
    vim.nvim_set_current_win(origin_window)
}

pub fn backspace(lua: &Lua, _: ()) -> LuaResult<()> {
    let vim = Vim::new(lua);
    let lines = vim.nvim_buf_get_lines(0, 0, 1, false)?;
    let mut prompt = lines.first().unwrap().clone();
    let path = Path::new(&prompt);
    if let Ok(meta) = path.metadata() {
        if meta.is_dir() {
            let new_path = path.parent().unwrap().as_os_str().to_str().to_owned().unwrap();
            crate::update_prompt(lua, format!("{}/", new_path))?;
            return Ok(());
        }
    }

    prompt.pop();
    crate::update_prompt(lua, prompt)?;

    Ok(())
}
