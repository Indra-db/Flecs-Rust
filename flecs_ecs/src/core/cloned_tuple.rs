#![allow(unused)]

use core::ffi::c_void;
use core::marker::PhantomData;

use crate::core::*;
use crate::sys;
use flecs_ecs_derive::tuples;
use sys::ecs_record_t;

pub struct ComponentsData<T: ClonedTuple, const LEN: usize> {
    pub array_components: [*mut c_void; LEN],
    pub has_all_components: bool,
    #[cfg(feature = "flecs_safety_locks")]
    pub(crate) read_ids: [u64; LEN],
    _marker: PhantomData<T>,
}

pub trait ClonedComponentPointers<T: ClonedTuple> {
    fn new<'a, const SHOULD_PANIC: bool>(
        world: impl WorldProvider<'a>,
        entity: Entity,
        record: *const ecs_record_t,
    ) -> Self;

    fn get_tuple<'a>(&self) -> T::TupleType<'a>;

    fn has_all_components(&self) -> bool;

    #[cfg(feature = "flecs_safety_locks")]
    fn read_ids(&self) -> &[u64];
}

impl<T: ClonedTuple, const LEN: usize> ClonedComponentPointers<T> for ComponentsData<T, LEN> {
    fn new<'a, const SHOULD_PANIC: bool>(
        world: impl WorldProvider<'a>,
        entity: Entity,
        record: *const ecs_record_t,
    ) -> Self {
        let mut array_components = [core::ptr::null::<c_void>() as *mut c_void; LEN];

        #[cfg(feature = "flecs_safety_locks")]
        let mut read_ids = [0; LEN];

        let has_all_components = T::populate_array_ptrs::<SHOULD_PANIC>(
            world,
            entity,
            record,
            &mut array_components[..],
            #[cfg(feature = "flecs_safety_locks")]
            &mut read_ids,
        );

        Self {
            array_components,
            has_all_components,
            #[cfg(feature = "flecs_safety_locks")]
            read_ids,
            _marker: PhantomData::<T>,
        }
    }

    fn get_tuple<'a>(&self) -> T::TupleType<'a> {
        T::create_tuple(&self.array_components[..])
    }

    fn has_all_components(&self) -> bool {
        self.has_all_components
    }

    #[cfg(feature = "flecs_safety_locks")]
    fn read_ids(&self) -> &[u64] {
        &self.read_ids
    }
}

pub trait ClonedTupleTypeOperation {
    type ActualType: Clone;
    type OnlyType: ComponentOrPairId;
    const IS_OPTION: bool;

    fn create_tuple_data(array_components_data: *mut c_void) -> Self::ActualType;
}

impl<T> ClonedTupleTypeOperation for &T
where
    T: ComponentOrPairId,
    <T as ComponentOrPairId>::CastType: Clone,
{
    type ActualType = <T as ComponentOrPairId>::CastType;
    type OnlyType = T;
    const IS_OPTION: bool = false;

    fn create_tuple_data(array_components_data: *mut c_void) -> Self::ActualType {
        let data_ptr = array_components_data as *const <T as ComponentOrPairId>::CastType;
        // SAFETY: up to this point we have checked that the data is not null
        unsafe { (*data_ptr).clone() }
    }
}

impl<T> ClonedTupleTypeOperation for Option<&T>
where
    T: ComponentOrPairId,
    <T as ComponentOrPairId>::CastType: Clone,
{
    type ActualType = Option<<T as ComponentOrPairId>::CastType>;
    type OnlyType = T;
    const IS_OPTION: bool = true;

    fn create_tuple_data(array_components_data: *mut c_void) -> Self::ActualType {
        if array_components_data.is_null() {
            None
        } else {
            let data_ptr = array_components_data as *const <T as ComponentOrPairId>::CastType;
            Some(unsafe { (*data_ptr).clone() })
        }
    }
}

#[diagnostic::on_unimplemented(
    message = "`there is a problem with {Self}`. Please double check the signature and if the types implement clone and are not empty components (tags) or the relationship are not made of 2 tags",
    label = "Failure in cloned signature",
    note = "Valid syntax: `.cloned::<(Position,)>()` -- single component",
    note = "Valid syntax: `.cloned::<(Position, Velocity)>()` -- multiple components",
    note = "Valid syntax: `.cloned::<((Position, Velocity))>()` -- relationship",
    note = "Valid syntax: `.cloned::<((Position, Velocity),Mass, etc)>()` -- multiple components: relationship and component"
)]
pub trait ClonedTuple: Sized {
    type Pointers: ClonedComponentPointers<Self>;
    type TupleType<'a>;

    fn create_ptrs<'a, const SHOULD_PANIC: bool>(
        world: impl WorldProvider<'a>,
        entity: Entity,
        record: *const ecs_record_t,
    ) -> Self::Pointers {
        Self::Pointers::new::<'a, SHOULD_PANIC>(world, entity, record)
    }

    fn populate_array_ptrs<'a, const SHOULD_PANIC: bool>(
        world: impl WorldProvider<'a>,
        entity: Entity,
        record: *const ecs_record_t,
        components: &mut [*mut c_void],
        #[cfg(feature = "flecs_safety_locks")] ids: &mut [u64],
    ) -> bool;

    fn create_tuple<'a>(array_components: &[*mut c_void]) -> Self::TupleType<'a>;
}

/////////////////////
// The higher sized tuples are done by a macro towards the bottom of this file.
/////////////////////

#[rustfmt::skip]
impl<A> ClonedTuple for A
where
    A: ClonedTupleTypeOperation,
{
    type Pointers = ComponentsData<A, 1>;
    type TupleType<'e> = A::ActualType;

    fn populate_array_ptrs<'a, const SHOULD_PANIC: bool>(
        world: impl WorldProvider<'a>, entity: Entity, record: *const ecs_record_t, components: &mut [*mut c_void],
        #[cfg(feature = "flecs_safety_locks")] ids: &mut [u64]
    ) -> bool {
        let world_ref = world.world();
        let world_ptr = unsafe { sys::ecs_get_world(world_ref.ptr_mut() as *const c_void) as *mut sys::ecs_world_t };
        let table = unsafe { (*record).table };
        let entity = *entity;
        let mut has_all_components = true;

        let id = <A::OnlyType as ComponentOrPairId>::get_id(world_ref);

        #[cfg(feature = "flecs_safety_locks")]
        { 
            ids[0] = id;
            
        }

        let component_ptr = unsafe { sys::ecs_rust_get_id(world_ptr, entity, record,id) };

            if component_ptr.is_null() {
                components[0] = core::ptr::null_mut();
                has_all_components = false;
                if SHOULD_PANIC && !A::IS_OPTION {
                    ecs_assert!(false, FlecsErrorCode::OperationFailed,
                        "Component `{}` not found on `EntityView::cloned` operation
                        with parameters: `{}`.
                        Use `try_cloned` variant to avoid assert/panicking if you want to handle the error
                        or use `Option<{}> instead to handle individual cases.",
                        core::any::type_name::<A::OnlyType>(), core::any::type_name::<Self>(), core::any::type_name::<A::ActualType>());
                    panic!("Component `{}` not found on `EntityView::cloned` operation
                    with parameters: `{}`.
                    Use `try_cloned` variant to avoid assert/panicking if
                    you want to handle the error or use `Option<{}>
                    instead to handle individual cases.",
                    core::any::type_name::<A::OnlyType>(), core::any::type_name::<Self>(), core::any::type_name::<A::ActualType>());
                }
            } else {
                components[0] = component_ptr;
            }

        has_all_components
    }

    fn create_tuple<'a>(array_components: &[*mut c_void]) -> Self::TupleType<'a> {
        A::create_tuple_data(array_components[0])
    }
}

pub struct Wrapper<T>(T);

pub trait TupleForm<'a, T, U> {
    type Tuple;
    const IS_OPTION: bool;

    fn return_type_for_tuple(array: *mut U, index: usize) -> Self::Tuple;
}

impl<'a, T: 'a> TupleForm<'a, T, T> for Wrapper<T> {
    type Tuple = &'a mut T;
    const IS_OPTION: bool = false;

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple(array: *mut T, index: usize) -> Self::Tuple {
        unsafe { &mut (*array.add(index)) }
    }
}

impl<'a, T: 'a> TupleForm<'a, Option<T>, T> for Wrapper<T> {
    type Tuple = Option<&'a mut T>;
    const IS_OPTION: bool = true;

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple(array: *mut T, index: usize) -> Self::Tuple {
        unsafe {
            if array.is_null() {
                None
            } else {
                Some(&mut (*array.add(index)))
            }
        }
    }
}

macro_rules! tuple_count {
    () => { 0 };
    ($head:ident) => { 1 };
    ($head:ident, $($tail:ident),*) => { 1 + tuple_count!($($tail),*) };
}

macro_rules! impl_cloned_tuple {
    ($($t:ident),*) => {
        impl<$($t: ClonedTupleTypeOperation),*> ClonedTuple for ($($t,)*) {
            type TupleType<'e> = ($(
                $t::ActualType,
            )*);

            type Pointers = ComponentsData<Self, { tuple_count!($($t),*) }>;

            #[allow(unused)]
            fn populate_array_ptrs<'a, const SHOULD_PANIC: bool>(
                world: impl WorldProvider<'a>, entity: Entity, record: *const ecs_record_t, components: &mut [*mut c_void],
                #[cfg(feature = "flecs_safety_locks")] ids : &mut [u64]
            ) -> bool {

                let world_ref = world.world();
                let world_ptr = unsafe { sys::ecs_get_world(world_ref.ptr_mut() as *const c_void) as *mut sys::ecs_world_t };
                let table = unsafe { (*record).table };
                let entity = *entity;
                let mut index : usize = 0;
                let mut has_all_components = true;

                $(
                    let id = <$t::OnlyType as ComponentOrPairId>::get_id(world_ref);

                                      #[cfg(feature = "flecs_safety_locks")]
                    {
                        ids[index] = id;
                    }

                    let component_ptr = unsafe { sys::ecs_rust_get_id(world_ptr, entity, record, id) };

                    if !component_ptr.is_null() {
                        components[index] = component_ptr;
                    } else {
                        components[index] = core::ptr::null_mut();
                        if !$t::IS_OPTION {
                            if SHOULD_PANIC {
                                ecs_assert!(false, FlecsErrorCode::OperationFailed,
                                    "Component `{}` not found on `EntityView::cloned` operation 
                                    with parameters: `{}`. 
                                    Use `try_cloned` variant to avoid assert/panicking if you want to handle 
                                    the error or use `Option<{}> instead to handle individual cases.",
                                    core::any::type_name::<$t::OnlyType>(), core::any::type_name::<Self>(),
                                    core::any::type_name::<$t::ActualType>());
                                panic!("Component `{}` not found on `EntityView::cloned`operation 
                                with parameters: `{}`. 
                                Use `try_cloned` variant to avoid assert/panicking if you want to handle the error 
                                or use `Option<{}> instead to handle individual cases.", core::any::type_name::<$t::OnlyType>(),
                                core::any::type_name::<Self>(), core::any::type_name::<$t::ActualType>());
                            }
                            has_all_components = false;
                        }
                    }
                    index += 1;
                )*

                has_all_components
            }

            #[allow(unused, clippy::unused_unit)]
            fn create_tuple<'a>(array_components: &[*mut c_void]) -> Self::TupleType<'a> {
                let mut column: isize = -1;
                ($({
                    column += 1;
                    $t::create_tuple_data(array_components[column as usize])
                },)*)
            }

        }
    }
}

tuples!(impl_cloned_tuple, 0, 32);
