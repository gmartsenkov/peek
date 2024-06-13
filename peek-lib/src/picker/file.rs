use mlua::prelude::*;
use mlua::{FromLua, Lua, LuaSerdeExt};
use serde::{Deserialize, Serialize};

use crate::functions;
use crate::vim::Vim;

#[derive(Serialize, Deserialize)]
pub struct File {
    path: String,
}

impl<'lua> FromLua<'lua> for File {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

pub fn filter(lua: &Lua, prompt: String) -> LuaResult<LuaValue> {
    let mut binding = std::process::Command::new("fd");
    let cmd = binding
        .arg("-t")
        .arg("file")
        .arg("-H")
        .arg("-E")
        .arg(".git")
        .output()
        .unwrap()
        .stdout;
    let fzf_output = crate::search::fzf(prompt, cmd);

    let search_results: Vec<File> = fzf_output
        .iter()
        .take(500)
        .map(|x| File { path: x.to_owned() })
        .collect();
    let result = lua.to_value(&search_results)?;
    Ok(result)
}

pub fn to_line(_lua: &Lua, data: File) -> LuaResult<String> {
    Ok(data.path)
}

pub fn open_file(lua: &Lua, _: ()) -> LuaResult<()> {
    let selected: Option<mlua::Value> = functions::selected_value(lua, ())?;

    if let Some(f) = selected {
        let file: File = lua.from_value(f)?;
        let vim = Vim::new(lua);
        let origin_window: usize = functions::origin_window(lua, ())?;
        let inner_func = lua.create_function(move |lua, ()| {
            let vim = Vim::new(lua);
            vim.edit_file(file.path.as_str()).ok();
            Ok(())
        })?;
        functions::exit(lua, ())?;
        vim.nvim_win_call(origin_window, inner_func)?;
        vim.nvim_set_current_win(origin_window)?;
    }
    Ok(())
}
