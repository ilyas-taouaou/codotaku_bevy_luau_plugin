use bevy::prelude::*;
use mlua::prelude::*;
use script_asset::{ScriptAsset, ScriptAssetLoader};

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

struct LuaSelf<'world> {
    world: &'world mut World,
    entity: Entity,
}

impl<'world> LuaSelf<'world> {
    fn new(world: &'world mut World, entity: Entity) -> Self {
        LuaSelf { world, entity }
    }
}

impl LuaUserData for LuaSelf<'_> {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("rotate", |_, this, vector: LuaVector| {
            let mut transform = this.world.get_mut::<Transform>(this.entity).unwrap();
            transform.rotate(Quat::from_euler(
                EulerRot::XYZ,
                vector.x(),
                vector.y(),
                vector.z(),
            ));
            Ok(())
        });
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
            let lua_self = LuaSelf::new(world, entity);

            let lua = &scripting_state.lua;

            let environent = lua.create_table().unwrap();

            lua.globals()
                .for_each(|key: LuaValue, value: LuaValue| environent.set(key, value))
                .unwrap();

            lua.load(code)
                .set_environment(environent.clone())
                .exec()
                .unwrap();
            let start = environent.get::<LuaFunction>("start").unwrap();

            lua.scope(|scope| {
                let lua_self = scope.create_userdata(lua_self).unwrap();
                start.call::<()>(lua_self)
            })
            .unwrap();

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
        let lua_self = LuaSelf::new(world, entity);

        let lua = &scripting_state.lua;
        lua.scope(|scope| {
            let update = environment.get::<LuaFunction>("update").unwrap();
            let lua_self = scope.create_userdata(lua_self).unwrap();
            update.call::<()>((lua_self, time.delta_seconds()))
        })
        .unwrap();
    }

    world.insert_resource(time);
    world.insert_non_send_resource(scripting_state);
}
