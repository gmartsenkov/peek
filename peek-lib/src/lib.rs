pub mod functions;
pub mod picker;
pub mod search;
pub mod vim;

use mlua::prelude::*;
use vim::Vim;

#[derive(serde::Deserialize, Debug)]
pub struct Config<'a> {
    #[serde(skip)]
    table: Option<mlua::Table<'a>>,
    pub cwd: Option<String>,
}

impl<'a> Config<'a> {
    pub fn new(lua: &'a Lua) -> Config {
        let vim = Vim::new(lua);
        vim.nvim_buf_get_var::<Config>(0, "peek_config").unwrap()
    }

    pub fn on_open_callback(&self) -> mlua::prelude::LuaResult<()> {
        self.table
            .as_ref()
            .unwrap()
            .get::<_, mlua::Function>("on_open")
            .unwrap()
            .call(())
    }

    pub fn filter(&self, prompt: String) -> mlua::prelude::LuaResult<Vec<mlua::Value>> {
        self.table
            .as_ref()
            .unwrap()
            .get::<_, mlua::Function>("filter")
            .unwrap()
            .call(prompt)
    }

    pub fn to_line(&self, value: &'a mlua::Value) -> mlua::prelude::LuaResult<String> {
        self.table
            .as_ref()
            .unwrap()
            .get::<_, mlua::Function>("to_line")
            .unwrap()
            .call(value)
    }
}

impl<'lua> FromLua<'lua> for Config<'lua> {
    fn from_lua(value: mlua::prelude::LuaValue<'lua>, lua: &'lua Lua) -> mlua::prelude::LuaResult<Self> {
        let mut options = mlua::DeserializeOptions::new();
        options.deny_unsupported_types = false;
        let mut b: Config = lua.from_value_with(value.clone(), options).unwrap();
        if let LuaValue::Table(t) = value {
            b.table = Some(t);
        }
        Ok(b)
    }
}

pub fn create_window(lua: &Lua, config: mlua::Table) -> LuaResult<()> {
    // simple_logging::log_to_file("test.log", log::LevelFilter::Info).unwrap();

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
    vim.nvim_buf_set_var(buffer, "peek_origin_window", LuaValue::Integer(origin_win.try_into().unwrap()))?;
    vim.nvim_buf_set_var(buffer, "peek_cursor", LuaValue::Integer(0))?;
    vim.nvim_buf_set_var(buffer, "peek_limit", LuaValue::Integer(20))?;
    vim.nvim_buf_set_var(buffer, "peek_offset", LuaValue::Integer(0))?;
    vim.nvim_buf_set_var(buffer, "peek_config", LuaValue::Table(config.clone()))?;

    let conf = Config::new(lua);
    let prompt = config.get::<_, String>("prompt");

    let custom_mappings = config.get("mappings").unwrap_or(lua.create_table().unwrap());
    apply_mappings(lua, buffer, default_mappings(lua));
    apply_mappings(lua, buffer, custom_mappings);

    // Define highlights (need to be moved outside)
    vim.nvim_set_hl(
        0,
        "PeekSelection",
        vim::HighlightOptions {
            bg: Some("purple".into()),
            fg: Some("white".into()),
        },
    )?;

    let initial_data: Vec<mlua::Value> = conf.filter("".to_string())?;
    vim.nvim_buf_set_var(buffer, "peek_results", lua.to_value(&initial_data).unwrap())?;
    vim.nvim_buf_set_var(buffer, "peek_results_count", lua.to_value(&initial_data.len()).unwrap())?;
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
                let config = Config::new(lua);
                let search_results: Vec<mlua::Value> = config.filter(prompt.clone()).unwrap();

                let vim = Vim::new(lua);
                vim.nvim_buf_set_var(buffer, "peek_results", lua.to_value(&search_results).unwrap())?;
                vim.nvim_buf_set_var(buffer, "peek_results_count", lua.to_value(&search_results.len()).unwrap())?;
                vim.nvim_buf_set_var(buffer, "peek_cursor", LuaValue::Integer(1))?;
                vim.nvim_buf_set_var(buffer, "peek_offset", LuaValue::Integer(0))?;
                render(lua).call(())?;
                vim.nvim_buf_add_highlight(buffer, 101, "PeekSelection", 1, 0, -1)?;
                Ok(())
            })?;
            vim.vim_schedule(callback)?;

            Ok(false)
        },
    )?;

    let buf_attach_options = vim::BufferAttachOptions {
        on_lines: Some(buff_attach_function),
    };
    vim.nvim_buf_attach(buffer, false, buf_attach_options)?;

    if let Ok(p) = prompt {
        crate::update_prompt(lua, p)?;
    }

    Ok(())
}

pub fn render(lua: &Lua) -> mlua::Function {
    lua.create_function(|lua, ()| {
        let vim = Vim::new(lua);
        let buffer = vim.bufnr()?;
        let config = Config::new(lua);
        let data: Vec<mlua::Value> = vim.nvim_buf_get_var(buffer, "peek_results")?;
        let limit: usize = vim.nvim_buf_get_var(buffer, "peek_limit")?;
        let offset: usize = vim.nvim_buf_get_var(buffer, "peek_offset")?;

        let lines: Vec<String> = data
            .iter()
            .map(|x| config.to_line(x).unwrap())
            .skip(offset)
            .take(limit)
            .collect();

        vim.nvim_buf_set_lines(buffer, 1, -1, false, lines)?;
        Ok(())
    })
    .unwrap()
}

pub fn apply_mappings(lua: &Lua, buffer: usize, table: mlua::Table) {
    let vim = Vim::new(lua);
    for pair in table.pairs::<String, mlua::Table>() {
        let (mode, keymaps) = pair.unwrap();

        for pair in keymaps.pairs::<String, mlua::Value>() {
            let (lhs, rhs) = pair.unwrap();
            vim.nvim_buf_set_keymap(buffer, mode.as_str(), lhs.as_str(), rhs)
                .unwrap();
        }
    }
}

pub fn default_mappings(lua: &Lua) -> mlua::Table<'_> {
    let table = lua.create_table().unwrap();
    let insert = lua.create_table().unwrap();
    let normal = lua.create_table().unwrap();

    normal
        .set("<ESC>", lua.create_function(functions::exit).unwrap())
        .unwrap();
    insert
        .set("<ESC>", lua.create_function(functions::exit).unwrap())
        .unwrap();
    insert
        .set("<C-j>", lua.create_function(functions::select_down).unwrap())
        .unwrap();
    insert
        .set("<Down>", lua.create_function(functions::select_down).unwrap())
        .unwrap();
    insert
        .set("<C-k>", lua.create_function(functions::select_up).unwrap())
        .unwrap();
    insert
        .set("<Up>", lua.create_function(functions::select_up).unwrap())
        .unwrap();

    table.set("i", insert).unwrap();
    table.set("n", normal).unwrap();
    table
}

pub fn update_prompt(lua: &Lua, prompt: String) -> LuaResult<()> {
    let vim = Vim::new(lua);
    vim.nvim_buf_set_lines(0, 0, 1, false, vec![prompt])?;
    vim.nvim_win_set_cursor(0, vec![1, 1000])
}
