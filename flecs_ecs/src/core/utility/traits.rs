use std::{
    ffi::{c_char, c_void},
    ptr,
};

use flecs_ecs_sys::{
    ecs_ctx_free_t, ecs_filter_str, ecs_iter_action_t, ecs_iter_fini, ecs_iter_t, ecs_os_api,
    ecs_table_lock, ecs_table_unlock,
};

use crate::{
    core::{
        builder,
        c_types::{InOutKind, OperKind},
        component_registration::ComponentInfo,
        Builder, Component, ComponentType, Entity, EntityView, FilterT, FlecsErrorCode, Id, IdT,
        Iter, IterT, Iterable, Query, Struct, Table, TableRange, TableT, Term, UntypedComponent,
        World, WorldT,
    },
    ecs_assert,
};

use super::{ecs_pair, EntityId, ObserverSystemBindingCtx};

/// Represents the input/output type of a component in an ECS system.
///
/// This trait defines the kind of access (input, output, or both) that an ECS system has
/// to a component. Implementing this trait allows specifying whether a component is read,
/// written, or both by a system. This categorization helps in optimizing access patterns
/// and maintaining data consistency within the ECS framework.
///
/// # Associated Constants
///
/// * `IN_OUT`: The kind of access (`InOutKind`) the system has to the component.
///
/// # Associated Types
///
/// * `Type`: The type of the component data. Must implement `ComponentInfo`.
pub trait InOutType {
    const IN_OUT: InOutKind;
    type Type: IntoComponentId;
}

/// Represents the operation type of a system in an ECS framework.
///
/// This trait is used to specify the kind of operation a system performs on a component,
/// such as adding, removing, or setting a component. Implementing this trait allows the ECS
/// framework to understand and optimize the execution of systems based on their operational
/// characteristics.
///
/// # Associated Constants
///
/// * `OPER`: The kind of operation (`OperKind`) the system performs.
///
/// # Associated Types
///
/// * `Type`: The type of the component data. Must implement `ComponentInfo`.
pub trait OperType {
    const OPER: OperKind;
    type Type: ComponentInfo;
}

/// Extracts the entity id from a type.
pub trait IntoEntityId {
    fn get_id(&self) -> u64;
}

impl IntoEntityId for IdT {
    #[inline]
    fn get_id(&self) -> IdT {
        *self
    }
}

impl IntoEntityId for EntityId {
    #[inline]
    fn get_id(&self) -> u64 {
        self.0
    }
}

impl IntoEntityId for Id {
    #[inline]
    fn get_id(&self) -> u64 {
        self.raw_id
    }
}

impl IntoEntityId for EntityView {
    #[inline]
    fn get_id(&self) -> u64 {
        self.raw_id
    }
}

impl IntoEntityId for Entity {
    #[inline]
    fn get_id(&self) -> u64 {
        self.raw_id
    }
}

impl<T> IntoEntityId for Component<T>
where
    T: ComponentInfo,
{
    #[inline]
    fn get_id(&self) -> u64 {
        self.base.entity.raw_id
    }
}

impl IntoEntityId for UntypedComponent {
    #[inline]
    fn get_id(&self) -> u64 {
        self.entity.raw_id
    }
}

impl<T> IntoEntityId for &T
where
    T: IntoEntityId,
{
    #[inline]
    fn get_id(&self) -> u64 {
        T::get_id(*self)
    }
}

impl<T> IntoEntityId for &mut T
where
    T: IntoEntityId,
{
    #[inline]
    fn get_id(&self) -> u64 {
        T::get_id(*self)
    }
}

/// Extension trait for tuples that implement `IntoEntityId`.
/// This extension is useful for when some function only expect one entity id, but not pairs of them.
/// so you only accept `IntoEntityId`. Where both pairs and a single id are accepted, you can use `IntoEntityIdExt`.
pub trait IntoEntityIdExt {
    const IS_PAIR: bool;

    fn get_id(&self) -> u64;

    /// This will return the id of the first part of a pair.
    /// If this is called on a non_pair, it will return the same as get_id.
    #[doc(hidden)] // not meant to be used by the user
    #[inline]
    fn get_id_first(&self) -> u64 {
        self.get_id()
    }

    /// This will return the id of the second part of a pair.
    /// If this is called on a non_pair, it will return the same as get_id.
    #[doc(hidden)]
    #[inline]
    fn get_id_second(&self) -> u64 {
        self.get_id()
    }
}

impl<T, U> IntoEntityIdExt for (T, U)
where
    T: IntoEntityId,
    U: IntoEntityId,
{
    const IS_PAIR: bool = true;

    #[inline]
    fn get_id(&self) -> u64 {
        ecs_pair(self.0.get_id(), self.1.get_id())
    }

    #[doc(hidden)] // not meant to be used by the user
    #[inline]
    fn get_id_first(&self) -> u64 {
        self.0.get_id()
    }

    #[doc(hidden)] // not meant to be used by the user
    #[inline]
    fn get_id_second(&self) -> u64 {
        self.1.get_id()
    }
}

// We can not implement for T where T : IntoEntityId, because it would essentially extend the trait, which we don't want
// so we have to implement for each type that implements IntoEntityId separately.

impl IntoEntityIdExt for IdT {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        IntoEntityId::get_id(self)
    }
}

impl IntoEntityIdExt for EntityId {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        IntoEntityId::get_id(self)
    }
}

impl IntoEntityIdExt for Id {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        IntoEntityId::get_id(self)
    }
}

impl IntoEntityIdExt for EntityView {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        IntoEntityId::get_id(self)
    }
}

impl IntoEntityIdExt for Entity {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        IntoEntityId::get_id(self)
    }
}

impl<T> IntoEntityIdExt for Component<T>
where
    T: ComponentInfo,
{
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        self.base.entity.raw_id
    }
}

impl IntoEntityIdExt for UntypedComponent {
    const IS_PAIR: bool = false;

    #[inline]
    fn get_id(&self) -> u64 {
        self.entity.raw_id
    }
}

impl<T> IntoEntityIdExt for &T
where
    T: IntoEntityIdExt,
{
    const IS_PAIR: bool = T::IS_PAIR;

    #[inline]
    fn get_id(&self) -> u64 {
        T::get_id(*self)
    }
}

impl<T> IntoEntityIdExt for &mut T
where
    T: IntoEntityIdExt,
{
    const IS_PAIR: bool = T::IS_PAIR;

    #[inline]
    fn get_id(&self) -> u64 {
        T::get_id(*self)
    }
}

pub trait IntoWorld {
    fn get_world_raw_mut(&self) -> *mut WorldT;
    #[inline]
    fn get_world_raw(&self) -> *const WorldT {
        self.get_world_raw_mut() as *const WorldT
    }
    #[inline]
    fn get_world(&self) -> World {
        World::new_wrap_raw_world(self.get_world_raw_mut())
    }
}

impl IntoWorld for *mut WorldT {
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        *self
    }
}

impl IntoWorld for *const WorldT {
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        *self as *mut WorldT
    }
}

impl IntoWorld for World {
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        self.raw_world
    }
}

impl IntoWorld for Id {
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        self.world
    }
}

impl IntoWorld for Entity {
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        self.world
    }
}

impl IntoWorld for EntityView {
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        self.world
    }
}

impl<T> IntoWorld for &T
where
    T: IntoWorld,
{
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        T::get_world_raw_mut(*self)
    }
}

impl<T> IntoWorld for &mut T
where
    T: IntoWorld,
{
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        T::get_world_raw_mut(*self)
    }
}

impl<T> IntoWorld for Option<T>
where
    T: IntoWorld,
{
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        match self {
            Some(t) => t.get_world_raw_mut(),
            None => std::ptr::null_mut(),
        }
    }
}

impl<'a, T> IntoWorld for Query<'a, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn get_world_raw_mut(&self) -> *mut WorldT {
        self.world.raw_world
    }
}

// set_override_pair_second

pub trait IntoComponentId {
    const IS_ENUM: bool;
    const IS_PAIR: bool;
    // These types are useful for merging functions in World class such ass add_pair<T,U> into add<T>.
    // When IntoComponentId is not a pair, First and Second will be same
    type First: ComponentInfo;
    type Second: ComponentInfo;

    fn get_id(world: impl IntoWorld) -> IdT;

    /// Get the symbol name of the component.
    ///
    /// # Safety
    ///
    /// Notice that this function for pairs (T, U) will return the type name of the tuple, not the individual components.
    /// This isn't a name stored in the ECS unlike a singular component.
    fn get_name() -> &'static str;
}

impl<T> IntoComponentId for T
where
    T: ComponentInfo,
{
    const IS_ENUM: bool = T::IS_ENUM;
    const IS_PAIR: bool = false;
    type First = T;
    type Second = T;

    #[inline]
    fn get_id(world: impl IntoWorld) -> IdT {
        T::get_id(world.get_world_raw_mut())
    }

    #[inline]
    fn get_name() -> &'static str {
        T::get_symbol_name()
    }
}

impl<T, U> IntoComponentId for (T, U)
where
    T: ComponentInfo,
    U: ComponentInfo + ComponentType<Struct>,
{
    const IS_ENUM: bool = false;
    const IS_PAIR: bool = true;
    type First = T;
    type Second = U;

    #[inline]
    fn get_id(world: impl IntoWorld) -> IdT {
        ecs_pair(
            T::get_id(world.get_world_raw_mut()),
            U::get_id(world.get_world_raw_mut()),
        )
    }

    #[inline]
    fn get_name() -> &'static str {
        std::any::type_name::<(T, U)>()
    }
}

pub trait IntoTable {
    fn get_table(&self) -> *mut TableT;
}

impl IntoTable for *mut TableT {
    #[inline]
    fn get_table(&self) -> *mut TableT {
        *self
    }
}

impl IntoTable for *const TableT {
    #[inline]
    fn get_table(&self) -> *mut TableT {
        *self as *mut TableT
    }
}

impl IntoTable for Table {
    #[inline]
    fn get_table(&self) -> *mut TableT {
        self.get_raw_table()
    }
}

impl IntoTable for TableRange {
    #[inline]
    fn get_table(&self) -> *mut TableT {
        self.table.get_raw_table()
    }
}

pub trait IterOperations {
    fn retrieve_iter(&self) -> IterT;

    fn iter_next(iter: &mut IterT) -> bool;

    fn get_filter_ptr(&self) -> *const FilterT;
}

pub trait IterAPI<'a, T>: IterOperations + IntoWorld
where
    T: Iterable<'a>,
{
    // TODO once we have tests in place, I will split this functionality up into multiple functions, which should give a small performance boost
    // by caching if the query has used a "is_ref" operation.
    // is_ref is true for any query that contains fields that are not matched on the entity itself
    // so parents, prefabs but also singletons, or fields that are matched on a fixed entity (.with<Foo>().src(my_entity))
    /// Each iterator.
    /// The "each" iterator accepts a function that is invoked for each matching entity.
    /// The following function signatures is valid:
    ///  - func(comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// Each iterators are automatically instanced.
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::each`
    #[doc(alias = "iterable::each")]
    fn each(&self, mut func: impl FnMut(T::TupleType)) {
        unsafe {
            let mut iter = self.retrieve_iter();

            while Self::iter_next(&mut iter) {
                let components_data = T::get_array_ptrs_of_components(&iter);
                let iter_count = iter.count as usize;
                let array_components = &components_data.array_components;

                ecs_table_lock(self.get_world_raw_mut(), iter.table);

                for i in 0..iter_count {
                    let tuple = if components_data.is_any_array_a_ref {
                        let is_ref_array_components = &components_data.is_ref_array_components;
                        T::get_tuple_with_ref(array_components, is_ref_array_components, i)
                    } else {
                        T::get_tuple(array_components, i)
                    };
                    func(tuple);
                }

                ecs_table_unlock(self.get_world_raw_mut(), iter.table);
            }
        }
    }

    /// Each iterator.
    /// The "each" iterator accepts a function that is invoked for each matching entity.
    /// The following function signatures is valid:
    ///  - func(e : Entity , comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// Each iterators are automatically instanced.
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::each`
    #[doc(alias = "iterable::each")]
    fn each_entity(&self, mut func: impl FnMut(&mut Entity, T::TupleType)) {
        unsafe {
            let mut iter = self.retrieve_iter();
            let world = self.get_world_raw_mut();
            while Self::iter_next(&mut iter) {
                let components_data = T::get_array_ptrs_of_components(&iter);
                let iter_count = iter.count as usize;
                let array_components = &components_data.array_components;

                ecs_table_lock(world, iter.table);

                // TODO random thought, I think I can determine the elements is a ref or not before the for loop and then pass two arrays with the indices of the ref and non ref elements
                // I will come back to this in the future, my thoughts are somewhere else right now. If my assumption is correct, this will get rid of the branch in the for loop
                // and potentially allow for more conditions for vectorization to happen. This could potentially offer a (small) performance boost since the branch predictor avoids probably
                // most of the cost since the branch is almost always the same.
                // update: I believe it's not possible due to not knowing the order of the components in the tuple. I will leave this here for now, maybe I will come back to it in the future.
                for i in 0..iter_count {
                    let mut entity = Entity::new_from_existing_raw(world, *iter.entities.add(i));

                    let tuple = if components_data.is_any_array_a_ref {
                        let is_ref_array_components = &components_data.is_ref_array_components;
                        T::get_tuple_with_ref(array_components, is_ref_array_components, i)
                    } else {
                        T::get_tuple(array_components, i)
                    };

                    func(&mut entity, tuple);
                }

                ecs_table_unlock(world, iter.table);
            }
        }
    }

    fn each_iter(&self, mut func: impl FnMut(&mut Iter, usize, T::TupleType)) {
        unsafe {
            let mut iter = self.retrieve_iter();
            let world = self.get_world_raw_mut();

            while Self::iter_next(&mut iter) {
                let components_data = T::get_array_ptrs_of_components(&iter);
                let iter_count = {
                    if iter.count == 0 {
                        1_usize
                    } else {
                        iter.count as usize
                    }
                };
                let array_components = &components_data.array_components;

                ecs_table_lock(world, iter.table);

                let mut iter_t = Iter::new(&mut iter);

                for i in 0..iter_count {
                    let tuple = if components_data.is_any_array_a_ref {
                        let is_ref_array_components = &components_data.is_ref_array_components;
                        T::get_tuple_with_ref(array_components, is_ref_array_components, i)
                    } else {
                        T::get_tuple(array_components, i)
                    };
                    func(&mut iter_t, i, tuple);
                }

                ecs_table_unlock(world, iter.table);
            }
        }
    }

    /// find iterator to find an entity
    /// The "find" iterator accepts a function that is invoked for each matching entity and checks if the condition is true.
    /// if it is, it returns that entity.
    /// The following function signatures is valid:
    ///  - func(comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// Each iterators are automatically instanced.
    ///
    /// # Returns
    ///
    /// * Some(Entity) if the entity was found, None if no entity was found
    ///
    /// # See also
    ///
    /// * C++ API: `find_delegate::invoke_callback`
    #[doc(alias = "find_delegate::invoke_callback")]
    fn find(&self, mut func: impl FnMut(T::TupleType) -> bool) -> Option<Entity> {
        unsafe {
            let mut iter = self.retrieve_iter();
            let mut entity: Option<Entity> = None;
            let world = self.get_world_raw_mut();

            while Self::iter_next(&mut iter) {
                let components_data = T::get_array_ptrs_of_components(&iter);
                let iter_count = iter.count as usize;
                let array_components = &components_data.array_components;

                ecs_table_lock(world, iter.table);

                for i in 0..iter_count {
                    let tuple = if components_data.is_any_array_a_ref {
                        let is_ref_array_components = &components_data.is_ref_array_components;
                        T::get_tuple_with_ref(array_components, is_ref_array_components, i)
                    } else {
                        T::get_tuple(array_components, i)
                    };
                    if func(tuple) {
                        entity = Some(Entity::new_from_existing_raw(
                            iter.world,
                            *iter.entities.add(i),
                        ));
                        break;
                    }
                }

                ecs_table_unlock(world, iter.table);
            }
            entity
        }
    }

    /// find iterator to find an entity
    /// The "find" iterator accepts a function that is invoked for each matching entity and checks if the condition is true.
    /// if it is, it returns that entity.
    /// The following function signatures is valid:
    ///  - func(entity : Entity, comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// Each iterators are automatically instanced.
    ///
    /// # Returns
    ///
    /// * Some(Entity) if the entity was found, None if no entity was found
    ///
    /// # See also
    ///
    /// * C++ API: `find_delegate::invoke_callback`
    #[doc(alias = "find_delegate::invoke_callback")]
    fn find_entity(
        &self,
        mut func: impl FnMut(&mut Entity, T::TupleType) -> bool,
    ) -> Option<Entity> {
        unsafe {
            let mut iter = self.retrieve_iter();
            let mut entity_result: Option<Entity> = None;
            let world = self.get_world_raw_mut();

            while Self::iter_next(&mut iter) {
                let components_data = T::get_array_ptrs_of_components(&iter);
                let iter_count = iter.count as usize;
                let array_components = &components_data.array_components;

                ecs_table_lock(world, iter.table);

                for i in 0..iter_count {
                    let mut entity =
                        Entity::new_from_existing_raw(iter.world, *iter.entities.add(i));

                    let tuple = if components_data.is_any_array_a_ref {
                        let is_ref_array_components = &components_data.is_ref_array_components;
                        T::get_tuple_with_ref(array_components, is_ref_array_components, i)
                    } else {
                        T::get_tuple(array_components, i)
                    };
                    if func(&mut entity, tuple) {
                        entity_result = Some(entity);
                        break;
                    }
                }

                ecs_table_unlock(world, iter.table);
            }
            entity_result
        }
    }

    /// find iterator to find an entity.
    /// The "find" iterator accepts a function that is invoked for each matching entity and checks if the condition is true.
    /// if it is, it returns that entity.
    /// The following function signatures is valid:
    ///  - func(iter : Iter, index : usize, comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// Each iterators are automatically instanced.
    ///
    /// # Returns
    ///
    /// * Some(Entity) if the entity was found, None if no entity was found
    ///
    /// # See also
    ///
    /// * C++ API: `find_delegate::invoke_callback`
    #[doc(alias = "find_delegate::invoke_callback")]
    fn find_iter(
        &self,
        mut func: impl FnMut(&mut Iter, usize, T::TupleType) -> bool,
    ) -> Option<Entity> {
        unsafe {
            let mut iter = self.retrieve_iter();
            let mut entity_result: Option<Entity> = None;
            let world = self.get_world_raw_mut();

            while Self::iter_next(&mut iter) {
                let components_data = T::get_array_ptrs_of_components(&iter);
                let array_components = &components_data.array_components;
                let iter_count = {
                    if iter.count == 0 {
                        1_usize
                    } else {
                        iter.count as usize
                    }
                };

                ecs_table_lock(world, iter.table);
                let mut iter_t = Iter::new(&mut iter);

                for i in 0..iter_count {
                    let tuple = if components_data.is_any_array_a_ref {
                        let is_ref_array_components = &components_data.is_ref_array_components;
                        T::get_tuple_with_ref(array_components, is_ref_array_components, i)
                    } else {
                        T::get_tuple(array_components, i)
                    };
                    if func(&mut iter_t, i, tuple) {
                        entity_result = Some(Entity::new_from_existing_raw(
                            iter.world,
                            *iter.entities.add(i),
                        ));
                        break;
                    }
                }

                ecs_table_unlock(world, iter.table);
            }
            entity_result
        }
    }

    /// iter iterator.
    /// The "iter" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    ///  - func(it: &mut Iter, comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// Iter iterators are not automatically instanced. When a result contains
    /// shared components, entities of the result will be iterated one by one.
    /// This ensures that applications can't accidentally read out of bounds by
    /// accessing a shared component as an array.
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::iter`
    #[doc(alias = "iterable::iter")]
    fn iter(&self, mut func: impl FnMut(&mut Iter, T::TupleSliceType)) {
        unsafe {
            let mut iter = self.retrieve_iter();
            let world = self.get_world_raw_mut();

            while Self::iter_next(&mut iter) {
                let components_data = T::get_array_ptrs_of_components(&iter);
                let iter_count = iter.count as usize;
                let array_components = &components_data.array_components;

                ecs_table_lock(world, iter.table);

                let tuple = if components_data.is_any_array_a_ref {
                    let is_ref_array_components = &components_data.is_ref_array_components;
                    T::get_tuple_slices_with_ref(
                        array_components,
                        is_ref_array_components,
                        iter_count,
                    )
                } else {
                    T::get_tuple_slices(array_components, iter_count)
                };
                let mut iter_t = Iter::new(&mut iter);
                func(&mut iter_t, tuple);
                ecs_table_unlock(world, iter.table);
            }
        }
    }

    /// iter iterator.
    /// The "iter" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    ///  - func(it: &mut Iter)
    ///
    /// Iter iterators are not automatically instanced. When a result contains
    /// shared components, entities of the result will be iterated one by one.
    /// This ensures that applications can't accidentally read out of bounds by
    /// accessing a shared component as an array.
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::iter`
    #[doc(alias = "iterable::iter")]
    fn iter_only(&self, mut func: impl FnMut(&mut Iter)) {
        unsafe {
            let mut iter = self.retrieve_iter();
            let world = self.get_world_raw_mut();
            while Self::iter_next(&mut iter) {
                ecs_table_lock(world, iter.table);
                let mut iter_t = Iter::new(&mut iter);
                func(&mut iter_t);
                ecs_table_unlock(world, iter.table);
            }
        }
    }

    /// Get the entity of the current filter
    ///
    /// # Arguments
    ///
    /// * `filter`: the filter to get the entity from
    ///
    /// # Returns
    ///
    /// The entity of the current filter
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::entity`
    #[doc(alias = "filter_base::entity")]
    fn as_entity(&self) -> Entity;

    /// Each term iterator.
    /// The "`each_term`" iterator accepts a function that is invoked for each term
    /// in the filter. The following function signature is valid:
    ///  - func(term: &mut Term)
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::term`
    #[doc(alias = "filter_base::each_term")]
    fn each_term(&self, mut func: impl FnMut(&mut Term)) {
        let filter = self.get_filter_ptr();
        let world = self.get_world();
        unsafe {
            for i in 0..(*filter).term_count {
                let mut term = Term::new_from_term(Some(&world), *(*filter).terms.add(i as usize));
                func(&mut term);
                term.reset(); // prevent freeing resources
            }
        }
    }

    /// Get the term of the current filter at the given index
    ///
    /// # Arguments
    ///
    /// * `index`: the index of the term to get
    /// * `filter`: the filter to get the term from
    ///
    /// # Returns
    ///
    /// The term requested
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::term`
    #[doc(alias = "filter_base::term")]
    fn get_term(&self, index: usize) -> Term {
        let filter = self.get_filter_ptr();
        let world = self.get_world();
        ecs_assert!(
            !filter.is_null(),
            FlecsErrorCode::InvalidParameter,
            "query filter is null"
        );
        Term::new_from_term(Some(&world), unsafe { *(*filter).terms.add(index) })
    }

    /// Get the field count of the current filter
    ///
    /// # Arguments
    ///
    /// * `filter`: the filter to get the field count from
    ///
    /// # Returns
    ///
    /// The field count of the current filter
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::field_count`
    #[doc(alias = "filter_base::field_count")]
    fn field_count(&self) -> i8 {
        let filter = self.get_filter_ptr();
        unsafe { (*filter).field_count }
    }

    /// Convert filter to string expression. Convert filter terms to a string expression.
    /// The resulting expression can be parsed to create the same filter.
    ///
    /// # Arguments
    ///
    /// * `filter`: the filter to convert to a string
    ///
    /// # Returns
    ///
    /// The string representation of the filter
    ///
    /// # See also
    ///
    /// * C++ API: `filter_base::str`
    #[doc(alias = "filter_base::str")]
    #[allow(clippy::inherent_to_string)] // this is a wrapper around a c function
    fn to_string(&self) -> String {
        let filter = self.get_filter_ptr();
        let world = self.get_world_raw_mut();
        let result: *mut c_char = unsafe { ecs_filter_str(world, filter as *const _) };
        let rust_string =
            String::from(unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() });
        unsafe {
            if let Some(free_func) = ecs_os_api.free_ {
                free_func(result as *mut _);
            }
        }
        rust_string
    }

    fn first(&self) -> Entity {
        let mut entity = Entity::default();
        let mut it = self.retrieve_iter();
        let world = self.get_world_raw_mut();
        if Self::iter_next(&mut it) && it.count > 0 {
            entity = Entity::new_from_existing_raw(world, unsafe { *it.entities.add(0) });
            unsafe { ecs_iter_fini(&mut it) };
        }
        entity
    }
}
#[doc(hidden)]
#[allow(non_camel_case_types)]
#[doc(hidden)]
pub mod private {
    use std::{ffi::c_void, ptr};

    use flecs_ecs_sys::{ecs_ctx_free_t, ecs_iter_t, ecs_table_lock, ecs_table_unlock};

    use crate::core::{Entity, Iter, IterT, Iterable, ObserverSystemBindingCtx};

    pub trait internal_ReactorAPI<'a, T>
    where
        T: Iterable<'a>,
    {
        fn set_binding_ctx(&mut self, binding_ctx: *mut c_void) -> &mut Self;

        fn set_binding_ctx_free(&mut self, binding_ctx_free: ecs_ctx_free_t) -> &mut Self;

        fn get_desc_binding_ctx(&self) -> *mut c_void;

        fn set_desc_callback(&mut self, callback: Option<unsafe extern "C" fn(*mut ecs_iter_t)>);

        /// Callback of the each functionality
        ///
        /// # Arguments
        ///
        /// * `iter` - The iterator which gets passed in from `C`
        ///
        /// # See also
        ///
        /// * C++ API: `iter_invoker::invoke_callback`
        unsafe extern "C" fn run_each<Func>(iter: *mut IterT)
        where
            Func: FnMut(T::TupleType),
        {
            let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
            let each = (*ctx).each.unwrap();
            let each = &mut *(each as *mut Func);

            let components_data = T::get_array_ptrs_of_components(&*iter);
            let array_components = &components_data.array_components;
            let iter_count = {
                if (*iter).count == 0 {
                    1_usize
                } else {
                    (*iter).count as usize
                }
            };

            ecs_table_lock((*iter).world, (*iter).table);

            for i in 0..iter_count {
                let tuple = if components_data.is_any_array_a_ref {
                    let is_ref_array_components = &components_data.is_ref_array_components;
                    T::get_tuple_with_ref(array_components, is_ref_array_components, i)
                } else {
                    T::get_tuple(array_components, i)
                };
                each(tuple);
            }

            ecs_table_unlock((*iter).world, (*iter).table);
        }

        /// Callback of the `each_entity` functionality
        ///
        /// # Arguments
        ///
        /// * `iter` - The iterator which gets passed in from `C`
        ///
        /// # See also
        ///
        /// * C++ API: `iter_invoker::invoke_callback`
        #[doc(alias = "iter_invoker::invoke_callback")]
        unsafe extern "C" fn run_each_entity<Func>(iter: *mut IterT)
        where
            Func: FnMut(&mut Entity, T::TupleType),
        {
            let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
            let each_entity = (*ctx).each_entity.unwrap();
            let each_entity = &mut *(each_entity as *mut Func);

            let components_data = T::get_array_ptrs_of_components(&*iter);
            let array_components = &components_data.array_components;
            let iter_count = {
                if (*iter).count == 0 {
                    1_usize
                } else {
                    (*iter).count as usize
                }
            };

            ecs_table_lock((*iter).world, (*iter).table);

            for i in 0..iter_count {
                let mut entity =
                    Entity::new_from_existing_raw((*iter).world, *(*iter).entities.add(i));
                let tuple = if components_data.is_any_array_a_ref {
                    let is_ref_array_components = &components_data.is_ref_array_components;
                    T::get_tuple_with_ref(array_components, is_ref_array_components, i)
                } else {
                    T::get_tuple(array_components, i)
                };

                each_entity(&mut entity, tuple);
            }
            ecs_table_unlock((*iter).world, (*iter).table);
        }

        /// Callback of the `each_iter` functionality
        ///
        /// # Arguments
        ///
        /// * `iter` - The iterator which gets passed in from `C`
        ///
        /// # See also
        ///
        /// * C++ API: `iter_invoker::invoke_callback`
        #[doc(alias = "iter_invoker::invoke_callback")]
        unsafe extern "C" fn run_each_iter<Func>(iter: *mut IterT)
        where
            Func: FnMut(&mut Iter, usize, T::TupleType),
        {
            let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
            let each_iter = (*ctx).each_iter.unwrap();
            let each_iter = &mut *(each_iter as *mut Func);

            let components_data = T::get_array_ptrs_of_components(&*iter);
            let array_components = &components_data.array_components;
            let iter_count = {
                if (*iter).count == 0 {
                    1_usize
                } else {
                    (*iter).count as usize
                }
            };

            ecs_table_lock((*iter).world, (*iter).table);
            let mut iter_t = Iter::new(&mut (*iter));

            for i in 0..iter_count {
                let tuple = if components_data.is_any_array_a_ref {
                    let is_ref_array_components = &components_data.is_ref_array_components;
                    T::get_tuple_with_ref(array_components, is_ref_array_components, i)
                } else {
                    T::get_tuple(array_components, i)
                };

                each_iter(&mut iter_t, i, tuple);
            }
            ecs_table_unlock((*iter).world, (*iter).table);
        }

        /// Callback of the `iter_only` functionality
        ///
        /// # Arguments
        ///
        /// * `iter` - The iterator which gets passed in from `C`
        ///
        /// # See also
        ///
        /// * C++ API: `iter_invoker::invoke_callback`
        #[doc(alias = "iter_invoker::invoke_callback")]
        unsafe extern "C" fn run_iter_only<Func>(iter: *mut IterT)
        where
            Func: FnMut(&mut Iter),
        {
            unsafe {
                let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
                let iter_only = (*ctx).iter_only.unwrap();
                let iter_only = &mut *(iter_only as *mut Func);
                let iter_count = {
                    if (*iter).count == 0 {
                        1_usize
                    } else {
                        (*iter).count as usize
                    }
                };

                ecs_table_lock((*iter).world, (*iter).table);

                for _ in 0..iter_count {
                    let mut iter_t = Iter::new(&mut *iter);
                    iter_only(&mut iter_t);
                }

                ecs_table_unlock((*iter).world, (*iter).table);
            }
        }

        /// Callback of the iter functionality
        ///
        /// # Arguments
        ///
        /// * `iter` - The iterator which gets passed in from `C`
        ///
        /// # See also
        ///
        /// * C++ API: `iter_invoker::invoke_callback`
        #[doc(alias = "iter_invoker::invoke_callback")]
        unsafe extern "C" fn run_iter<Func>(iter: *mut IterT)
        where
            Func: FnMut(&mut Iter, T::TupleSliceType),
        {
            let ctx: *mut ObserverSystemBindingCtx = (*iter).binding_ctx as *mut _;
            let iter_func = (*ctx).iter.unwrap();
            let iter_func = &mut *(iter_func as *mut Func);

            let components_data = T::get_array_ptrs_of_components(&*iter);
            let array_components = &components_data.array_components;
            let iter_count = {
                if (*iter).count == 0 {
                    1_usize
                } else {
                    (*iter).count as usize
                }
            };

            ecs_table_lock((*iter).world, (*iter).table);

            for i in 0..iter_count {
                let tuple = if components_data.is_any_array_a_ref {
                    let is_ref_array_components = &components_data.is_ref_array_components;
                    T::get_tuple_slices_with_ref(array_components, is_ref_array_components, i)
                } else {
                    T::get_tuple_slices(array_components, i)
                };
                let mut iter_t = Iter::new(&mut *iter);
                iter_func(&mut iter_t, tuple);
            }

            ecs_table_unlock((*iter).world, (*iter).table);
        }

        // free functions

        extern "C" fn on_free_each(ptr: *mut c_void) {
            let ptr_func: *mut fn(T::TupleType) = ptr as *mut fn(T::TupleType);
            unsafe {
                ptr::drop_in_place(ptr_func);
            }
        }

        extern "C" fn on_free_each_entity(ptr: *mut c_void) {
            let ptr_func: *mut fn(&mut Entity, T::TupleType) =
                ptr as *mut fn(&mut Entity, T::TupleType);
            unsafe {
                ptr::drop_in_place(ptr_func);
            }
        }

        extern "C" fn on_free_each_iter(ptr: *mut c_void) {
            let ptr_func: *mut fn(&mut Iter, usize, T::TupleType) =
                ptr as *mut fn(&mut Iter, usize, T::TupleType);
            unsafe {
                ptr::drop_in_place(ptr_func);
            }
        }

        extern "C" fn on_free_iter_only(ptr: *mut c_void) {
            let ptr_func: *mut fn(&Iter) = ptr as *mut fn(&Iter);
            unsafe {
                ptr::drop_in_place(ptr_func);
            }
        }

        extern "C" fn on_free_iter(ptr: *mut c_void) {
            let ptr_func: *mut fn(&Iter, T::TupleSliceType) =
                ptr as *mut fn(&Iter, T::TupleSliceType);
            unsafe {
                ptr::drop_in_place(ptr_func);
            }
        }

        /// Get the binding context
        fn get_binding_ctx(&mut self) -> &mut ObserverSystemBindingCtx {
            let mut binding_ctx: *mut ObserverSystemBindingCtx =
                self.get_desc_binding_ctx() as *mut _;

            if binding_ctx.is_null() {
                let new_binding_ctx = Box::<ObserverSystemBindingCtx>::default();
                let static_ref = Box::leak(new_binding_ctx);
                binding_ctx = static_ref;
                self.set_binding_ctx(binding_ctx as *mut c_void);
                self.set_binding_ctx_free(Some(Self::binding_ctx_drop));
            }
            unsafe { &mut *binding_ctx }
        }

        /// drop the binding context
        extern "C" fn binding_ctx_drop(ptr: *mut c_void) {
            let ptr_struct: *mut ObserverSystemBindingCtx = ptr as *mut ObserverSystemBindingCtx;
            unsafe {
                ptr::drop_in_place(ptr_struct);
            }
        }
    }
}
pub trait ReactorAPI<'a, T>: Builder + private::internal_ReactorAPI<'a, T>
where
    T: Iterable<'a>,
{
    /// Set action / ctx
    ///
    /// # Arguments
    ///
    /// * `callback` - the callback to set
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::run`
    #[doc(alias = "system_builder_i::ctx")]
    #[doc(alias = "observer_builder_i::ctx")]
    fn set_run_callback(&mut self, callback: ecs_iter_action_t) -> &mut Self;

    fn set_instanced(&mut self, instanced: bool);

    /// Set context
    ///
    /// # See also
    ///
    /// * C++ API: `observer_builder_i::ctx`
    /// * C++ API: `system_builder_i::ctx`
    #[doc(alias = "observer_builder_i::ctx")]
    #[doc(alias = "system_builder_i::ctx")]
    fn set_context(&mut self, context: *mut c_void) -> &mut Self;

    fn on_each<Func>(&mut self, func: Func) -> <Self as builder::Builder>::BuiltType
    where
        Func: FnMut(T::TupleType) + 'static,
    {
        let binding_ctx = self.get_binding_ctx();

        let each_func = Box::new(func);
        let each_static_ref = Box::leak(each_func);

        binding_ctx.each = Some(each_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_each = Some(Self::on_free_each);

        self.set_desc_callback(Some(Self::run_each::<Func> as unsafe extern "C" fn(_)));

        self.set_instanced(true);

        self.build()
    }

    fn on_each_entity<Func>(&mut self, func: Func) -> <Self as builder::Builder>::BuiltType
    where
        Func: FnMut(&mut Entity, T::TupleType) + 'static,
    {
        let binding_ctx = self.get_binding_ctx();

        let each_entity_func = Box::new(func);
        let each_entity_static_ref = Box::leak(each_entity_func);

        binding_ctx.each_entity = Some(each_entity_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_each_entity = Some(Self::on_free_each_entity);

        self.set_desc_callback(Some(
            Self::run_each_entity::<Func> as unsafe extern "C" fn(_),
        ));

        self.set_instanced(true);

        self.build()
    }

    fn on_each_iter<Func>(&mut self, func: Func) -> <Self as builder::Builder>::BuiltType
    where
        Func: FnMut(&mut Iter, usize, T::TupleType) + 'static,
    {
        let binding_ctx = self.get_binding_ctx();

        let each_iter_func = Box::new(func);
        let each_iter_static_ref = Box::leak(each_iter_func);

        binding_ctx.each_iter = Some(each_iter_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_each_iter = Some(Self::on_free_each_iter);

        self.set_desc_callback(Some(Self::run_each_iter::<Func> as unsafe extern "C" fn(_)));

        self.set_instanced(true);

        self.build()
    }

    fn on_iter_only<Func>(&mut self, func: Func) -> <Self as builder::Builder>::BuiltType
    where
        Func: FnMut(&mut Iter) + 'static,
    {
        let binding_ctx = self.get_binding_ctx();
        let iter_func = Box::new(func);
        let iter_static_ref = Box::leak(iter_func);
        binding_ctx.iter_only = Some(iter_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_iter_only = Some(Self::on_free_iter_only);

        self.set_desc_callback(Some(Self::run_iter_only::<Func> as unsafe extern "C" fn(_)));

        //TODO are we sure this shouldn't be instanced?

        self.build()
    }

    fn on_iter<Func>(&mut self, func: Func) -> <Self as builder::Builder>::BuiltType
    where
        Func: FnMut(&mut Iter, T::TupleSliceType) + 'static,
    {
        let binding_ctx = self.get_binding_ctx();

        let iter_func = Box::new(func);
        let iter_static_ref = Box::leak(iter_func);

        binding_ctx.iter = Some(iter_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_iter = Some(Self::on_free_iter);

        self.set_desc_callback(Some(Self::run_iter::<Func> as unsafe extern "C" fn(_)));

        //TODO are we sure this shouldn't be instanced?

        self.build()
    }
}

macro_rules! implement_reactor_api {
    ($type:ty) => {
        impl<'a, T> internal_ReactorAPI<'a, T> for $type
        where
            T: Iterable<'a>,
        {
            fn set_binding_ctx(&mut self, binding_ctx: *mut c_void) -> &mut Self {
                self.desc.binding_ctx = binding_ctx;
                self
            }

            fn set_binding_ctx_free(
                &mut self,
                binding_ctx_free: flecs_ecs_sys::ecs_ctx_free_t,
            ) -> &mut Self {
                self.desc.binding_ctx_free = binding_ctx_free;
                self
            }

            fn get_desc_binding_ctx(&self) -> *mut c_void {
                self.desc.binding_ctx
            }

            fn set_desc_callback(
                &mut self,
                callback: Option<unsafe extern "C" fn(*mut flecs_ecs_sys::ecs_iter_t)>,
            ) {
                self.desc.callback = callback;
            }
        }

        impl<'a, T> ReactorAPI<'a, T> for $type
        where
            T: Iterable<'a>,
        {
            fn set_run_callback(&mut self, callback: ecs_iter_action_t) -> &mut Self {
                self.desc.run = callback;
                self
            }

            fn set_instanced(&mut self, instanced: bool) {
                self.is_instanced = instanced;
            }

            fn set_context(&mut self, context: *mut c_void) -> &mut Self {
                self.desc.ctx = context;
                self
            }
        }
    };
}

pub(crate) use implement_reactor_api;
