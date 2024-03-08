use flecs_ecs::{core::component_registration::*, macros::Component};

#[cfg(test)]
#[ctor::ctor]
fn init() {
    unsafe {
        flecs_ecs::sys::ecs_os_init();
    }
}

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
