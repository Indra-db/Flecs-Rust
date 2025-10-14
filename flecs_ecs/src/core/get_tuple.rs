#![allow(unused)]

use core::ffi::c_void;
use core::marker::PhantomData;

use crate::core::*;
use crate::sys;
use flecs_ecs_derive::tuples;
use sys::ecs_record_t;

#[cfg(feature = "flecs_safety_locks")]
#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub enum SafetyInfo {
    Read(sys::ecs_safety_info_t),
    Write(sys::ecs_safety_info_t),
}

#[cfg(feature = "flecs_safety_locks")]
impl Default for SafetyInfo {
    #[inline]
    fn default() -> Self {
        SafetyInfo::Read(sys::ecs_safety_info_t::default())
    }
}

pub struct ComponentsData<T: GetTuple, const LEN: usize> {
    pub array_components: [*mut c_void; LEN],
    pub has_all_components: bool,
    #[cfg(feature = "flecs_safety_locks")]
    pub(crate) safety_info: [SafetyInfo; LEN],
    _marker: PhantomData<T>,
}

pub trait GetComponentPointers<T: GetTuple> {
    fn new<'a, const SHOULD_PANIC: bool>(
        world: impl WorldProvider<'a>,
        entity: Entity,
        record: *const ecs_record_t,
    ) -> Self;

    fn new_singleton<'a, const SHOULD_PANIC: bool>(world: impl WorldProvider<'a>) -> Self;

    fn get_tuple<'a>(&self) -> T::TupleType<'a>;

    fn has_all_components(&self) -> bool;

    fn component_ptrs(&self) -> &[*mut c_void];

    #[cfg(feature = "flecs_safety_locks")]
    fn safety_info(&self) -> &[SafetyInfo];
}

impl<T: GetTuple, const LEN: usize> GetComponentPointers<T> for ComponentsData<T, LEN> {
    fn new<'a, const SHOULD_PANIC: bool>(
        world: impl WorldProvider<'a>,
        entity: Entity,
        record: *const ecs_record_t,
    ) -> Self {
        let mut array_components = [core::ptr::null::<c_void>() as *mut c_void; LEN];

        #[cfg(feature = "flecs_safety_locks")]
        let mut safety_info = [SafetyInfo::Read(sys::ecs_safety_info_t::default()); LEN];

        let has_all_components = T::populate_array_ptrs::<SHOULD_PANIC>(
            world,
            entity,
            record,
            &mut array_components[..],
            #[cfg(feature = "flecs_safety_locks")]
            &mut safety_info[..],
        );

        Self {
            array_components,
            has_all_components,
            #[cfg(feature = "flecs_safety_locks")]
            safety_info,
            _marker: PhantomData::<T>,
        }
    }

    fn new_singleton<'a, const SHOULD_PANIC: bool>(world: impl WorldProvider<'a>) -> Self {
        let mut array_components = [core::ptr::null::<c_void>() as *mut c_void; LEN];

        #[cfg(feature = "flecs_safety_locks")]
        let mut safety_info = [SafetyInfo::Read(sys::ecs_safety_info_t::default()); LEN];

        let has_all_components = T::populate_array_ptrs_singleton::<SHOULD_PANIC>(
            world,
            &mut array_components[..],
            #[cfg(feature = "flecs_safety_locks")]
            &mut safety_info[..],
        );

        Self {
            array_components,
            has_all_components,
            #[cfg(feature = "flecs_safety_locks")]
            safety_info,
            _marker: PhantomData::<T>,
        }
    }

    fn get_tuple<'a>(&self) -> T::TupleType<'a> {
        T::create_tuple(&self.array_components[..])
    }

    fn has_all_components(&self) -> bool {
        self.has_all_components
    }

    fn component_ptrs(&self) -> &[*mut c_void] {
        &self.array_components
    }

    #[cfg(feature = "flecs_safety_locks")]
    fn safety_info(&self) -> &[SafetyInfo] {
        &self.safety_info
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
        world: impl WorldProvider<'a>,
        entity: Entity,
        record: *const ecs_record_t,
    ) -> Self::Pointers {
        Self::Pointers::new::<'a, SHOULD_PANIC>(world, entity, record)
    }

    fn create_ptrs_singleton<'a, const SHOULD_PANIC: bool>(
        world: impl WorldProvider<'a>,
    ) -> Self::Pointers {
        Self::Pointers::new_singleton::<'a, SHOULD_PANIC>(world)
    }

    #[inline(always)]
    fn internal_populate_array_ptrs<'a, const SHOULD_PANIC: bool, T: GetTupleTypeOperation>(
        world: &WorldRef<'a>,
        world_ptr: *mut sys::ecs_world_t,
        entity: u64,
        record: *const ecs_record_t,
        id: u64,
        components: &mut [*mut c_void],
        has_all_components: &mut bool,
        index: usize,
        #[cfg(feature = "flecs_safety_locks")] safety_info: &mut [SafetyInfo],
    ) {
        if <T::OnlyType as ComponentOrPairId>::IS_PAIR {
            assert!(
                {
                    let first = ecs_first(id, world);
                    first != flecs::Wildcard::ID && first != flecs::Any::ID
                },
                "Pair with flecs::Wildcard or flecs::Any as first terms are not supported"
            );

            assert!(
                {
                    let id = unsafe { sys::ecs_get_typeid(world_ptr, id) };
                    let cast_id =
                        world.component_id::<<T::OnlyType as ComponentOrPairId>::CastType>();
                    id != 0 && id == cast_id
                },
                "Pair is not a (data) component. Possible cause: PairIsTag trait or cast type is not the same as the pair due to flecs::Wildcard or flecs::Any"
            );
        }

        let get_ptr = if T::OnlyType::IS_ENUM {
            let target: sys::ecs_id_t = unsafe { sys::ecs_get_target(world_ptr, entity, id, 0) };

            if target != 0 {
                if !T::IS_IMMUTABLE {
                    ecs_assert!(
                        false,
                        "Enums registered with `add_enum` should be `get` immutable, changing it won't actually change the value."
                    );
                }

                #[cfg(feature = "flecs_meta")]
                {
                    let id_underlying_type = world.component_id::<i32>();
                    let pair_id = ecs_pair(flecs::Constant::ID, *id_underlying_type);
                    let record = unsafe { sys::ecs_record_find(world_ptr, target) };
                    let constant_value = unsafe {
                        sys::flecs_get_id_from_record(world_ptr, target, record, pair_id)
                    };
                    ecs_assert!(
                        !constant_value.component_ptr.is_null(),
                        FlecsErrorCode::InternalError,
                        "missing enum constant value {}",
                        core::any::type_name::<T>()
                    );

                    constant_value
                }

                // Fallback if we don't have the reflection addon
                #[cfg(not(feature = "flecs_meta"))]
                {
                    // get constant value from constant entity
                    let constant_value =
                        unsafe { sys::flecs_get_id_from_record(world_ptr, entity, record, id) };

                    ecs_assert!(
                        !constant_value.component_ptr.is_null(),
                        FlecsErrorCode::InternalError,
                        "missing enum constant value {}",
                        core::any::type_name::<T>()
                    );

                    unsafe { constant_value }
                }
            } else {
                // if there is no matching pair for (r,*), try just r
                unsafe { sys::flecs_get_id_from_record(world_ptr, entity, record, id) }
            }
        } else if T::IS_IMMUTABLE {
            unsafe { sys::flecs_get_id_from_record(world_ptr, entity, record, id) }
        } else {
            unsafe { sys::flecs_get_mut_id_from_record(world_ptr, record, id) }
        };

        let component_ptr = get_ptr.component_ptr;

        if component_ptr.is_null() {
            components[index] = core::ptr::null_mut();
            if !T::IS_OPTION {
                if SHOULD_PANIC {
                    ecs_assert!(
                        false,
                        FlecsErrorCode::OperationFailed,
                        "Component `{}` not found on `EntityView::get` operation 
with parameters: `{}`. 
Use `try_get` variant to avoid assert/panicking if you want to handle 
the error or use `Option<{}> instead to handle individual cases.",
                        core::any::type_name::<T::OnlyType>(),
                        core::any::type_name::<Self>(),
                        core::any::type_name::<T::ActualType<'a>>()
                    );
                    panic!(
                        "Component `{}` not found on `EntityView::get`operation 
with parameters: `{}`. 
Use `try_get` variant to avoid assert/panicking if you want to handle the error 
or use `Option<{}> instead to handle individual cases.",
                        core::any::type_name::<T::OnlyType>(),
                        core::any::type_name::<Self>(),
                        core::any::type_name::<T::ActualType<'a>>()
                    );
                }
                *has_all_components = false;
            }
        } else {
            components[index] = component_ptr;
            #[cfg(feature = "flecs_safety_locks")]
            {
                if T::IS_IMMUTABLE {
                    safety_info[index] = SafetyInfo::Read(get_ptr.si);
                } else {
                    safety_info[index] = SafetyInfo::Write(get_ptr.si);
                }
            }
        }
    }

    fn populate_array_ptrs<'a, const SHOULD_PANIC: bool>(
        world: impl WorldProvider<'a>,
        entity: Entity,
        record: *const ecs_record_t,
        components: &mut [*mut c_void],
        #[cfg(feature = "flecs_safety_locks")] safety_info: &mut [SafetyInfo],
    ) -> bool;

    fn populate_array_ptrs_singleton<'a, const SHOULD_PANIC: bool>(
        world: impl WorldProvider<'a>,
        components: &mut [*mut c_void],
        #[cfg(feature = "flecs_safety_locks")] safety_info: &mut [SafetyInfo],
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
        world: impl WorldProvider<'a>,
        entity: Entity,
        record: *const ecs_record_t,
        components: &mut [*mut c_void],
        #[cfg(feature = "flecs_safety_locks")] safety_info: &mut [SafetyInfo],
    ) -> bool {
        let world = world.world();
        let world_ptr = unsafe {
            sys::ecs_get_world(world.world_ptr() as *const c_void) as *mut sys::ecs_world_t
        };
        let entity = *entity;
        let id = <A::OnlyType as ComponentOrPairId>::get_id(world);

        let mut has_all_components = true;
        Self::internal_populate_array_ptrs::<SHOULD_PANIC, A>(
            &world,
            world_ptr,
            entity,
            record,
            id,
            components,
            &mut has_all_components,
            0,
            #[cfg(feature = "flecs_safety_locks")]
            safety_info,
        );

        has_all_components
    }

    fn populate_array_ptrs_singleton<'a, const SHOULD_PANIC: bool>(
    world: impl WorldProvider<'a>,
    components: &mut [*mut c_void],
    #[cfg(feature = "flecs_safety_locks")] safety_info: &mut [SafetyInfo],
    ) -> bool {
        let world = world.world();
        let world_ptr = unsafe {
            sys::ecs_get_world(world.world_ptr() as *const c_void) as *mut sys::ecs_world_t
        };
        let entity = <<A::OnlyType as ComponentOrPairId>::First>::entity_id(world);
        let record = unsafe { sys::ecs_record_find(world_ptr, entity) };
        let id =  <A::OnlyType as ComponentOrPairId>::get_id(world);
        let mut has_all_components = true;

        Self::internal_populate_array_ptrs::<SHOULD_PANIC,A>(
            &world,
            world_ptr,
            entity,
            record,
            id,
            components,
            &mut has_all_components,
            0,
            #[cfg(feature = "flecs_safety_locks")]
            safety_info,
        );

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
                world: impl WorldProvider<'a>, entity: Entity, record: *const ecs_record_t, components: &mut [*mut c_void], #[cfg(feature = "flecs_safety_locks")] safety_info : &mut [SafetyInfo]
            ) -> bool {

                let world_ptr = unsafe { sys::ecs_get_world(world.world_ptr() as *const c_void) as *mut sys::ecs_world_t };
                let world_ref = world.world();
                let entity = *entity;
                let mut index : usize = 0;
                let mut has_all_components = true;

                $(
                    let id = <$t::OnlyType as ComponentOrPairId>::get_id(world_ref);
                    Self::internal_populate_array_ptrs::<SHOULD_PANIC, $t>(
                        &world_ref,
                        world_ptr,
                        entity,
                        record,
                        id,
                        components,
                        &mut has_all_components,
                        index,
                        #[cfg(feature = "flecs_safety_locks")]
                        safety_info,
                    );
                    index += 1;
                )*

                has_all_components
            }


            #[allow(unused)]
            fn populate_array_ptrs_singleton<'a, const SHOULD_PANIC: bool>(
                world: impl WorldProvider<'a>, components: &mut [*mut c_void], #[cfg(feature = "flecs_safety_locks")] safety_info : &mut [SafetyInfo]
            ) -> bool {

                let world_ptr = unsafe { sys::ecs_get_world(world.world_ptr() as *const c_void) as *mut sys::ecs_world_t };
                let world_ref = world.world();
                let mut index : usize = 0;
                let mut has_all_components = true;

                $(
                    let entity = <<$t::OnlyType as ComponentOrPairId>::First>::entity_id(world_ref);
                    let record = unsafe { sys::ecs_record_find(world_ptr, entity) };
                    let id = <$t::OnlyType as ComponentOrPairId>::get_id(world_ref);

                    Self::internal_populate_array_ptrs::<SHOULD_PANIC, $t>(
                        &world_ref,
                        world_ptr,
                        entity,
                        record,
                        id,
                        components,
                        &mut has_all_components,
                        index,
                        #[cfg(feature = "flecs_safety_locks")]
                        safety_info,
                    );
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
