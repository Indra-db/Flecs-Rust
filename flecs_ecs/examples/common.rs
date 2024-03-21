#![allow(dead_code)]

pub use flecs_ecs::{core::*, macros::Component};

#[derive(Debug, Default, Clone, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Default, Debug, Clone, Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Default, Clone, Component)]
pub struct Mass {
    pub value: f32,
}

#[derive(Default, Clone, Component)]
pub struct Eats;

#[derive(Default, Clone, Component)]
pub struct Apples;

#[derive(Default, Clone, Component)]
pub struct Pears;

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
pub struct Damage {
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

#[derive(Default, Clone, Component)]
pub struct First;

#[derive(Default, Clone, Component)]
pub struct Second;

#[derive(Default, Clone, Component)]
pub struct Third;

#[derive(Default, Clone, Component)]
pub struct Group;

#[allow(dead_code)]
fn main() {
    //this file is for common structs and functions used in the examples
}
