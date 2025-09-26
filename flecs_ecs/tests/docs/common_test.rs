#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unexpected_cfgs)]

pub use core::ffi::c_void;
pub use flecs_ecs::prelude::*;
pub use flecs_ecs_sys as sys;

#[derive(Debug, Clone, Default, Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Default, Component)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Default, Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Default, Debug)]
pub struct Gravity {
    pub value: f32,
}

#[derive(Component, Default, Debug)]
pub struct Mesh {
    pub value: f32,
}

#[derive(Component, Default, Debug, Clone, PartialEq)]
pub struct Mass {
    pub value: f32,
}

#[derive(Component, Default, Debug, Clone, PartialEq)]
pub struct MaxSpeed {
    pub value: u32,
}

#[derive(Component, Default, Debug, Clone, PartialEq)]
pub struct Defense {
    pub value: u32,
}

#[derive(Component, Default, Debug, Clone, PartialEq)]
pub struct Game {
    pub time: u32,
}

#[derive(Component, Default, Debug, Clone, PartialEq)]
pub struct SimTime {
    pub value: f32,
}

#[derive(Component, Default, Debug, Clone, PartialEq)]
pub struct Depth {
    pub value: u32,
}

#[derive(Component, Default, Debug, Clone, PartialEq)]
pub struct SimConfig {
    pub sim_speed: f32,
}

#[derive(Component, Default, Debug, Clone, PartialEq)]
pub struct Health {
    pub value: u32,
}

#[derive(Component, Default, Debug, Clone, PartialEq)]
pub struct Plate {
    pub contents: u32,
}

#[derive(Component, Default, Clone, Debug)]
pub struct TimeOfDay(pub f32);

#[derive(Component)]
pub struct Foo;

#[derive(Component)]
pub struct Bar;

#[derive(Component)]
pub struct Speed;

#[derive(Component)]
pub struct LocatedIn;

#[derive(Component)]
pub struct SpaceShip;

#[derive(Component)]
pub struct DockedTo;

#[derive(Component)]
pub struct Planet;

#[derive(Component)]
pub struct Serializable;

#[derive(Component)]
pub struct Likes;

#[derive(Component)]
pub struct Apples;

#[derive(Component)]
pub struct Pears;

#[derive(Component)]
pub struct Eats;

#[derive(Component)]
pub struct Unit;

#[derive(Component)]
pub struct Warrior;

#[derive(Component)]
pub struct Archer;

#[derive(Component)]
pub struct Node;
#[derive(Component)]
pub struct Npc;
