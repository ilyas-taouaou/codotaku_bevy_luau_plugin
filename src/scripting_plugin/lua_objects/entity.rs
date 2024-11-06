use bevy::prelude::*;
use mlua::prelude::*;

#[derive(Clone, Copy)]
pub struct LuaEntity(pub Entity);

impl LuaUserData for LuaEntity {}

impl FromLua for LuaEntity {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        Ok(value.as_userdata().unwrap().borrow::<Self>()?.clone())
    }
}
