use mlua::{FromLua, Function, Lua, LuaSerdeExt};
use serde::{Deserialize, Serialize};

use crate::vim::{GetOptionValue, Vim};
use crate::{functions, search};

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
        let vim = Vim::new(lua);
        let prompt_tokens = prompt.split(' ').collect();
        let buffer_ids = vim.nvim_list_bufs()?;
        let listed_buffers: Vec<Buffer> = buffer_ids
            .into_iter()
            .filter(|id| {
                vim.nvim_get_option_value("buflisted".into(), GetOptionValue { buf: Some(*id) })
                    .unwrap()
            })
            .map(|id| Buffer {
                id,
                name: vim.nvim_buf_get_name(id).unwrap(),
            })
            .filter(|buffer| search::contains(&prompt_tokens, &buffer.name))
            .collect();

        let result = lua.to_value(&listed_buffers)?;
        Ok(result)
    })
    .unwrap()
}

pub fn initial_data(lua: &Lua) -> Function {
    lua.create_function(|lua, ()| filter(lua).call::<_, Vec<mlua::Value>>(""))
        .unwrap()
}

pub fn to_line(lua: &Lua) -> Function {
    lua.create_function(|_lua, buffer: Buffer| Ok(buffer.name)).unwrap()
}

pub fn mappings(lua: &Lua) -> Function {
    lua.create_function(|lua, (window, buffer): (usize, usize)| {
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
        vim.nvim_buf_set_keymap(buffer, crate::vim::Mode::Insert, "<CR>".into(), open_buffer(lua, buffer, window))?;

        Ok(())
    })
    .unwrap()
}

pub fn open_buffer(lua: &Lua, buffer: usize, window: usize) -> Function {
    lua.create_function(move |lua, ()| {
        let selected: Option<Buffer> = functions::selected_value(lua, buffer).call(())?;

        if let Some(selected_buffer) = selected {
            let vim = Vim::new(lua);
            let origin_window: usize = functions::origin_window(lua, buffer).call(())?;

            vim.nvim_win_set_buf(origin_window, selected_buffer.id)?;
            functions::exit(lua, window, buffer).call(())?;
        }
        Ok(())
    })
    .unwrap()
}
