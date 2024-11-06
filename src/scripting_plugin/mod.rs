use bevy::prelude::*;
use lua_objects::{LuaEntity, LuaMesh, LuaTransform, LuaWorld};
use mlua::prelude::*;
use script_asset::{ScriptAsset, ScriptAssetLoader};

mod lua_objects;
mod script_asset;

pub struct ScriptingPlugin {}
impl ScriptingPlugin {
    pub fn new() -> Self {
        ScriptingPlugin {}
    }
}

struct ScriptingState {
    lua: Lua,
}

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        let state = ScriptingState { lua: Lua::new() };
        let globals = state.lua.globals();
        let transform_constructor = state
            .lua
            .create_function(|_lua, properties: LuaTable| {
                let mut transform = Transform::default();

                if let Ok(position) = properties.get::<LuaVector>("position") {
                    transform.translation = Vec3::new(position.x(), position.y(), position.z());
                }

                if let Ok(rotation) = properties.get::<LuaVector>("rotation") {
                    transform.rotation =
                        Quat::from_euler(EulerRot::XYZ, rotation.x(), rotation.y(), rotation.z());
                }

                if let Ok(scale) = properties.get::<LuaVector>("scale") {
                    transform.scale = Vec3::new(scale.x(), scale.y(), scale.z());
                }

                Ok(LuaTransform(transform))
            })
            .unwrap();
        let primitive_constructor = state
            .lua
            .create_function(|lua, properties: LuaTable| {
                let name = properties.get::<String>("type").unwrap();
                match name.as_str() {
                    "Cuboid" => {
                        let half_size = properties.get::<LuaVector>("half_size").unwrap();
                        let half_size = Vec3::new(half_size.x(), half_size.y(), half_size.z());
                        let mesh = LuaMesh::new(Mesh::from(Cuboid { half_size }));
                        Ok(mesh)
                    }
                    _ => Err(mlua::Error::RuntimeError("Unknown primitive".to_string())),
                }
            })
            .unwrap();

        globals.set("Transform", transform_constructor).unwrap();
        globals.set("Primitive", primitive_constructor).unwrap();
        app.insert_non_send_resource(state)
            .init_asset::<ScriptAsset>()
            .init_asset_loader::<ScriptAssetLoader>()
            .add_systems(Update, run_scripts)
            .add_systems(Update, update_scripts);
    }
}

#[derive(Component)]
pub struct RunningScript {
    environment: LuaTable,
}

#[derive(Component)]
pub struct ScriptComponent {
    asset: Handle<ScriptAsset>,
}

impl ScriptComponent {
    pub fn new(asset: Handle<ScriptAsset>) -> Self {
        ScriptComponent { asset }
    }
}

fn run_scripts(world: &mut World) {
    let scripting_state = world.remove_non_send_resource::<ScriptingState>().unwrap();
    let script_assets = world.remove_resource::<Assets<ScriptAsset>>().unwrap();
    let mut query = world.query_filtered::<(Entity, &ScriptComponent), Without<RunningScript>>();

    let mut running_entities = Vec::new();

    for (entity, script) in query.iter_mut(world) {
        running_entities.push((
            entity,
            script_assets
                .get(&script.asset)
                .map(|script| script.code().to_string()),
        ));
    }

    for (entity, code) in running_entities {
        if let Some(code) = code {
            let lua = &scripting_state.lua;

            let environent = lua.create_table().unwrap();

            lua.globals()
                .for_each(|key: LuaValue, value: LuaValue| environent.set(key, value))
                .unwrap();

            lua.sandbox(true).unwrap();
            lua.load(code)
                .set_environment(environent.clone())
                .exec()
                .unwrap();
            let start = environent.get::<LuaFunction>("_start").unwrap();

            lua.scope(|scope| {
                let lua_world = scope.create_userdata(LuaWorld::new(world)).unwrap();
                start.call::<()>((LuaEntity(entity), lua_world))
            })
            .unwrap();
            lua.sandbox(false).unwrap();

            world.entity_mut(entity).insert(RunningScript {
                environment: environent,
            });
        }
    }

    world.insert_non_send_resource(scripting_state);
    world.insert_resource(script_assets);
}

fn update_scripts(world: &mut World) {
    let scripting_state = world.remove_non_send_resource::<ScriptingState>().unwrap();
    let time = world.remove_resource::<Time>().unwrap();
    let mut query = world.query::<(Entity, &RunningScript)>();

    let mut running_entities = Vec::new();

    for (entity, running_script) in query.iter_mut(world) {
        running_entities.push((entity, running_script.environment.clone()));
    }

    for (entity, environment) in running_entities {
        let lua = &scripting_state.lua;
        let update = environment.get::<LuaFunction>("_update").unwrap();

        lua.sandbox(true).unwrap();

        lua.scope(|scope| {
            let lua_world = scope.create_userdata(LuaWorld::new(world)).unwrap();
            update.call::<()>((LuaEntity(entity), lua_world, time.delta_seconds()))
        })
        .unwrap();
        lua.sandbox(false).unwrap();
    }

    world.insert_resource(time);
    world.insert_non_send_resource(scripting_state);
}
