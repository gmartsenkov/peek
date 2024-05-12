use mlua::{FromLua, Function, Lua, LuaSerdeExt};
use serde::{Deserialize, Serialize};

use crate::vim::{GetOptionValue, Vim};
use crate::{functions, search, Config};

#[derive(Serialize, Deserialize)]
struct Buffer {
    id: usize,
    name: String,
}

impl<'lua> FromLua<'lua> for Buffer {
    fn from_lua(value: mlua::prelude::LuaValue<'lua>, lua: &'lua Lua) -> mlua::prelude::LuaResult<Self> {
        lua.from_value(value)
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

pub fn to_line(lua: &Lua) -> Function {
    lua.create_function(|_lua, buffer: Buffer| Ok(buffer.name)).unwrap()
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
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<CR>", open_buffer(lua))?;

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

            functions::exit(lua).call(())?;
            vim.nvim_win_set_buf(origin_window, selected_buffer.id)?;
            vim.nvim_set_current_win(origin_window)?;
        }
        Ok(())
    })
    .unwrap()
}
