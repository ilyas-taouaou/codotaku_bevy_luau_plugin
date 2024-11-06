use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

use bevy::prelude::*;
use mlua::prelude::*;

use super::{LuaEntity, LuaTransform};

pub struct LuaWorld<'world> {
    world: &'world mut World,
}

impl<'world> LuaWorld<'world> {
    pub fn new(world: &'world mut World) -> Self {
        LuaWorld { world }
    }
}

impl LuaUserData for LuaWorld<'_> {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "get_component",
            |lua, this, (entity, name): (LuaEntity, String)| match name.as_str() {
                "Transform" => lua.create_userdata(LuaTransform(
                    this.world.get::<Transform>(entity.0).unwrap().clone(),
                )),
                _ => Err(mlua::Error::RuntimeError("Unknown component".to_string())),
            },
        );

        methods.add_method_mut("set_components", |lua, this, properties: LuaTable| {
            let entity = properties.get::<LuaEntity>("entity")?;
            properties.for_each(|key: String, value: LuaValue| {
                if key == "entity" {
                    return Ok(());
                }

                match key.as_str() {
                    "Transform" => {
                        let transform: LuaTransform = FromLua::from_lua(value, lua)?;
                        this.world.entity_mut(entity.0).insert(transform.0);
                        Ok(())
                    }
                    "Mesh" => {
                        let mesh: LuaMeshHandle = FromLua::from_lua(value, lua)?;
                        let transform = this.world.get::<Transform>(entity.0).unwrap().clone();
                        // TODO: Add material to scripting API, instead of hardcoding it here
                        let mut materials = this
                            .world
                            .get_resource_mut::<Assets<StandardMaterial>>()
                            .unwrap();
                        let material = materials.add(StandardMaterial {
                            base_color: Color::WHITE,
                            ..Default::default()
                        });
                        this.world.entity_mut(entity.0).insert(PbrBundle {
                            mesh: mesh.0.clone(),
                            material,
                            transform,
                            ..Default::default()
                        });
                        Ok(())
                    }
                    _ => Err(mlua::Error::RuntimeError(format!(
                        "Unknown component: {}",
                        key
                    ))),
                }
            })
        });

        methods.add_method_mut("add_mesh", |_, this, mesh: LuaMesh| {
            let mut assets = this.world.get_resource_mut::<Assets<Mesh>>().unwrap();
            Ok(LuaMeshHandle(assets.add(mesh.0.read().unwrap().clone())))
        });

        // new_entity
        methods.add_method_mut("new_entity", |_, this, ()| {
            Ok(LuaEntity(this.world.spawn_empty().id()))
        });
    }
}

#[derive(Clone)]
pub struct LuaMeshHandle(pub Handle<Mesh>);
impl LuaUserData for LuaMeshHandle {}

impl FromLua for LuaMeshHandle {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        Ok(value.as_userdata().unwrap().borrow::<Self>()?.clone())
    }
}

#[derive(Clone)]
pub struct LuaMesh(Arc<RwLock<Mesh>>);

impl LuaMesh {
    pub fn new(mesh: Mesh) -> Self {
        LuaMesh(Arc::new(RwLock::new(mesh)))
    }
}

impl LuaUserData for LuaMesh {}

impl FromLua for LuaMesh {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        Ok(value.as_userdata().unwrap().borrow::<Self>()?.clone())
    }
}
