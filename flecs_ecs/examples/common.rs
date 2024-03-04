#![allow(dead_code)]

pub use flecs_ecs::core::*;
pub use flecs_ecs_derive::Component;

#[allow(unused_imports)]
pub use std::ffi::CStr;

#[derive(Debug, Default, Clone, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Default, Clone, Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Default, Clone, Component)]
struct Mass {
    pub value: f32,
}

#[derive(Default, Clone, Component)]
pub struct Eats;
#[derive(Default, Clone, Component)]
pub struct Apples;

#[derive(Default, Clone, Component)]
pub struct Walking;

#[derive(Default, Clone, Component)]
pub struct Tag;

#[derive(Default, Clone, Component)]
pub struct Human;

#[derive(Default, Clone, Component)]
pub struct Attack {
    pub value: f32,
}

#[derive(Default, Clone, Component)]
pub struct Defence {
    pub value: f32,
}

#[derive(Default, Clone, Component)]
pub struct FreightCapacity {
    pub value: f32,
}

#[derive(Default, Clone, Component)]
pub struct ImpulseSpeed {
    pub value: f32,
}

#[derive(Default, Clone, Component)]
pub struct HasFlt;

#[allow(dead_code)]
fn main() {
    //this file is for common structs and functions used in the examples
}
