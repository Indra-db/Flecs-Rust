use crate::core::*;
use crate::sys::*;

use super::meta_functions::*;
use super::{
    AssignBoolFnPtr, AssignBoolFnPtrUnsafe, AssignCharFnPtr, AssignCharFnPtrUnsafe,
    AssignEntityFnPtr, AssignEntityFnPtrUnsafe, AssignFloatFnPtr, AssignFloatFnPtrUnsafe,
    AssignIntFnPtr, AssignIntFnPtrUnsafe, AssignNullFnPtr, AssignNullFnPtrUnsafe,
    AssignStringFnPtr, AssignStringFnPtrUnsafe, AssignUIntFnPtr, AssignUIntFnPtrUnsafe, ClearFnPtr,
    ClearFnPtrUnsafe, CountFnPtr, CountFnPtrUnsafe, EnsureElementFnPtr, EnsureElementFnPtrUnsafe,
    EnsureMemberFnPtr, EnsureMemberFnPtrUnsafe, ResizeFnPtr, ResizeFnPtrUnsafe,
    SerializeElementFnPtr, SerializeElementFnPtrUnsafe, SerializeFnPtr, SerializeFnPtrUnsafe,
    SerializeMemberFnPtr, SerializeMemberFnPtrUnsafe,
};

/// Serializer object, used for serializing opaque types
pub type Serializer = ecs_serializer_t;

/// Serializer function, used to serialize opaque types
pub type SerializeT = ecs_meta_serialize_t;

/// Type safe interface for opaque types
pub struct Opaque<'a, T: 'static, ElemType = ()> {
    world: WorldRef<'a>,
    pub desc: ecs_opaque_desc_t,
    phantom: core::marker::PhantomData<T>,
    phantom2: core::marker::PhantomData<ElemType>,
}

impl<'a, T, ElemType> Opaque<'a, T, ElemType> {
    /// Creates a new Opaque instance
    pub fn new(world: impl WorldProvider<'a>) -> Self {
        let id = *world
            .world()
            .components_map()
            .get(&core::any::TypeId::of::<T>())
            .unwrap_or_else(|| panic!("Component with name: {} is not registered, pre-register components with `world.component::<T>() or world.component_ext::<T>(id)`", core::any::type_name::<T>()));

        Self {
            world: world.world(),
            desc: ecs_opaque_desc_t {
                entity: id,
                type_: Default::default(),
            },
            phantom: core::marker::PhantomData,
            phantom2: core::marker::PhantomData,
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
            phantom: core::marker::PhantomData,
            phantom2: core::marker::PhantomData,
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
            core::mem::transmute::<SerializeFnPtr<T>, SerializeFnPtrUnsafe>(func.to_extern_fn())
        });
        self
    }

    /// Serialize member function
    pub fn serialize_member(&mut self, func: impl SerializeMember<T>) -> &mut Self {
        self.desc.type_.serialize_member = Some(unsafe {
            core::mem::transmute::<SerializeMemberFnPtr<T>, SerializeMemberFnPtrUnsafe>(
                func.to_extern_fn(),
            )
        });
        self
    }

    /// Serialize element function
    pub fn serialize_element(&mut self, func: impl SerializeElement<T>) -> &mut Self {
        self.desc.type_.serialize_element = Some(unsafe {
            core::mem::transmute::<SerializeElementFnPtr<T>, SerializeElementFnPtrUnsafe>(
                func.to_extern_fn(),
            )
        });
        self
    }

    /// Assign bool value
    pub fn assign_bool(&mut self, func: impl AssignBoolFn<T>) -> &mut Self {
        self.desc.type_.assign_bool = Some(unsafe {
            core::mem::transmute::<AssignBoolFnPtr<T>, AssignBoolFnPtrUnsafe>(func.to_extern_fn())
        });
        self
    }

    /// Assign char value
    pub fn assign_char(&mut self, func: impl AssignCharFn<T>) -> &mut Self {
        self.desc.type_.assign_char = Some(unsafe {
            core::mem::transmute::<AssignCharFnPtr<T>, AssignCharFnPtrUnsafe>(func.to_extern_fn())
        });
        self
    }

    /// Assign int value
    pub fn assign_int(&mut self, func: impl AssignIntFn<T>) -> &mut Self {
        self.desc.type_.assign_int = Some(unsafe {
            core::mem::transmute::<AssignIntFnPtr<T>, AssignIntFnPtrUnsafe>(func.to_extern_fn())
        });
        self
    }

    /// Assign unsigned int value
    pub fn assign_uint(&mut self, func: impl AssignUIntFn<T>) -> &mut Self {
        self.desc.type_.assign_uint = Some(unsafe {
            core::mem::transmute::<AssignUIntFnPtr<T>, AssignUIntFnPtrUnsafe>(func.to_extern_fn())
        });
        self
    }

    /// Assign float value
    pub fn assign_float(&mut self, func: impl AssignFloatFn<T>) -> &mut Self {
        self.desc.type_.assign_float = Some(unsafe {
            core::mem::transmute::<AssignFloatFnPtr<T>, AssignFloatFnPtrUnsafe>(func.to_extern_fn())
        });
        self
    }

    /// Assign string value
    pub fn assign_string(&mut self, func: impl AssignStringFn<T>) -> &mut Self {
        self.desc.type_.assign_string = Some(unsafe {
            core::mem::transmute::<AssignStringFnPtr<T>, AssignStringFnPtrUnsafe>(
                func.to_extern_fn(),
            )
        });
        self
    }

    /// Assign entity value
    pub fn assign_entity(&mut self, func: impl AssignEntityFn<'a, T>) -> &mut Self {
        self.desc.type_.assign_entity = Some(unsafe {
            core::mem::transmute::<AssignEntityFnPtr<'a, T>, AssignEntityFnPtrUnsafe>(
                func.to_extern_fn(),
            )
        });
        self
    }

    /// Assign null value
    pub fn assign_null(&mut self, func: impl AssignNullFn<T>) -> &mut Self {
        self.desc.type_.assign_null = Some(unsafe {
            core::mem::transmute::<AssignNullFnPtr<T>, AssignNullFnPtrUnsafe>(func.to_extern_fn())
        });
        self
    }

    /// Clear collection elements
    pub fn clear(&mut self, func: impl ClearFn<T>) -> &mut Self {
        self.desc.type_.clear = Some(unsafe {
            core::mem::transmute::<ClearFnPtr<T>, ClearFnPtrUnsafe>(func.to_extern_fn())
        });
        self
    }

    /// Ensure & get element
    pub fn ensure_element(&mut self, func: impl EnsureElementFn<T, ElemType>) -> &mut Self {
        self.desc.type_.ensure_element = Some(unsafe {
            core::mem::transmute::<EnsureElementFnPtr<T, ElemType>, EnsureElementFnPtrUnsafe>(
                func.to_extern_fn(),
            )
        });
        self
    }

    /// Ensure & get element
    pub fn ensure_member(&mut self, func: impl EnsureMemberFn<T>) -> &mut Self {
        self.desc.type_.ensure_member = Some(unsafe {
            core::mem::transmute::<EnsureMemberFnPtr<T>, EnsureMemberFnPtrUnsafe>(
                func.to_extern_fn(),
            )
        });
        self
    }

    /// Return number of elements
    pub fn count(&mut self, func: impl CountFn<T>) -> &mut Self {
        self.desc.type_.count = Some(unsafe {
            core::mem::transmute::<CountFnPtr<T>, CountFnPtrUnsafe>(func.to_extern_fn())
        });
        self
    }

    /// Resize to number of elements
    pub fn resize(&mut self, func: impl ResizeFn<T>) -> &mut Self {
        self.desc.type_.resize = Some(unsafe {
            core::mem::transmute::<ResizeFnPtr<T>, ResizeFnPtrUnsafe>(func.to_extern_fn())
        });
        self
    }
}

impl<T, ElemType> Drop for Opaque<'_, T, ElemType> {
    /// Finalizes the opaque type descriptor when it is dropped
    fn drop(&mut self) {
        unsafe {
            ecs_opaque_init(self.world.world_ptr_mut(), &self.desc);
        }
    }
}
