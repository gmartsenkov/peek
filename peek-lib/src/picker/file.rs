use mlua::{Function, Lua, LuaSerdeExt};
use serde::{Deserialize, Serialize};

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
