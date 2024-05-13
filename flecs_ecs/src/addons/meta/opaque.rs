use std::ffi::{c_char, c_void};

use crate::core::*;
use crate::sys::*;

type AssignBoolFn<T> = extern "C" fn(*mut T, bool);
type AssignCharFn<T> = extern "C" fn(*mut T, i8);
type AssignIntFn<T> = extern "C" fn(*mut T, i64);
type AssignUIntFn<T> = extern "C" fn(*mut T, u64);
type AssignFloatFn<T> = extern "C" fn(*mut T, f32);
//todo!("replace with idiomatic rust equivalent of c_char. might need changes to flecs")
type AssignStringFn<T> = extern "C" fn(*mut T, *const c_char);
type AssignEntityFn<T> = extern "C" fn(*mut T, *mut WorldT, EntityT);
type AssignNullFn<T> = extern "C" fn(*mut T);
type ClearFn<T> = extern "C" fn(*mut T);
//todo!("still have to do ensure_element function for collections")
type EnsureMemberFn<T> = extern "C" fn(*mut T, *const c_char) -> *mut c_void;
type CountFn<T> = extern "C" fn(*mut T) -> usize;
type ResizeFn<T> = extern "C" fn(*mut T, usize);
/// Serializer object, used for serializing opaque types
type Serializer = ecs_serializer_t;

/// Serializer function, used to serialize opaque types
type SerializeT = ecs_meta_serialize_t;

/// Type safe variant of serializer function
type SerializeFn<T> = extern "C" fn(*const Serializer, *const T) -> i32;

pub struct Opaque<'a, T>
where
    T: ComponentId,
{
    world: WorldRef<'a>,
    pub desc: ecs_opaque_desc_t,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T> Opaque<'a, T>
where
    T: ComponentId,
{
    pub fn new(world: impl IntoWorld<'a>) -> Self {
        Self {
            world: world.world(),
            desc: ecs_opaque_desc_t {
                entity: T::get_id(world),
                type_: Default::default(),
            },
            phantom: std::marker::PhantomData,
        }
    }

    pub fn as_type(&mut self, func: impl IntoId) -> &mut Self {
        self.desc.type_.as_type = *func.into();
        self
    }

    pub fn serialize(&mut self, func: SerializeFn<T>) -> &mut Self {
        self.desc.type_.serialize = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(*const flecs_ecs_sys::ecs_serializer_t, *const T) -> i32,
                unsafe extern "C" fn(
                    *const flecs_ecs_sys::ecs_serializer_t,
                    *const std::ffi::c_void,
                ) -> i32,
            >(func)
        });
        self
    }

    pub fn assign_bool(&mut self, func: AssignBoolFn<T>) -> &mut Self {
        self.desc.type_.assign_bool = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(*mut T, bool),
                unsafe extern "C" fn(*mut std::ffi::c_void, bool),
            >(func)
        });
        self
    }

    pub fn assign_char(&mut self, func: AssignCharFn<T>) -> &mut Self {
        self.desc.type_.assign_char = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(*mut T, i8),
                unsafe extern "C" fn(*mut std::ffi::c_void, i8),
            >(func)
        });
        self
    }

    pub fn assign_int(&mut self, func: AssignIntFn<T>) -> &mut Self {
        self.desc.type_.assign_int = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(*mut T, i64),
                unsafe extern "C" fn(*mut std::ffi::c_void, i64),
            >(func)
        });
        self
    }

    pub fn assign_uint(&mut self, func: AssignUIntFn<T>) -> &mut Self {
        self.desc.type_.assign_uint = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(*mut T, u64),
                unsafe extern "C" fn(*mut std::ffi::c_void, u64),
            >(func)
        });
        self
    }

    pub fn assign_float(&mut self, func: AssignFloatFn<T>) -> &mut Self {
        self.desc.type_.assign_float = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(*mut T, f32),
                unsafe extern "C" fn(*mut std::ffi::c_void, f64),
            >(func)
        });
        self
    }

    pub fn assign_string(&mut self, func: AssignStringFn<T>) -> &mut Self {
        self.desc.type_.assign_string = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(*mut T, *const i8),
                unsafe extern "C" fn(*mut std::ffi::c_void, *const i8),
            >(func)
        });
        self
    }

    pub fn assign_entity(&mut self, func: AssignEntityFn<T>) -> &mut Self {
        self.desc.type_.assign_entity = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(*mut T, *mut flecs_ecs_sys::ecs_world_t, u64),
                unsafe extern "C" fn(*mut std::ffi::c_void, *mut flecs_ecs_sys::ecs_world_t, u64),
            >(func)
        });
        self
    }

    pub fn assign_null(&mut self, func: AssignNullFn<T>) -> &mut Self {
        self.desc.type_.assign_null = Some(unsafe {
            std::mem::transmute::<extern "C" fn(*mut T), unsafe extern "C" fn(*mut std::ffi::c_void)>(
                func,
            )
        });
        self
    }

    pub fn clear(&mut self, func: ClearFn<T>) -> &mut Self {
        self.desc.type_.clear = Some(unsafe {
            std::mem::transmute::<extern "C" fn(*mut T), unsafe extern "C" fn(*mut std::ffi::c_void)>(
                func,
            )
        });
        self
    }

    pub fn ensure_member(&mut self, func: EnsureMemberFn<T>) -> &mut Self {
        self.desc.type_.ensure_member = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(*mut T, *const i8) -> *mut std::ffi::c_void,
                unsafe extern "C" fn(*mut std::ffi::c_void, *const i8) -> *mut std::ffi::c_void,
            >(func)
        });
        self
    }

    pub fn count(&mut self, func: CountFn<T>) -> &mut Self {
        self.desc.type_.count = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(*mut T) -> usize,
                unsafe extern "C" fn(*const std::ffi::c_void) -> usize,
            >(func)
        });
        self
    }

    pub fn resize(&mut self, func: ResizeFn<T>) -> &mut Self {
        self.desc.type_.resize = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(*mut T, usize),
                unsafe extern "C" fn(*mut std::ffi::c_void, usize),
            >(func)
        });
        self
    }
}

impl<'a, T> Drop for Opaque<'a, T>
where
    T: ComponentId,
{
    fn drop(&mut self) {
        unsafe {
            ecs_opaque_init(self.world.world_ptr_mut(), &self.desc);
        }
    }
}
