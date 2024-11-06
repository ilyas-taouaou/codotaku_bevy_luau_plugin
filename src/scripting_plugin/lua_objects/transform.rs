use bevy::prelude::*;
use mlua::prelude::*;

#[derive(Clone, Copy)]
pub struct LuaTransform(pub Transform);

impl FromLua for LuaTransform {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        Ok(value.as_userdata().unwrap().borrow::<Self>()?.clone())
    }
}

impl LuaUserData for LuaTransform {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("position", |_, this| {
            Ok(LuaVector::new(
                this.0.translation.x,
                this.0.translation.y,
                this.0.translation.z,
            ))
        });

        fields.add_field_method_get("scale", |_, this| {
            Ok(LuaVector::new(
                this.0.scale.x,
                this.0.scale.y,
                this.0.scale.z,
            ))
        });

        fields.add_field_method_get("rotation", |_, this| {
            let (x, y, z) = this.0.rotation.to_euler(EulerRot::XYZ);
            Ok(LuaVector::new(x, y, z))
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("rotate", |_, this, vector: LuaVector| {
            this.0.rotate(Quat::from_euler(
                EulerRot::XYZ,
                vector.x(),
                vector.y(),
                vector.z(),
            ));
            Ok(this.clone())
        });
    }
}
