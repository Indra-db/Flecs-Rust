use flecs_ecs_bridge::core::c_types::*;
use flecs_ecs_bridge::core::component::*;
use flecs_ecs_bridge_derive::Component;
use std::sync::OnceLock;

#[derive(Clone, Debug, Component, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Component, Default)]
pub struct Velocity {
    pub x: f64,
    pub y: f64,
}
