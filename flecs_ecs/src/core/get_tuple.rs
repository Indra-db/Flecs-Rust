#![allow(unused)]

use std::ffi::c_void;
use std::marker::PhantomData;

use crate::core::*;
use crate::sys;
use flecs_ecs_derive::tuples;
use sys::ecs_record_t;

pub struct ComponentsData<T: GetTuple, const LEN: usize> {
    pub array_components: [*mut c_void; LEN],
    pub has_all_components: bool,
    _marker: PhantomData<T>,
}

pub trait GetComponentPointers<T: GetTuple> {
    fn new<'a, const SHOULD_PANIC: bool>(
        world: impl IntoWorld<'a>,
        entity: Entity,
        record: *const ecs_record_t,
    ) -> Self;

    fn get_tuple<'a>(&self) -> T::TupleType<'a>;

    fn has_all_components(&self) -> bool;
}

impl<T: GetTuple, const LEN: usize> GetComponentPointers<T> for ComponentsData<T, LEN> {
    fn new<'a, const SHOULD_PANIC: bool>(
        world: impl IntoWorld<'a>,
        entity: Entity,
        record: *const ecs_record_t,
    ) -> Self {
        let mut array_components = [std::ptr::null::<c_void>() as *mut c_void; LEN];

        let has_all_components = T::populate_array_ptrs::<SHOULD_PANIC>(
            world,
            entity,
            record,
            &mut array_components[..],
        );

        Self {
            array_components,
            has_all_components,
            _marker: PhantomData::<T>,
        }
    }

    fn get_tuple<'a>(&self) -> T::TupleType<'a> {
        T::create_tuple(&self.array_components[..])
    }

    fn has_all_components(&self) -> bool {
        self.has_all_components
    }
}

pub trait GetTupleTypeOperation {
    type ActualType<'e>;
    type OnlyType: ComponentOrPairId;
    const IS_OPTION: bool;
    const IS_IMMUTABLE: bool;

    fn create_tuple_data<'a>(array_components_data: *mut c_void) -> Self::ActualType<'a>;
}

impl<T> GetTupleTypeOperation for &T
where
    T: ComponentOrPairId + DataComponent,
{
    type ActualType<'e> = &'e <T as ComponentOrPairId>::CastType;
    type OnlyType = T;
    const IS_OPTION: bool = false;
    const IS_IMMUTABLE: bool = true;

    fn create_tuple_data<'a>(array_components_data: *mut c_void) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as *const <T as ComponentOrPairId>::CastType;
        // SAFETY: up to this point we have checked that the data is not null
        unsafe { &*data_ptr }
    }
}

impl<T> GetTupleTypeOperation for &mut T
where
    T: ComponentOrPairId + DataComponent,
{
    type ActualType<'e> = &'e mut <T as ComponentOrPairId>::CastType;
    type OnlyType = T;
    const IS_OPTION: bool = false;
    const IS_IMMUTABLE: bool = false;

    fn create_tuple_data<'a>(array_components_data: *mut c_void) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as *mut <T as ComponentOrPairId>::CastType;
        // SAFETY: up to this point we have checked that the data is not null
        unsafe { &mut *data_ptr }
    }
}

impl<T> GetTupleTypeOperation for Option<&T>
where
    T: ComponentOrPairId + DataComponent,
{
    type ActualType<'e> = Option<&'e <T as ComponentOrPairId>::CastType>;
    type OnlyType = T;
    const IS_OPTION: bool = true;
    const IS_IMMUTABLE: bool = true;

    fn create_tuple_data<'a>(array_components_data: *mut c_void) -> Self::ActualType<'a> {
        if array_components_data.is_null() {
            None
        } else {
            let data_ptr = array_components_data as *const <T as ComponentOrPairId>::CastType;
            Some(unsafe { &*data_ptr })
        }
    }
}

impl<T> GetTupleTypeOperation for Option<&mut T>
where
    T: ComponentOrPairId + DataComponent,
{
    type ActualType<'e> = Option<&'e mut <T as ComponentOrPairId>::CastType>;
    type OnlyType = T;
    const IS_OPTION: bool = true;
    const IS_IMMUTABLE: bool = false;

    fn create_tuple_data<'a>(array_components_data: *mut c_void) -> Self::ActualType<'a> {
        if array_components_data.is_null() {
            None
        } else {
            let data_ptr = array_components_data as *mut <T as ComponentOrPairId>::CastType;
            Some(unsafe { &mut *data_ptr })
        }
    }
}

#[diagnostic::on_unimplemented(
    message = "`there is a problem with {Self}`. Please double check the signature and if the singular types are not empty components (tags) or the relationship are not made of 2 tags",
    label = "Failure in get signature",
    note = "Valid syntax: `.get::<&Position>()` -- single component",
    note = "Valid syntax: `.get::<(&Position, &mut Velocity)>()` -- multiple components",
    note = "Valid syntax: `.get::<&(Position, Velocity)>()` -- relationship",
    note = "Valid syntax: `.get::<(&(Position, Velocity),&Mass, &etc)>()` -- multiple components: relationship and component"
)]
pub trait GetTuple: Sized {
    type Pointers: GetComponentPointers<Self>;
    type TupleType<'a>;
    const ALL_IMMUTABLE: bool;

    fn create_ptrs<'a, const SHOULD_PANIC: bool>(
        world: impl IntoWorld<'a>,
        entity: Entity,
        record: *const ecs_record_t,
    ) -> Self::Pointers {
        Self::Pointers::new::<'a, SHOULD_PANIC>(world, entity, record)
    }

    fn populate_array_ptrs<'a, const SHOULD_PANIC: bool>(
        world: impl IntoWorld<'a>,
        entity: Entity,
        record: *const ecs_record_t,
        components: &mut [*mut c_void],
    ) -> bool;

    fn create_tuple<'a>(array_components: &[*mut c_void]) -> Self::TupleType<'a>;
}

/////////////////////
// The higher sized tuples are done by a macro towards the bottom of this file.
/////////////////////

#[rustfmt::skip]
impl<A> GetTuple for A
where
    A: GetTupleTypeOperation,
{
    type Pointers = ComponentsData<A, 1>;
    type TupleType<'e> = A::ActualType<'e>;
    const ALL_IMMUTABLE: bool = A::IS_IMMUTABLE;

    fn populate_array_ptrs<'a, const SHOULD_PANIC: bool>(
        world: impl IntoWorld<'a>, entity: Entity, record: *const ecs_record_t, components: &mut [*mut c_void]
    ) -> bool {
        let world_ptr = unsafe { sys::ecs_get_world(world.world_ptr() as *const c_void) as *mut WorldT };
        let table = unsafe { (*record).table };
        let entity = *entity;
        let id = <A::OnlyType as ComponentOrPairId>::get_id(world);
        let mut has_all_components = true;
        
        let component_ptr = if A::OnlyType::IS_ENUM {

            let target: IdT = unsafe {
                sys::ecs_get_target(world_ptr, entity, id, 0)
            };

            if target != 0 {
                if !A::IS_IMMUTABLE {
                    ecs_assert!(false, "Enums registered with `add_enum` should be `get` immutable, changing it won't actually change the value.");
                }
                
                // get constant value from constant entity
                let constant_value = unsafe { sys::ecs_get_id(world_ptr, target, id) } as *mut c_void;

                ecs_assert!(
                    !constant_value.is_null(),
                    FlecsErrorCode::InternalError,
                    "missing enum constant value {}",
                    std::any::type_name::<A>()
                );

                unsafe { constant_value }
            } else {
                // if there is no matching pair for (r,*), try just r
                unsafe { sys::ecs_rust_get_id(world_ptr, entity, record,table,id) }
            }
        } else if A::IS_IMMUTABLE { 
            unsafe { sys::ecs_rust_get_id(world_ptr, entity, record,table,id) }
         } else {
           unsafe { sys::ecs_rust_mut_get_id(world_ptr, entity, record,table,id)}
         };
         
        
        if component_ptr.is_null() {
            components[0] = std::ptr::null_mut();
            has_all_components = false;
            if SHOULD_PANIC && !A::IS_OPTION {
                ecs_assert!(false, FlecsErrorCode::OperationFailed,
"Component `{}` not found on `EntityView::get` operation 
with parameters: `{}`. 
Use `try_get` variant to avoid assert/panicking if you want to handle the error 
or use `Option<{}> instead to handle individual cases.",
std::any::type_name::<A::OnlyType>(), std::any::type_name::<Self>(), std::any::type_name::<A::ActualType<'a>>());
panic!("Component `{}` not found on `EntityView::get` operation 
with parameters: `{}`. 
Use `try_get` variant to avoid assert/panicking if 
you want to handle the error or use `Option<{}> 
instead to handle individual cases.",
std::any::type_name::<A::OnlyType>(), std::any::type_name::<Self>(), std::any::type_name::<A::ActualType<'a>>());
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

macro_rules! impl_get_tuple {
    ($($t:ident),*) => {
        impl<$($t: GetTupleTypeOperation),*> GetTuple for ($($t,)*) {
            type TupleType<'e> = ($(
                $t::ActualType<'e>,
            )*);

            type Pointers = ComponentsData<Self, { tuple_count!($($t),*) }>;

            const ALL_IMMUTABLE: bool = { $($t::IS_IMMUTABLE &&)* true };

            #[allow(unused)]
            fn populate_array_ptrs<'a, const SHOULD_PANIC: bool>(
                world: impl IntoWorld<'a>, entity: Entity, record: *const ecs_record_t, components: &mut [*mut c_void]
            ) -> bool {

                let world_ptr = unsafe { sys::ecs_get_world(world.world_ptr() as *const c_void) as *mut WorldT };
                let world_ref = world.world();
                let table = unsafe { (*record).table };
                let entity = *entity;
                let mut index : usize = 0;
                let mut has_all_components = true;

                $(
                    let id = <$t::OnlyType as ComponentOrPairId>::get_id(world_ref);

                    let component_ptr = if $t::OnlyType::IS_ENUM {

                        let target: IdT = unsafe {
                            sys::ecs_get_target(world_ptr, entity, id, 0)
                        };

                        if target != 0 {
                            if !$t::IS_IMMUTABLE {
                                ecs_assert!(false, "Enums registered with `set_enum` should be `get` immutable, changing it won't actually change the value.");
                            }

                            // get constant value from constant entity
                            let constant_value = unsafe { sys::ecs_get_id(world_ptr, target, id) } as *mut c_void;

                            ecs_assert!(
                                !constant_value.is_null(),
                                FlecsErrorCode::InternalError,
                                "missing enum constant value {}",
                                std::any::type_name::<$t>()
                            );

                            unsafe { constant_value }
                        } else {
                            // if there is no matching pair for (r,*), try just r
                            unsafe { sys::ecs_rust_get_id(world_ptr, entity, record,table,id) }
                        }
                    } else if $t::IS_IMMUTABLE {
                        unsafe { sys::ecs_rust_get_id(world_ptr, entity, record,table,id) }
                     } else {
                       unsafe { sys::ecs_rust_mut_get_id(world_ptr, entity, record,table,id)}
                     };


                    if !component_ptr.is_null() {
                        components[index] = component_ptr;
                    } else {
                        components[index] = std::ptr::null_mut();
                        if !$t::IS_OPTION {
                            if SHOULD_PANIC {
                                ecs_assert!(false, FlecsErrorCode::OperationFailed,
"Component `{}` not found on `EntityView::get` operation 
with parameters: `{}`. 
Use `try_get` variant to avoid assert/panicking if you want to handle 
the error or use `Option<{}> instead to handle individual cases.",
std::any::type_name::<$t::OnlyType>(), std::any::type_name::<Self>(),
std::any::type_name::<$t::ActualType<'a>>());
panic!("Component `{}` not found on `EntityView::get`operation 
with parameters: `{}`. 
Use `try_get` variant to avoid assert/panicking if you want to handle the error 
or use `Option<{}> instead to handle individual cases.", std::any::type_name::<$t::OnlyType>(),
std::any::type_name::<Self>(), std::any::type_name::<$t::ActualType<'a>>());
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

tuples!(impl_get_tuple, 0, 16);

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[derive(Component)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Component)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn are_all_terms_const() {
        assert!(<(&Position, &Velocity) as GetTuple>::ALL_IMMUTABLE);

        assert!(<(Option<&Position>, &Velocity) as GetTuple>::ALL_IMMUTABLE);

        assert!(<(&Position, Option<&Velocity>) as GetTuple>::ALL_IMMUTABLE);

        assert!(<(Option<&Position>, Option<&Velocity>) as GetTuple>::ALL_IMMUTABLE);

        assert!(!<(&mut Position, &Velocity) as GetTuple>::ALL_IMMUTABLE);

        assert!(!<(&Position, &mut Velocity) as GetTuple>::ALL_IMMUTABLE);

        assert!(!<(Option<&mut Position>, &Velocity) as GetTuple>::ALL_IMMUTABLE);

        assert!(!<(Option<&Position>, &mut Velocity) as GetTuple>::ALL_IMMUTABLE);

        assert!(!<(&mut Position, Option<&Velocity>) as GetTuple>::ALL_IMMUTABLE);

        assert!(!<(Option<&mut Position>, Option<&Velocity>) as GetTuple>::ALL_IMMUTABLE);

        assert!(!<(&mut Position, &mut Velocity) as GetTuple>::ALL_IMMUTABLE);

        assert!(!<(Option<&mut Position>, &mut Velocity) as GetTuple>::ALL_IMMUTABLE);
    }
}
