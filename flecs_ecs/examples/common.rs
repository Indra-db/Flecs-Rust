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
pub struct Eats;
#[derive(Default, Clone, Component)]
pub struct Apples;

#[derive(Default, Clone, Component)]
pub struct Walking;

#[derive(Default, Clone, Component)]
pub struct Tag;

#[derive(Default, Clone, Component)]
pub struct Human;

#[allow(dead_code)]
fn main() {
    //this file is for common structs and functions used in the examples
}
