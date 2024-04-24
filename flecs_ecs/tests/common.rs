#![allow(dead_code)]
#![allow(unused_imports)]

use flecs_ecs::prelude::*;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    unsafe {
        flecs_ecs::sys::ecs_os_init();
    }
}

#[derive(Component)]
pub struct QueryWrapper {
    pub query_entity: Entity,
}

#[derive(Component)]
pub struct Likes;

#[derive(Component)]
pub struct Apples;

#[derive(Component)]
pub struct Pears;

#[derive(Component)]
pub struct Eats;

#[derive(Component)]
pub struct SelfRef {
    pub value: Entity,
}

#[derive(Component)]
pub struct SelfRef2 {
    pub value: Entity,
}

#[derive(Debug, Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Position2 {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Component)]
pub struct PositionClone {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct PositionPair {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct MyStruct {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Component)]
pub struct Velocity {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
#[repr(C)]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Component)]
pub struct Other {
    pub value: i32,
}

#[derive(Component)]
pub struct Other2 {
    pub value: i32,
}

#[derive(Component)]
pub struct Mass {
    pub value: i32,
}

#[derive(Component)]
pub struct TypeA {
    pub value: i32,
}

#[derive(Component)]
pub struct Prefab {}

#[derive(Component)]
pub struct Obj {}

#[derive(Component)]
pub struct Obj2 {}

#[derive(Component)]
pub struct Rel {}

#[derive(Component)]
pub struct Rel2 {}

#[derive(Component)]
pub struct RelFoo {
    pub foo: u32,
}

#[derive(Component)]
pub struct Alice {}

#[derive(Component)]
pub struct Bob {}

#[derive(Component)]
pub struct Tag {}

#[derive(Component)]
pub struct TagA {}

#[derive(Component)]
pub struct TagB {}

#[derive(Component)]
pub struct TagC {}

#[derive(Component)]
pub struct TagD {}

#[derive(Component)]
pub struct TagE {}

#[derive(Component)]
pub struct TagF {}

#[derive(Component)]
pub struct TagG {}

#[derive(Component)]
pub struct TagH {}

#[derive(Component)]
pub struct TagI {}

#[derive(Component)]
pub struct TagJ {}

#[derive(Component)]
pub struct TagK {}

#[derive(Component)]
pub struct TagL {}

#[derive(Component)]
pub struct TagM {}

#[derive(Component)]
pub struct TagN {}

#[derive(Component)]
pub struct TagO {}

#[derive(Component)]
pub struct TagP {}

#[derive(Component)]
pub struct TagQ {}

#[derive(Component)]
pub struct TagR {}

#[derive(Component)]
pub struct TagS {}

#[derive(Component)]
pub struct TagT {}

#[derive(Component)]
pub struct TagV {}

#[derive(Component)]
pub struct TagX {}

#[derive(Component)]
pub struct TagClone {}

#[derive(Component)]
pub struct Parent;

#[derive(Component)]
pub struct EntityType;

#[derive(Component)]
pub struct Base;
#[derive(Component)]
pub struct Head;

#[derive(Component)]
pub struct Turret;

#[derive(Component)]
pub struct Beam;
#[derive(Component)]
pub struct Railgun;

#[derive(Component)]
pub struct Foo;

#[derive(Component)]
pub struct Bar;

#[derive(Component)]
pub struct First;

#[derive(Component)]
pub struct Pod {
    pub value: i32,
    pub clone_count: u32,
    pub drop_count: u32,
    pub ctor_count: u32,
}

impl Default for Pod {
    fn default() -> Self {
        Pod {
            value: 0,
            clone_count: 0,
            drop_count: 0,
            ctor_count: 1,
        }
    }
}

impl Pod {
    pub fn new(value: i32) -> Self {
        Pod {
            value,
            clone_count: 0,
            drop_count: 0,
            ctor_count: 1,
        }
    }
}

impl Clone for Pod {
    fn clone(&self) -> Self {
        Pod {
            value: self.value,
            clone_count: self.clone_count + 1,
            drop_count: 0,
            ctor_count: 0,
        }
    }
}

impl Drop for Pod {
    fn drop(&mut self) {
        self.drop_count += 1;
    }
}

#[derive(Component)]
#[register(Position, Velocity, u32)]
pub struct Template<T> {
    pub value: T,
}

#[derive(Component, Default)]
pub struct Templatex {
    pub value: String,
}

// this component pre-registration is required for the case of if the test cases run in single world application mode (feature flag).
// since tests run in multi world mode.
pub fn create_world() -> World {
    let world = World::new();
    let world_ref = world.get_world();

    register_component_multi_world_application::<QueryWrapper>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Likes>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Apples>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Pears>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Eats>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<SelfRef>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<SelfRef2>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Position>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Position2>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<PositionClone>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<PositionPair>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<MyStruct>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Velocity>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Color>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Other>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Other2>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Mass>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TypeA>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Prefab>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Obj>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Obj2>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Rel>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Rel2>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<RelFoo>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Alice>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Bob>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Tag>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagA>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagB>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagC>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagD>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagE>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagF>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagG>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagH>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagI>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagJ>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagK>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagL>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagM>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagN>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagO>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagP>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagQ>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagR>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagS>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagT>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagV>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagX>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<TagClone>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Parent>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<EntityType>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Base>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Head>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Turret>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Beam>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Railgun>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Foo>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Bar>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<First>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Pod>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Template<u32>>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Template<Position>>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Template<Velocity>>(&world_ref, std::ptr::null());
    register_component_multi_world_application::<Templatex>(&world_ref, std::ptr::null());

    world
}
