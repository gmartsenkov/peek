use mlua::{FromLua, Function, Lua, LuaSerdeExt};
use serde::{Deserialize, Serialize};

use crate::vim::{GetOptionValue, Vim};
use crate::{functions, search};

#[derive(Serialize, Deserialize)]
struct Buffer {
    id: usize,
    name: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Config {
    cwd: Option<String>,
}

impl<'lua> FromLua<'lua> for Buffer {
    fn from_lua(value: mlua::prelude::LuaValue<'lua>, lua: &'lua Lua) -> mlua::prelude::LuaResult<Self> {
        lua.from_value(value)
    }
}

impl<'lua> FromLua<'lua> for Config {
    fn from_lua(value: mlua::prelude::LuaValue<'lua>, lua: &'lua Lua) -> mlua::prelude::LuaResult<Self> {
        let mut options = mlua::DeserializeOptions::new();
        options.deny_unsupported_types = false;
        lua.from_value_with(value, options)
    }
}

pub fn filter(lua: &Lua) -> Function {
    lua.create_function(|lua, prompt: String| {
        let _vim = Vim::new(lua);
        let listed_buffers = listed_buffers(lua);
        let buffer_names: String = listed_buffers
            .iter()
            .map(|x| x.name.clone())
            .collect::<Vec<String>>()
            .join("\n");
        let matches = search::fzf(prompt, buffer_names.as_bytes().to_vec());
        let filtered_buffers: Vec<Buffer> = listed_buffers
            .into_iter()
            .filter(|x| matches.contains(&x.name))
            .collect();

        let result = lua.to_value(&filtered_buffers)?;
        Ok(result)
    })
    .unwrap()
}

fn listed_buffers(lua: &Lua) -> Vec<Buffer> {
    let vim = Vim::new(lua);
    let buffer_ids = vim.nvim_list_bufs().unwrap();
    buffer_ids
        .into_iter()
        .filter(|id| {
            vim.nvim_get_option_value("buflisted".into(), GetOptionValue { buf: Some(*id) })
                .unwrap()
        })
        .map(|id| Buffer {
            id,
            name: vim.nvim_buf_get_name(id).unwrap(),
        })
        .collect()
}
pub fn initial_data(lua: &Lua) -> Function {
    lua.create_function(|lua, ()| filter(lua).call::<_, Vec<mlua::Value>>(""))
        .unwrap()
}

pub fn to_line(lua: &Lua) -> Function {
    lua.create_function(|_lua, buffer: Buffer| Ok(buffer.name)).unwrap()
}

pub fn mappings(lua: &Lua) -> Function {
    lua.create_function(|lua, ()| {
        let vim = Vim::new(lua);
        let buffer = vim.nvim_get_current_buf().unwrap();
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Normal, "<ESC>".into(), functions::exit(lua))
            .unwrap();
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<ESC>".into(), functions::exit(lua))
            .unwrap();
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<C-j>".into(), functions::select_down(lua))?;
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<Down>".into(), functions::select_down(lua))?;
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<C-k>".into(), functions::select_up(lua))?;
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<Up>".into(), functions::select_up(lua))?;
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<CR>".into(), open_buffer(lua))?;

        Ok(())
    })
    .unwrap()
}

pub fn open_buffer(lua: &Lua) -> Function {
    lua.create_function(move |lua, ()| {
        let selected: Option<Buffer> = functions::selected_value(lua).call(())?;

        if let Some(selected_buffer) = selected {
            let vim = Vim::new(lua);
            let origin_window: usize = functions::origin_window(lua).call(())?;

            vim.nvim_win_set_buf(origin_window, selected_buffer.id)?;
            functions::exit(lua).call(())?;
        }
        Ok(())
    })
    .unwrap()
}
