use rand::random;
use std::os::raw::c_char;
use std::sync::OnceLock;

use crate::core::c_binding::bindings::ecs_set_scope;

use super::{
    c_binding::bindings::ecs_set_with,
    c_types::{EntityT, WorldT},
};

#[derive(Debug)]
pub struct ComponentDescriptor {
    pub symbol: String,
    pub name: String,
    pub custom_id: Option<u64>,
    pub layout: std::alloc::Layout,
}

// Dummy function to simulate ID generation
fn generate_id() -> u64 {
    random()
}

//dummy function to simulate data generation
fn register_component_data(
    world: *mut WorldT,
    name: *const c_char,
    allow_tag: bool,
) -> ComponentData {
    let mut prev_scope: EntityT = 0;
    let mut prev_with: EntityT = 0;

    if !world.is_null() {
        prev_scope = unsafe { ecs_set_scope(world, 0) };
        prev_with = unsafe { ecs_set_with(world, 0) };
    }

    if prev_with != 0 {
        unsafe { ecs_set_with(world, prev_with) };
    }
    if prev_scope != 0 {
        unsafe { ecs_set_scope(world, prev_scope) };
    }
    ComponentData {
        id: generate_id(),
        size: 0,
        alignment: 0,
        allow_tags: false,
    }
}

pub struct ComponentData {
    pub id: u64,
    pub size: usize,
    pub alignment: usize,
    pub allow_tags: bool,
}

pub trait CachedComponentData {
    fn get_data() -> &'static ComponentData;

    fn get_id() -> u64 {
        Self::get_data().id
    }

    fn get_size() -> usize {
        Self::get_data().size
    }

    fn get_alignment() -> usize {
        Self::get_data().alignment
    }

    fn get_allow_tags() -> bool {
        Self::get_data().allow_tags
    }
}

macro_rules! impl_cached_component_data  {
    ($($t:ty),*) => {
        $(
            impl CachedComponentData for $t {
                fn get_data() -> &'static ComponentData {
                    static ONCE_LOCK : OnceLock<ComponentData> = OnceLock::new();
                    ONCE_LOCK.get_or_init(|| register_component_data())
                }
            }
        )*
    };
}
