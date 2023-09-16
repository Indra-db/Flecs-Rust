use std::{
    ffi::{c_void, CStr, CString},
    sync::OnceLock,
};

use libc::strlen;

use crate::core::c_binding::bindings::ecs_get_world;

use super::{
    c_binding::bindings::{
        ecs_get_name, ecs_get_path_w_sep, ecs_get_symbol, ecs_get_type, ecs_has_id, ecs_is_alive,
        ecs_is_valid, EcsDisabled,
    },
    c_types::*,
    component::CachedComponentData,
    flecs_type::Type,
    id::Id,
};

static SEPARATOR: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"::\0") };

#[derive(Default)]
pub struct Entity {
    pub id: Id,
}

impl Entity {
    /// Wrap an existing entity id.
    /// # Arguments
    /// * `world` - The world the entity belongs to.
    /// * `id` - The entity id.
    pub fn new(world: *mut WorldT, id: EntityT) -> Self {
        unsafe {
            Self {
                id: Id::new(
                    if world.is_null() {
                        std::ptr::null_mut()
                    } else {
                        ecs_get_world(world as *mut c_void) as *mut WorldT
                    },
                    id,
                ),
            }
        }
    }

    // Explicit conversion from flecs::entity_t to Entity
    pub const fn new_only_id(id: EntityT) -> Self {
        Self {
            id: Id::new_only_id(id),
        }
    }

    /// checks if entity is valid
    pub fn get_is_valid(&self) -> bool {
        self.id.world != std::ptr::null_mut() && unsafe { ecs_is_valid(self.id.world, self.id.id) }
    }

    /// Checks if entity is alive.
    pub fn get_is_alive(&self) -> bool {
        self.id.world != std::ptr::null_mut() && unsafe { ecs_is_alive(self.id.world, self.id.id) }
    }

    /// Returns the entity name.
    pub fn get_name(&self) -> &'static str {
        unsafe {
            CStr::from_ptr(ecs_get_name(self.id.world, self.id.id))
                .to_str()
                .unwrap_or("")
        }
    }

    //TODO check if we need this -> can we use get_symbol from CachedComponentData?
    /// Returns the entity symbol.
    pub fn get_symbol(&self) -> &'static str {
        unsafe {
            CStr::from_ptr(ecs_get_symbol(self.id.world, self.id.id))
                .to_str()
                .unwrap_or("")
        }
    }

    /// Return the hierarchical entity path.
    /// # Note
    /// if you're using the default separator "::" you can use get_hierachy_path_default
    /// which does no extra heap allocations to communicate with C
    pub fn get_hierachy_path(&self, sep: &str, init_sep: &str) -> Option<String> {
        self.get_hierachy_path_from_parent_id(0, sep, init_sep)
    }

    /// Return the hierarchical entity path using the default separator "::".
    pub fn get_hierachy_path_default(&self) -> Option<String> {
        self.get_hierachy_path_from_parent_id_default(0)
    }

    /// Return the hierarchical entity path relative to a parent.
    /// # Note
    /// if you're using the default separator "::" you can use get_hierachy_path_default
    /// which does no extra heap allocations to communicate with C
    pub fn get_hierachy_path_from_parent_id(
        &self,
        parent: EntityT,
        sep: &str,
        init_sep: &str,
    ) -> Option<String> {
        let c_sep = CString::new(sep).unwrap();
        let raw_ptr = if sep == init_sep {
            unsafe {
                ecs_get_path_w_sep(
                    self.id.world,
                    parent,
                    self.id.id,
                    c_sep.as_ptr(),
                    c_sep.as_ptr(),
                )
            }
        } else {
            unsafe {
                ecs_get_path_w_sep(
                    self.id.world,
                    parent,
                    self.id.id,
                    c_sep.as_ptr(),
                    CString::new(init_sep).unwrap().as_ptr(),
                )
            }
        };

        if raw_ptr.is_null() {
            return None;
        }

        let len = unsafe { strlen(raw_ptr) } as usize;

        // Convert the C string to a Rust String without any new heap allocation.
        // The String will de-allocate the C string when it goes out of scope.
        Some(unsafe {
            String::from_utf8_unchecked(Vec::from_raw_parts(raw_ptr as *mut u8, len, len))
        })
    }

    /// Return the hierarchical entity path relative to a parent id using the default separator "::".
    pub fn get_hierachy_path_from_parent_id_default(&self, parent: EntityT) -> Option<String> {
        unsafe {
            let raw_ptr = ecs_get_path_w_sep(
                self.id.world,
                parent,
                self.id.id,
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
            );

            if raw_ptr.is_null() {
                return None;
            }

            let len = strlen(raw_ptr) as usize;

            // Convert the C string to a Rust String without any new heap allocation.
            // The String will de-allocate the C string when it goes out of scope.
            Some(String::from_utf8_unchecked(Vec::from_raw_parts(
                raw_ptr as *mut u8,
                len,
                len,
            )))
        }
    }

    /// Return the hierarchical entity path relative to a parent type.
    /// # Note
    /// if you're using the default separator "::" you can use get_hierachy_path_default
    /// which does no extra heap allocations to communicate with C
    pub fn get_hierachy_path_from_parent_type<T: CachedComponentData>(
        &self,
        sep: &str,
        init_sep: &str,
    ) -> Option<String> {
        self.get_hierachy_path_from_parent_id(T::get_id(self.id.world), sep, init_sep)
    }

    /// Return the hierarchical entity path relative to a parent type using the default separator "::".
    pub fn get_hierachy_path_from_parent_type_default<T: CachedComponentData>(
        &self,
    ) -> Option<String> {
        self.get_hierachy_path_from_parent_id_default(T::get_id(self.id.world))
    }

    pub fn get_is_enabled(&self) -> bool {
        unsafe { !ecs_has_id(self.id.world, self.id.id, EcsDisabled) }
    }

    pub fn get_entity_type(&self) -> Type {
        Type::new(self.id.world, unsafe {
            ecs_get_type(self.id.world, self.id.id)
        })
    }
}
