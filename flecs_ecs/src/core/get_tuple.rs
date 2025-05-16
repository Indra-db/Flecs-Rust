#![allow(unused)]

use core::ffi::c_void;
use core::marker::PhantomData;

use crate::core::*;
use crate::sys;
use flecs_ecs_derive::tuples;
use sys::ecs_record_t;

#[cfg(feature = "flecs_safety_readwrite_locks")]
#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub enum ReadWriteId {
    Read(u64),
    Write(u64),
}

#[cfg(feature = "flecs_safety_readwrite_locks")]
impl Default for ReadWriteId {
    #[inline]
    fn default() -> Self {
        ReadWriteId::Read(0)
    }
}

#[cfg(feature = "flecs_safety_readwrite_locks")]
impl core::ops::Deref for ReadWriteId {
    type Target = u64;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        match self {
            ReadWriteId::Read(id) => id,
            ReadWriteId::Write(id) => id,
        }
    }
}

#[cfg(feature = "flecs_safety_readwrite_locks")]
pub trait ColumnIndexArray {
    fn init() -> Self;
    fn column_indices_mut(&mut self) -> &mut [ComponentTypeRWLock];
}
#[cfg(feature = "flecs_safety_readwrite_locks")]
impl<const LEN: usize> ColumnIndexArray for [ComponentTypeRWLock; LEN] {
    fn init() -> Self {
        [ComponentTypeRWLock::Dense((0, ReadWriteId::default())); LEN]
    }
    fn column_indices_mut(&mut self) -> &mut [ComponentTypeRWLock] {
        self
    }
}

pub struct ComponentsData<T: GetTuple, const LEN: usize> {
    pub array_components: [*mut c_void; LEN],
    pub has_all_components: bool,
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    pub(crate) read_write_ids: [ReadWriteId; LEN],
    _marker: PhantomData<T>,
}

pub trait GetComponentPointers<T: GetTuple> {
    fn new<'a, const SHOULD_PANIC: bool>(
        world: impl WorldProvider<'a>,
        entity: Entity,
        record: *const ecs_record_t,
    ) -> Self;

    fn get_tuple<'a>(&self) -> T::TupleType<'a>;

    fn has_all_components(&self) -> bool;

    #[cfg(feature = "flecs_safety_readwrite_locks")]
    fn read_write_ids(&self) -> &[ReadWriteId];
}

impl<T: GetTuple, const LEN: usize> GetComponentPointers<T> for ComponentsData<T, LEN> {
    fn new<'a, const SHOULD_PANIC: bool>(
        world: impl WorldProvider<'a>,
        entity: Entity,
        record: *const ecs_record_t,
    ) -> Self {
        let mut array_components = [core::ptr::null::<c_void>() as *mut c_void; LEN];

        #[cfg(feature = "flecs_safety_readwrite_locks")]
        let mut read_write_ids = [ReadWriteId::Read(0); LEN];

        let has_all_components = T::populate_array_ptrs::<SHOULD_PANIC>(
            world,
            entity,
            record,
            &mut array_components[..],
            #[cfg(feature = "flecs_safety_readwrite_locks")]
            &mut read_write_ids,
        );

        Self {
            array_components,
            has_all_components,
            #[cfg(feature = "flecs_safety_readwrite_locks")]
            read_write_ids,
            _marker: PhantomData::<T>,
        }
    }

    fn get_tuple<'a>(&self) -> T::TupleType<'a> {
        T::create_tuple(&self.array_components[..])
    }

    fn has_all_components(&self) -> bool {
        self.has_all_components
    }

    #[cfg(feature = "flecs_safety_readwrite_locks")]
    fn read_write_ids(&self) -> &[ReadWriteId] {
        &self.read_write_ids
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
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    type ArrayColumnIndex: ColumnIndexArray;
    const ALL_IMMUTABLE: bool;

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
        #[cfg(feature = "flecs_safety_readwrite_locks")] ids: &mut [ReadWriteId],
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
    #[cfg(feature = "flecs_safety_readwrite_locks")]
    type ArrayColumnIndex = [ComponentTypeRWLock; 1];
    const ALL_IMMUTABLE: bool = A::IS_IMMUTABLE;

    fn populate_array_ptrs<'a, const SHOULD_PANIC: bool>(
        world: impl WorldProvider<'a>, entity: Entity, record: *const ecs_record_t, components: &mut [*mut c_void], #[cfg(feature = "flecs_safety_readwrite_locks")] ids : &mut [ReadWriteId]
    ) -> bool {
        let world = world.world();
        let world_ptr = unsafe { sys::ecs_get_world(world.world_ptr() as *const c_void) as *mut sys::ecs_world_t };
        let table = unsafe { (*record).table };
        let entity = *entity;
        let id = <A::OnlyType as ComponentOrPairId>::get_id(world);

        #[cfg(feature = "flecs_safety_readwrite_locks")]
        { 
            if A::IS_IMMUTABLE {
                ids[0] = ReadWriteId::Read(id);
            } else {
                ids[0] = ReadWriteId::Write(id);
            }
        }
        
        if <A::OnlyType as ComponentOrPairId>::IS_PAIR {
            assert!(
                {
                    let id = unsafe { sys::ecs_get_typeid(world_ptr, id) };
                    let cast_id = world.component_id::<<A::OnlyType as ComponentOrPairId>::CastType>();
                    //TODO: this seems bugged with (flecs::wildcard, bar) where it matches (foo,bar), but says bar is the typeid when it should be foo
                    id != 0 && id == cast_id
                },
                "Pair is not a (data) component. Possible cause: PairIsTag trait or cast type is not the same as the pair due to flecs::Wildcard or flecs::Any"
            );
            assert!(
                {
                    let first = ecs_first(id,world); 
                    first != flecs::Wildcard::ID && first != flecs::Any::ID
                }, "Pair with flecs::Wildcard or flecs::Any as first terms are not supported"
            );

            #[cfg(feature = "flecs_safety_readwrite_locks")]
            { 
                let first_id = *ecs_first(id,world);
                let second_id = *ecs_second(id,world);
                if second_id == flecs::Wildcard::ID || second_id == flecs::Any::ID {
                    let target_id = unsafe { sys::ecs_get_target(world_ptr, entity, id, 0) };
                    if A::IS_IMMUTABLE {
                        ids[0] = ReadWriteId::Read(ecs_pair(first_id, target_id));
                    } else {
                        ids[0] = ReadWriteId::Write(ecs_pair(first_id, target_id));
                    }
                }
            }
        }

        let mut has_all_components = true;
        
        let component_ptr = if A::OnlyType::IS_ENUM {

            let target: sys::ecs_id_t = unsafe {
                sys::ecs_get_target(world_ptr, entity, id, 0)
            };

            if target != 0 {
                if !A::IS_IMMUTABLE {
                    ecs_assert!(false, "Enums registered with `add_enum` should be `get` immutable, changing it won't actually change the value.");
                }

                #[cfg(feature = "flecs_meta")]
                {
                    let id_underlying_type = world.component_id::<i32>();
                    let pair_id = ecs_pair(flecs::meta::Constant::ID, *id_underlying_type);
                    let constant_value = unsafe { sys::ecs_get_id(world_ptr, target, pair_id) } as *mut c_void;

                    ecs_assert!(
                        !constant_value.is_null(),
                        FlecsErrorCode::InternalError,
                        "missing enum constant value {}",
                        core::any::type_name::<A>()
                    );

                    unsafe { constant_value }
                }

               // Fallback if we don't have the reflection addon
               #[cfg(not(feature = "flecs_meta"))]
               {
                 // get constant value from constant entity
                 let constant_value = unsafe { sys::ecs_get_id(world_ptr, target, id) } as *mut c_void;

                 ecs_assert!(
                     !constant_value.is_null(),
                     FlecsErrorCode::InternalError,
                     "missing enum constant value {}",
                     core::any::type_name::<A>()
                 );

                 unsafe { constant_value }
               }
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
            components[0] = core::ptr::null_mut();
            has_all_components = false;
            if SHOULD_PANIC && !A::IS_OPTION {
                ecs_assert!(false, FlecsErrorCode::OperationFailed,
"Component `{}` not found on `EntityView::get` operation 
with parameters: `{}`. 
Use `try_get` variant to avoid assert/panicking if you want to handle the error 
or use `Option<{}> instead to handle individual cases.",
core::any::type_name::<A::OnlyType>(), core::any::type_name::<Self>(), core::any::type_name::<A::ActualType<'a>>());
panic!("Component `{}` not found on `EntityView::get` operation 
with parameters: `{}`. 
Use `try_get` variant to avoid assert/panicking if 
you want to handle the error or use `Option<{}> 
instead to handle individual cases.",
core::any::type_name::<A::OnlyType>(), core::any::type_name::<Self>(), core::any::type_name::<A::ActualType<'a>>());
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
            #[cfg(feature = "flecs_safety_readwrite_locks")]
            type ArrayColumnIndex = [ComponentTypeRWLock; { tuple_count!($($t),*) }];
            const ALL_IMMUTABLE: bool = { $($t::IS_IMMUTABLE &&)* true };

            #[allow(unused)]
            fn populate_array_ptrs<'a, const SHOULD_PANIC: bool>(
                world: impl WorldProvider<'a>, entity: Entity, record: *const ecs_record_t, components: &mut [*mut c_void], #[cfg(feature = "flecs_safety_readwrite_locks")] ids : &mut [ReadWriteId]
            ) -> bool {

                let world_ptr = unsafe { sys::ecs_get_world(world.world_ptr() as *const c_void) as *mut sys::ecs_world_t };
                let world_ref = world.world();
                let table = unsafe { (*record).table };
                let entity = *entity;
                let mut index : usize = 0;
                let mut has_all_components = true;

                $(
                    let id = <$t::OnlyType as ComponentOrPairId>::get_id(world_ref);

                    #[cfg(feature = "flecs_safety_readwrite_locks")]
                    {
                        if $t::IS_IMMUTABLE {
                            ids[index] = ReadWriteId::Read(id);
                        } else {
                            ids[index] = ReadWriteId::Write(id);
                        }
                    }

                    if <$t::OnlyType as ComponentOrPairId>::IS_PAIR {
                        assert!(
                            {
                                let id = unsafe { sys::ecs_rust_get_typeid(world_ptr, id, (*record).idr) };
                                let cast_id = world_ref.component_id::<<$t::OnlyType as ComponentOrPairId>::CastType>();
                                id != 0 && id == cast_id
                            },
                            "Pair is not a (data) component. Possible cause: PairIsTag trait"
                        );

                        assert!(
                            {
                                let first = ecs_first(id,world_ref);
                                first != flecs::Wildcard::ID && first != flecs::Any::ID
                            },
                            "Pair with flecs::Wildcard or flecs::Any as first terms are not supported"
                        );

                        #[cfg(feature = "flecs_safety_readwrite_locks")]
                        {
                            let first_id = *ecs_first(id, world_ref);
                            let second_id = *ecs_second(id,world_ref);
                            if second_id == flecs::Wildcard::ID || second_id == flecs::Any::ID {
                                let target_id = unsafe { sys::ecs_get_target(world_ptr, entity, id, 0) };
                                if $t::IS_IMMUTABLE {
                                    ids[index] = ReadWriteId::Read(ecs_pair(first_id, target_id));
                                } else {
                                    ids[index] = ReadWriteId::Write(ecs_pair(first_id, target_id));
                                }
                            }
                        }
                    }

                    let component_ptr = if $t::OnlyType::IS_ENUM {

                        let target: sys::ecs_id_t = unsafe {
                            sys::ecs_get_target(world_ptr, entity, id, 0)
                        };

                        if target != 0 {
                            if !$t::IS_IMMUTABLE {
                                ecs_assert!(false, "Enums registered with `set_enum` should be `get` immutable, changing it won't actually change the value.");
                            }

                            #[cfg(feature = "flecs_meta")]
                            {
                                let id_underlying_type = world_ref.component_id::<i32>();
                                let pair_id = ecs_pair(flecs::meta::Constant::ID, *id_underlying_type);
                                let constant_value = unsafe { sys::ecs_get_id(world_ptr, target, pair_id) } as *mut c_void;

                                ecs_assert!(
                                    !constant_value.is_null(),
                                    FlecsErrorCode::InternalError,
                                    "missing enum constant value {}",
                                    core::any::type_name::<$t>()
                                );

                                unsafe { constant_value }
                            }

                           // Fallback if we don't have the reflection addon
                           #[cfg(not(feature = "flecs_meta"))]
                           {
                             // get constant value from constant entity
                             let constant_value = unsafe { sys::ecs_get_id(world_ptr, target, id) } as *mut c_void;

                             ecs_assert!(
                                 !constant_value.is_null(),
                                 FlecsErrorCode::InternalError,
                                 "missing enum constant value {}",
                                 core::any::type_name::<$t>()
                             );

                             unsafe { constant_value }
                           }
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
                        components[index] = core::ptr::null_mut();
                        if !$t::IS_OPTION {
                            if SHOULD_PANIC {
                                ecs_assert!(false, FlecsErrorCode::OperationFailed,
"Component `{}` not found on `EntityView::get` operation 
with parameters: `{}`. 
Use `try_get` variant to avoid assert/panicking if you want to handle 
the error or use `Option<{}> instead to handle individual cases.",
core::any::type_name::<$t::OnlyType>(), core::any::type_name::<Self>(),
core::any::type_name::<$t::ActualType<'a>>());
panic!("Component `{}` not found on `EntityView::get`operation 
with parameters: `{}`. 
Use `try_get` variant to avoid assert/panicking if you want to handle the error 
or use `Option<{}> instead to handle individual cases.", core::any::type_name::<$t::OnlyType>(),
core::any::type_name::<Self>(), core::any::type_name::<$t::ActualType<'a>>());
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

tuples!(impl_get_tuple, 0, 32);

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
