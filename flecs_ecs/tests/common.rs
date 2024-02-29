use flecs_ecs::core::c_types::*;
use flecs_ecs::core::component_registration::*;
use flecs_ecs_derive::Component;
use std::ffi::CStr;
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

#[derive(Clone, Debug, Component, Default)]
pub struct Mass {
    pub value: f32,
}

#[derive(Clone, Debug, Component, Default)]
pub struct TypeA {
    pub value: f32,
}

#[derive(Clone, Debug, Component, Default)]
pub struct TagA {}

#[derive(Clone, Debug, Component, Default)]
pub struct TagB {}

#[derive(Clone, Debug, Component, Default)]
pub struct TagC {}
