use super::c_binding::bindings::{ecs_cpp_component_validate, ecs_get_scope};
use super::utility::functions::get_full_type_name;
use super::{c_types::*, component_registration::*, entity::Entity};
use flecs_ecs_bridge_derive::Component;
use std::ffi::c_char;
use std::os::raw::c_void;
use std::sync::OnceLock;

use std::{marker::PhantomData, ops::Deref};

pub struct UntypedComponent {
    pub entity: Entity,
}

impl Deref for UntypedComponent {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl UntypedComponent {
    pub fn new(world: *mut WorldT, id: IdT) -> Self {
        UntypedComponent {
            entity: Entity::new_from_existing(world, id),
        }
    }
}

#[cfg(feature = "flecs_meta")]
impl UntypedComponent {}

#[cfg(feature = "flecs_metrics")]
impl UntypedComponent {}

struct Component<T: CachedComponentData> {
    pub base: UntypedComponent,
    _marker: PhantomData<T>,
}

impl<T: CachedComponentData> Component<T> {
    pub fn new(world: *mut WorldT, id: IdT, name: Option<&str>, allow_tag: bool) -> Self {
        let mut implicit_name = false;
        let name = if let Some(name) = name {
            name
        } else {
            // Keep track of whether name was explicitly set. If not, and the
            // component was already registered, just use the registered name.
            //
            // The registered name may differ from the typename as the registered
            // name includes the flecs scope. This can in theory be different from
            // the Rust namespace though it is good practice to keep them the same
            implicit_name = true;
            get_full_type_name::<T>()
        };

        if T::is_registered_with_world(world) {
            return Self {
                base: UntypedComponent::new(world, unsafe { T::get_id_unchecked() }),
                _marker: PhantomData,
            };
        } else {
            T::register_explicit(world);
        }

        Component {
            base: UntypedComponent::new(world, id),
            _marker: PhantomData,
        }
    }
}

impl<T: CachedComponentData> Deref for Component<T> {
    type Target = UntypedComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

type EcsCtxFreeT = extern "C" fn(*mut std::ffi::c_void);

struct ComponentBindingCtx {
    on_add: Option<*mut c_void>,
    on_remove: Option<*mut c_void>,
    on_set: Option<*mut c_void>,
    free_on_add: Option<EcsCtxFreeT>,
    free_on_remove: Option<EcsCtxFreeT>,
    free_on_set: Option<EcsCtxFreeT>,
}

impl Drop for ComponentBindingCtx {
    fn drop(&mut self) {
        if let Some(on_add) = self.on_add {
            if let Some(free_on_add) = self.free_on_add {
                free_on_add(on_add);
            }
        }
        if let Some(on_remove) = self.on_remove {
            if let Some(free_on_remove) = self.free_on_remove {
                free_on_remove(on_remove);
            }
        }
        if let Some(on_set) = self.on_set {
            if let Some(free_on_set) = self.free_on_set {
                free_on_set(on_set);
            }
        }
    }
}
impl Default for ComponentBindingCtx {
    fn default() -> Self {
        Self {
            on_add: None,
            on_remove: None,
            on_set: None,
            free_on_add: None,
            free_on_remove: None,
            free_on_set: None,
        }
    }
}
impl ComponentBindingCtx {
    pub fn new(
        on_add: Option<*mut std::ffi::c_void>,
        on_remove: Option<*mut std::ffi::c_void>,
        on_set: Option<*mut std::ffi::c_void>,
        free_on_add: Option<EcsCtxFreeT>,
        free_on_remove: Option<EcsCtxFreeT>,
        free_on_set: Option<EcsCtxFreeT>,
    ) -> Self {
        Self {
            on_add,
            on_remove,
            on_set,
            free_on_add,
            free_on_remove,
            free_on_set,
        }
    }
}
