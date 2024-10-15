use crate::core::*;
use crate::sys::*;

use super::meta_functions::*;

/// Serializer object, used for serializing opaque types
pub type Serializer = ecs_serializer_t;

/// Serializer function, used to serialize opaque types
pub type SerializeT = ecs_meta_serialize_t;

/// Type safe interface for opaque types
pub struct Opaque<'a, T: 'static, ElemType = ()> {
    world: WorldRef<'a>,
    pub desc: ecs_opaque_desc_t,
    phantom: std::marker::PhantomData<T>,
    phantom2: std::marker::PhantomData<ElemType>,
}

impl<'a, T, ElemType> Opaque<'a, T, ElemType> {
    /// Creates a new Opaque instance
    pub fn new(world: impl WorldProvider<'a>) -> Self {
        let id = *world
            .world()
            .components_map()
            .get(&std::any::TypeId::of::<T>())
            .unwrap_or_else(|| panic!("Component with name: {} is not registered, pre-register components with `world.component::<T>() or world.component_ext::<T>(id)`", std::any::type_name::<T>()));

        Self {
            world: world.world(),
            desc: ecs_opaque_desc_t {
                entity: id,
                type_: Default::default(),
            },
            phantom: std::marker::PhantomData,
            phantom2: std::marker::PhantomData,
        }
    }

    /// Creates a new Opaque instance of an internal or external component
    pub fn new_id(world: impl WorldProvider<'a>, id: impl Into<Entity>) -> Self {
        Self {
            world: world.world(),
            desc: ecs_opaque_desc_t {
                entity: *id.into(),
                type_: Default::default(),
            },
            phantom: std::marker::PhantomData,
            phantom2: std::marker::PhantomData,
        }
    }

    /// Type that describes the type kind/structure of the opaque type
    pub fn as_type(&mut self, func: impl Into<Entity>) -> &mut Self {
        self.desc.type_.as_type = *func.into();
        self
    }

    /// Serialize function
    /// Fn(&Serializer, &T) -> i32
    pub fn serialize(&mut self, func: impl SerializeFn<T>) -> &mut Self {
        self.desc.type_.serialize = Some(unsafe {
            std::mem::transmute::<
                extern "C-unwind" fn(&flecs_ecs_sys::ecs_serializer_t, &T) -> i32,
                unsafe extern "C-unwind" fn(
                    *const flecs_ecs_sys::ecs_serializer_t,
                    *const std::ffi::c_void,
                ) -> i32,
            >(func.to_extern_fn())
        });
        self
    }

    /// Assign bool value
    pub fn assign_bool(&mut self, func: impl AssignBoolFn<T>) -> &mut Self {
        self.desc.type_.assign_bool = Some(unsafe {
            std::mem::transmute::<
                extern "C-unwind" fn(&mut T, bool),
                unsafe extern "C-unwind" fn(*mut std::ffi::c_void, bool),
            >(func.to_extern_fn())
        });
        self
    }

    /// Assign char value
    pub fn assign_char(&mut self, func: impl AssignCharFn<T>) -> &mut Self {
        self.desc.type_.assign_char = Some(unsafe {
            std::mem::transmute::<
                extern "C-unwind" fn(&mut T, i8),
                unsafe extern "C-unwind" fn(*mut std::ffi::c_void, i8),
            >(func.to_extern_fn())
        });
        self
    }

    /// Assign int value
    pub fn assign_int(&mut self, func: impl AssignIntFn<T>) -> &mut Self {
        self.desc.type_.assign_int = Some(unsafe {
            std::mem::transmute::<
                extern "C-unwind" fn(&mut T, i64),
                unsafe extern "C-unwind" fn(*mut std::ffi::c_void, i64),
            >(func.to_extern_fn())
        });
        self
    }

    /// Assign unsigned int value
    pub fn assign_uint(&mut self, func: impl AssignUIntFn<T>) -> &mut Self {
        self.desc.type_.assign_uint = Some(unsafe {
            std::mem::transmute::<
                extern "C-unwind" fn(&mut T, u64),
                unsafe extern "C-unwind" fn(*mut std::ffi::c_void, u64),
            >(func.to_extern_fn())
        });
        self
    }

    /// Assign float value
    pub fn assign_float(&mut self, func: impl AssignFloatFn<T>) -> &mut Self {
        self.desc.type_.assign_float = Some(unsafe {
            std::mem::transmute::<
                extern "C-unwind" fn(&mut T, f32),
                unsafe extern "C-unwind" fn(*mut std::ffi::c_void, f64),
            >(func.to_extern_fn())
        });
        self
    }

    /// Assign string value
    pub fn assign_string(&mut self, func: impl AssignStringFn<T>) -> &mut Self {
        self.desc.type_.assign_string = Some(unsafe {
            std::mem::transmute::<
                extern "C-unwind" fn(&mut T, *const i8),
                unsafe extern "C-unwind" fn(*mut std::ffi::c_void, *const i8),
            >(func.to_extern_fn())
        });
        self
    }

    /// Assign entity value
    pub fn assign_entity(&mut self, func: impl AssignEntityFn<'a, T>) -> &mut Self {
        self.desc.type_.assign_entity = Some(unsafe {
            std::mem::transmute::<
                extern "C-unwind" fn(&'a mut T, WorldRef<'a>, Entity),
                unsafe extern "C-unwind" fn(
                    *mut std::ffi::c_void,
                    *mut flecs_ecs_sys::ecs_world_t,
                    u64,
                ),
            >(func.to_extern_fn())
        });
        self
    }

    /// Assign null value
    pub fn assign_null(&mut self, func: impl AssignNullFn<T>) -> &mut Self {
        self.desc.type_.assign_null = Some(unsafe {
            std::mem::transmute::<
                extern "C-unwind" fn(&mut T),
                unsafe extern "C-unwind" fn(*mut std::ffi::c_void),
            >(func.to_extern_fn())
        });
        self
    }

    /// Clear collection elements
    pub fn clear(&mut self, func: impl ClearFn<T>) -> &mut Self {
        self.desc.type_.clear = Some(unsafe {
            std::mem::transmute::<
                extern "C-unwind" fn(&mut T),
                unsafe extern "C-unwind" fn(*mut std::ffi::c_void),
            >(func.to_extern_fn())
        });
        self
    }

    /// Ensure & get element
    pub fn ensure_element(&mut self, func: impl EnsureElementFn<T, ElemType>) -> &mut Self {
        self.desc.type_.ensure_element = Some(unsafe {
            std::mem::transmute::<
                extern "C-unwind" fn(&mut T, usize) -> &mut ElemType,
                unsafe extern "C-unwind" fn(*mut std::ffi::c_void, usize) -> *mut std::ffi::c_void,
            >(func.to_extern_fn())
        });
        self
    }

    /// Ensure & get element
    pub fn ensure_member(&mut self, func: impl EnsureMemberFn<T>) -> &mut Self {
        self.desc.type_.ensure_member = Some(unsafe {
            std::mem::transmute::<
                extern "C-unwind" fn(&mut T, *const i8) -> *mut std::ffi::c_void,
                unsafe extern "C-unwind" fn(
                    *mut std::ffi::c_void,
                    *const i8,
                ) -> *mut std::ffi::c_void,
            >(func.to_extern_fn())
        });
        self
    }

    /// Return number of elements
    pub fn count(&mut self, func: impl CountFn<T>) -> &mut Self {
        self.desc.type_.count = Some(unsafe {
            std::mem::transmute::<
                extern "C-unwind" fn(&mut T) -> usize,
                unsafe extern "C-unwind" fn(*const std::ffi::c_void) -> usize,
            >(func.to_extern_fn())
        });
        self
    }

    /// Resize to number of elements
    pub fn resize(&mut self, func: impl ResizeFn<T>) -> &mut Self {
        self.desc.type_.resize = Some(unsafe {
            std::mem::transmute::<
                extern "C-unwind" fn(&mut T, usize),
                unsafe extern "C-unwind" fn(*mut std::ffi::c_void, usize),
            >(func.to_extern_fn())
        });
        self
    }
}

impl<'a, T, ElemType> Drop for Opaque<'a, T, ElemType> {
    /// Finalizes the opaque type descriptor when it is dropped
    fn drop(&mut self) {
        unsafe {
            ecs_opaque_init(self.world.world_ptr_mut(), &self.desc);
        }
    }
}
