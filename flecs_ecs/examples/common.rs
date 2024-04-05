#![allow(dead_code)]
#![allow(unused_imports)]

pub use flecs_ecs::{core::*, macros::Component};

#[derive(Debug, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Mass {
    pub value: f32,
}

#[derive(Component)]
pub struct Eats;

#[derive(Component)]
pub struct Apples;

#[derive(Component)]
pub struct Pears;

#[derive(Component)]
pub struct Walking;

#[derive(Component)]
pub struct Tag;

#[derive(Component)]
pub struct Human;

#[derive(Component)]
pub struct Attack {
    pub value: f32,
}

#[derive(Component)]
pub struct Defence {
    pub value: f32,
}

#[derive(Component)]
pub struct Damage {
    pub value: f32,
}

#[derive(Component)]
pub struct FreightCapacity {
    pub value: f32,
}

#[derive(Component)]
pub struct ImpulseSpeed {
    pub value: f32,
}

#[derive(Component)]
pub struct HasFlt;

#[derive(Component)]
pub struct First;

#[derive(Component)]
pub struct Second;

#[derive(Component)]
pub struct Third;

#[derive(Component)]
pub struct Group;

#[allow(dead_code)]
fn main() {
    //this file is for common structs and functions used in the examples
}
