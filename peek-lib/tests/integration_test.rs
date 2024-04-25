use mlua::Lua;
use peek_lib::vim::Vim;

#[test]
fn test_nvim_get_current_buffer() {
    let lua = Lua::new();
    let globals = lua.globals();
    let vim = lua.create_table().unwrap();
    let api = lua.create_table().unwrap();
    let nvim_get_current_buf = lua.create_function(|_, ()| Ok(1)).unwrap();
    api.set("nvim_get_current_buf", nvim_get_current_buf).unwrap();
    vim.set("api", api).unwrap();
    globals.set("vim", vim).unwrap();

    let vi = Vim::new(&lua);
    assert_eq!(vi.nvim_get_current_buf().unwrap(), 1);
}
