declare function vector(x: number, y: number, z: number): Vector3

declare class Entity
    
end

declare class Transform
    position: Vector3
    scale: Vector3
    rotate: (transform: Transform, rotation: Vector3) -> Transform
end

declare class World
    get_component: (world: World, entity: Entity, name: string) -> any
    set_component: (world: World, entity: Entity, name: string, component: any) -> ()
end