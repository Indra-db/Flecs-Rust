#![allow(dead_code)]
use flecs_ecs::macros::Component;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    unsafe {
        flecs_ecs::sys::ecs_os_init();
    }
}

#[derive(Clone, Debug, Component, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Component, Default)]
pub struct Velocity {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Component, Default)]
pub struct Mass {
    pub value: i32,
}

#[derive(Clone, Debug, Component, Default)]
pub struct TypeA {
    pub value: i32,
}

#[derive(Clone, Debug, Component, Default)]
pub struct TagA {}

#[derive(Clone, Debug, Component, Default)]
pub struct TagB {}

#[derive(Clone, Debug, Component, Default)]
pub struct TagC {}

#[derive(Clone, Debug, Component, Default)]
pub struct Parent;

#[derive(Clone, Debug, Component, Default)]
pub struct EntityType;

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
