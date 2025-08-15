#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unexpected_cfgs)]

use core::ops::{Deref, DerefMut};
use std::collections::HashMap;

pub use flecs_ecs::prelude::*;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    unsafe {
        flecs_ecs::sys::ecs_os_init();
    }

    // Use the crash handler for integration tests
    #[cfg(feature = "test-with-crash-handler")]
    test_crash_handler::register();
}
#[derive(Debug, Component, Clone, Copy)]
pub struct Count(pub i32);

impl Deref for Count {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<i32> for &mut Count {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
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
pub struct EntityRef {
    pub value: Entity,
}

#[derive(Component)]
pub struct SelfRef2 {
    pub value: Entity,
}

#[derive(Component)]
pub struct Value {
    pub value: i32,
}

#[derive(Debug, Component, Default, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Component, Default)]
#[meta]
pub struct Point {
    x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }
}

#[derive(Debug, Component, Default, Clone)]
pub struct Velocity {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct MyStruct {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Other {
    pub value: i32,
}

#[derive(Component, Default)]
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

#[derive(Component, Default)]
pub struct RelFoo {
    pub foo: u32,
}

#[derive(Component)]
pub struct Alice {}

#[derive(Component)]
pub struct Bob {}

#[derive(Component)]
pub struct Tag;

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

#[derive(Component, Clone, Copy)]
pub struct Count2 {
    pub a: i32,
    pub b: i32,
}

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
    #[allow(dead_code)]
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
pub struct Template<T: Send + Sync + 'static> {
    pub value: T,
}

#[derive(Component, Default)]
pub struct Templatex {
    pub value: String,
}

pub fn create_world_with_flags<T: ComponentId + Default + DataComponent + ComponentType<Struct>>()
-> World {
    let world = World::new();

    internal_register_component::<false, false, T>(&world, core::ptr::null());
    world.set(T::default());

    world
}
