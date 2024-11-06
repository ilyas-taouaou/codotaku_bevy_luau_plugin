local rotation_direction = math.random(2) == 1 and 1 or -1

function _start(self: Entity, world: World)
    local transform = world:get_component(self, "Transform")
    if transform then
        print("Position", transform.position)
        print("Rotation", transform.rotation)
        print("Scale", transform.scale)
    end

    local mesh = world:add_mesh(Primitive {
        type = "Cuboid",
        half_size = vector(0.1, 0.1, 0.1),
    })
 
    for x = -5, 5 do
        for z = -5, 5 do
            local entity = world:new_entity()
            world:set_components {
                entity = entity,
                Transform = Transform {
                    position = vector(x, math.random(-1.0, 1.0), z),
                }, 
                Mesh = mesh,
            }
        end
    end
end
 
function _update(self: Entity, world: World, delta_time: number)
    local transform: Transform = world:get_component(self, "Transform")
    if transform then
        transform = transform:rotate(vector(0, rotation_direction * delta_time, 0))
        world:set_components {
            entity = self,
            Transform = transform,
        }
    end
end