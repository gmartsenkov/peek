use mlua::prelude::*;
use mlua::{FromLua, Lua, LuaSerdeExt};
use serde::{Deserialize, Serialize};

use crate::vim::{GetOptionValue, Vim};
use crate::{functions, search, Config};

#[derive(Serialize, Deserialize)]
pub struct Buffer {
    id: usize,
    name: String,
}

impl<'lua> FromLua<'lua> for Buffer {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

pub fn filter(lua: &Lua, prompt: String) -> LuaResult<LuaValue> {
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
}

fn listed_buffers(lua: &Lua) -> Vec<Buffer> {
    let vim = Vim::new(lua);
    let config = Config::new(lua);
    let buffer_ids = vim.nvim_list_bufs().unwrap();
    buffer_ids
        .into_iter()
        .filter(|id| {
            vim.nvim_get_option_value("buflisted", GetOptionValue { buf: Some(*id) })
                .unwrap()
        })
        .map(|id| {
            let name = vim.nvim_buf_get_name(id).unwrap();

            if let Some(cwd) = &config.cwd {
                return Buffer {
                    id,
                    name: name.strip_prefix(cwd).unwrap_or(&name).to_string(),
                };
            }

            Buffer { id, name }
        })
        .collect()
}

pub fn to_line(_lua: &Lua, buffer: Buffer) -> LuaResult<String> {
    Ok(buffer.name)
}

pub fn on_open(lua: &Lua, _: ()) -> LuaResult<()> {
    let vim = Vim::new(lua);
    let buffer = vim.nvim_get_current_buf().unwrap();
    vim.nvim_buf_set_keymap(buffer, "n", "<ESC>", LuaValue::Function(lua.create_function(functions::exit)?))?;
    vim.nvim_buf_set_keymap(buffer, "i", "<ESC>", LuaValue::Function(lua.create_function(functions::exit)?))?;
    vim.nvim_buf_set_keymap(buffer, "i", "<C-j>", LuaValue::Function(lua.create_function(functions::select_down)?))?;
    vim.nvim_buf_set_keymap(buffer, "i", "<Down>", LuaValue::Function(lua.create_function(functions::select_down)?))?;
    vim.nvim_buf_set_keymap(buffer, "i", "<C-k>", LuaValue::Function(lua.create_function(functions::select_up)?))?;
    vim.nvim_buf_set_keymap(buffer, "i", "<Up>", LuaValue::Function(lua.create_function(functions::select_up)?))?;
    vim.nvim_buf_set_keymap(buffer, "i", "<CR>", LuaValue::Function(lua.create_function(open_buffer)?))?;

    Ok(())
}

pub fn open_buffer(lua: &Lua, _: ()) -> LuaResult<()> {
    let selected: Option<mlua::Value> = functions::selected_value(lua, ())?;

    if let Some(selected_buffer) = selected {
        let buf: Buffer = lua.from_value(selected_buffer)?;
        let vim = Vim::new(lua);
        let origin_window: usize = functions::origin_window(lua, ())?;

        functions::exit(lua, ())?;
        vim.nvim_win_set_buf(origin_window, buf.id)?;
        vim.nvim_set_current_win(origin_window)?;
    }
    Ok(())
}
