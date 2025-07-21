use core::ffi::c_char;

use flecs_ecs::core::*;
use flecs_ecs::sys;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::string::String;

#[cfg(feature = "flecs_json")]
use alloc::string::ToString;

/// Custom error type for `try_first_only` failures.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FirstOnlyError {
    NoEntities,
    MoreThanOneEntity,
}

pub trait IterOperations {
    #[doc(hidden)]
    fn retrieve_iter(&self) -> sys::ecs_iter_t;

    #[doc(hidden)]
    fn retrieve_iter_stage<'a>(&self, stage: impl WorldProvider<'a>) -> sys::ecs_iter_t;

    #[doc(hidden)]
    fn iter_next(&self, iter: &mut sys::ecs_iter_t) -> bool;

    #[doc(hidden)]
    fn iter_next_func(&self) -> unsafe extern "C" fn(*mut sys::ecs_iter_t) -> bool;

    #[doc(hidden)]
    fn query_ptr(&self) -> *const sys::ecs_query_t;
}

pub trait QueryAPI<'a, P, T>: IterOperations + WorldProvider<'a>
where
    T: QueryTuple,
{
    // TODO once we have tests in place, I will split this functionality up into multiple functions, which should give a small performance boost
    // by caching if the query has used a "is_ref" operation.
    // is_ref is true for any query that contains fields that are not matched on the entity itself
    // so parents, prefabs but also singletons, or fields that are matched on a fixed entity (.with<Foo>().src_id(my_entity))
    /// Each iterator.
    /// The "each" iterator accepts a function that is invoked for each matching entity.
    /// The following function signatures is valid:
    ///  - func(comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// # See also
    ///
    /// * [`World::each()`]
    fn each(&self, mut func: impl FnMut(T::TupleType<'_>)) {
        const {
            assert!(
                !T::CONTAINS_ANY_TAG_TERM,
                "a type provided in the query signature is a Tag and cannot be used with `.each`. use `.run` instead or provide the tag with `.with()`"
            );
        }

        unsafe {
            let mut iter = self.retrieve_iter();
            iter.flags |= sys::EcsIterCppEach;

            while self.iter_next(&mut iter) {
                iter.flags |= sys::EcsIterCppEach;
                let (is_any_array, mut components_data) = T::create_ptrs(&iter);
                let iter_count = {
                    if iter.count == 0 && iter.table.is_null() {
                        1_usize
                    } else {
                        iter.count as usize
                    }
                };

                sys::ecs_table_lock(self.world_ptr_mut(), iter.table);

                if !is_any_array.a_ref && !is_any_array.a_row {
                    for i in 0..iter_count {
                        let tuple = components_data.get_tuple(i);
                        func(tuple);
                    }
                } else if is_any_array.a_row {
                    for i in 0..iter_count {
                        let tuple = components_data.get_tuple_with_row(&iter, i);
                        func(tuple);
                    }
                } else {
                    for i in 0..iter_count {
                        let tuple = components_data.get_tuple_with_ref(i);
                        func(tuple);
                    }
                }

                sys::ecs_table_unlock(self.world_ptr_mut(), iter.table);
            }
        }
    }

    /// Each iterator.
    /// The "each" iterator accepts a function that is invoked for each matching entity.
    /// The following function signatures is valid:
    ///  - func(e : Entity , comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// # See also
    ///
    /// * [`World::each_entity()`]
    fn each_entity(&self, mut func: impl FnMut(EntityView, T::TupleType<'_>)) {
        const {
            assert!(
                !T::CONTAINS_ANY_TAG_TERM,
                "a type provided in the query signature is a Tag and cannot be used with `.each`. use `.run` instead or provide the tag with `.with()`"
            );
        }

        unsafe {
            let world = self.world();
            let world_ptr = world.ptr_mut();
            let mut iter = self.retrieve_iter();

            while self.iter_next(&mut iter) {
                iter.flags |= sys::EcsIterCppEach;

                ecs_assert!(
                    !iter.entities.is_null(),
                    FlecsErrorCode::InvalidParameter,
                    "Query does not return entities ($this variable is not populated).\nQuery: {:?}",
                    world.entity_from_id((*iter.query).entity)
                );

                let (_is_any_array, mut components_data) = T::create_ptrs(&iter);
                let iter_count = {
                    if iter.count == 0 && iter.table.is_null() {
                        1_usize
                    } else {
                        iter.count as usize
                    }
                };

                //sys::ecs_table_lock(world_ptr, iter.table);

                // TODO random thought, I think I can determine the elements is a ref or not before the for loop and then pass two arrays with the indices of the ref and non ref elements
                // I will come back to this in the future, my thoughts are somewhere else right now. If my assumption is correct, this will get rid of the branch in the for loop
                // and potentially allow for more conditions for vectorization to happen. This could potentially offer a (small) performance boost since the branch predictor avoids probably
                // most of the cost since the branch is almost always the same.
                // update: I believe it's not possible due to not knowing the order of the components in the tuple. I will leave this here for now, maybe I will come back to it in the future.

                //if !is_any_array.a_ref && !is_any_array.a_row {
                for i in 0..iter_count {
                    let entity = EntityView::new_from(world, *iter.entities.add(i));
                    let tuple = components_data.get_tuple(i);
                    func(entity, tuple);
                }
                // } else if is_any_array.a_row {
                //     for i in 0..iter_count {
                //         let entity = EntityView::new_from(world, *iter.entities.add(i));
                //         let tuple = components_data.get_tuple_with_row(&iter, i);
                //         func(entity, tuple);
                //     }
                // } else {
                //     for i in 0..iter_count {
                //         let entity = EntityView::new_from(world, *iter.entities.add(i));
                //         let tuple = components_data.get_tuple_with_ref(i);
                //         func(entity, tuple);
                //     }
                // }

                //sys::ecs_table_unlock(world_ptr, iter.table);
            }
        }
    }

    /// Each iterator. This variant of `each` provides access to the [`TableIter`] object,
    /// which contains more information about the object being iterated.
    /// The `usize` argument contains the index of the entity being iterated,
    /// which can be used to obtain entity-specific data from the `TableIter` object.
    ///
    /// # Example
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Position {
    ///     x: i32,
    ///     y: i32,
    /// }
    ///
    /// #[derive(Component, Debug)]
    /// struct Likes;
    ///
    /// let world = World::new();
    ///
    /// let eva = world.entity_named("eva");
    ///
    /// world
    ///     .entity_named("adam")
    ///     .set(Position { x: 10, y: 20 })
    ///     .add((Likes::id(), eva));
    ///
    /// world
    ///     .query::<&Position>()
    ///     .with((Likes::id(), id::<flecs::Wildcard>()))
    ///     .build()
    ///     .each_iter(|it, index, p| {
    ///         let e = it.entity(index).unwrap();
    ///         println!("{:?}: {:?} - {:?}", e.name(), p, it.id(1).to_str());
    ///     });
    ///
    /// // Output:
    /// //  "adam": Position { x: 10, y: 20 } - "(flecs_ecs.main.Likes,eva)"
    /// ```
    fn each_iter(&self, mut func: impl FnMut(TableIter<false, P>, usize, T::TupleType<'_>))
    where
        P: ComponentId,
    {
        const {
            assert!(
                !T::CONTAINS_ANY_TAG_TERM,
                "a type provided in the query signature is a Tag and cannot be used with `.each`. use `.run` instead or provide the tag with `.with()`"
            );
        }

        unsafe {
            let world_ptr = self.world_ptr_mut();
            let mut iter = self.retrieve_iter();
            iter.flags |= sys::EcsIterCppEach;

            while self.iter_next(&mut iter) {
                let (is_any_array, mut components_data) = T::create_ptrs(&iter);
                let iter_count = {
                    if iter.count == 0 && iter.table.is_null() {
                        1_usize
                    } else {
                        iter.count as usize
                    }
                };

                sys::ecs_table_lock(world_ptr, iter.table);

                if !is_any_array.a_ref && !is_any_array.a_row {
                    for i in 0..iter_count {
                        let tuple = components_data.get_tuple(i);
                        let iter_t = TableIter::new(&mut iter);
                        func(iter_t, i, tuple);
                    }
                } else if is_any_array.a_row {
                    for i in 0..iter_count {
                        let tuple = components_data.get_tuple_with_row(&iter, i);
                        let iter_t = TableIter::new(&mut iter);
                        func(iter_t, i, tuple);
                    }
                } else {
                    for i in 0..iter_count {
                        let tuple = components_data.get_tuple_with_ref(i);
                        let iter_t = TableIter::new(&mut iter);
                        func(iter_t, i, tuple);
                    }
                }

                sys::ecs_table_unlock(world_ptr, iter.table);
            }
        }
    }

    /// find iterator to find an entity
    /// The "find" iterator accepts a function that is invoked for each matching entity and checks if the condition is true.
    /// if it is, it returns that entity.
    /// The following function signatures is valid:
    ///  - func(comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// # Returns
    ///
    /// * `Some(EntityView<'_>)` if the entity was found, `None` if no entity was found.
    fn find(&self, mut func: impl FnMut(T::TupleType<'_>) -> bool) -> Option<EntityView<'a>> {
        unsafe {
            let mut iter = self.retrieve_iter();
            let mut entity: Option<EntityView> = None;
            let world_ptr = self.world_ptr_mut();

            while self.iter_next(&mut iter) {
                let world = self.world();
                let (is_any_array, mut components_data) = T::create_ptrs(&iter);
                let iter_count = iter.count as usize;

                sys::ecs_table_lock(world_ptr, iter.table);

                if !is_any_array.a_ref && !is_any_array.a_row {
                    for i in 0..iter_count {
                        let tuple = components_data.get_tuple(i);
                        if func(tuple) {
                            entity = Some(EntityView::new_from(world, *iter.entities.add(i)));
                            break;
                        }
                    }
                } else if is_any_array.a_row {
                    for i in 0..iter_count {
                        let tuple = components_data.get_tuple_with_row(&iter, i);
                        if func(tuple) {
                            entity = Some(EntityView::new_from(world, *iter.entities.add(i)));
                            break;
                        }
                    }
                } else {
                    for i in 0..iter_count {
                        let tuple = components_data.get_tuple_with_ref(i);
                        if func(tuple) {
                            entity = Some(EntityView::new_from(world, *iter.entities.add(i)));
                            break;
                        }
                    }
                }

                sys::ecs_table_unlock(world_ptr, iter.table);
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
    /// # Returns
    ///
    /// * `Some(EntityView<'_>)` if the entity was found, `None` if no entity was found.
    fn find_entity(
        &self,
        mut func: impl FnMut(EntityView, T::TupleType<'_>) -> bool,
    ) -> Option<EntityView<'a>> {
        unsafe {
            let mut iter = self.retrieve_iter();
            let mut entity_result: Option<EntityView> = None;
            let world_ptr = self.world_ptr_mut();

            while self.iter_next(&mut iter) {
                let world = self.world();
                let (is_any_array, mut components_data) = T::create_ptrs(&iter);
                let iter_count = iter.count as usize;

                sys::ecs_table_lock(world_ptr, iter.table);

                if !is_any_array.a_ref && !is_any_array.a_row {
                    for i in 0..iter_count {
                        let entity = EntityView::new_from(world, *iter.entities.add(i));

                        let tuple = components_data.get_tuple(i);
                        if func(entity, tuple) {
                            entity_result = Some(entity);
                            break;
                        }
                    }
                } else if is_any_array.a_row {
                    for i in 0..iter_count {
                        let entity = EntityView::new_from(world, *iter.entities.add(i));
                        let tuple = components_data.get_tuple_with_row(&iter, i);
                        if func(entity, tuple) {
                            entity_result = Some(entity);
                            break;
                        }
                    }
                } else {
                    // is_any_array.a_ref
                    for i in 0..iter_count {
                        let entity = EntityView::new_from(world, *iter.entities.add(i));
                        let tuple = components_data.get_tuple_with_ref(i);
                        if func(entity, tuple) {
                            entity_result = Some(entity);
                            break;
                        }
                    }
                }

                sys::ecs_table_unlock(world_ptr, iter.table);
            }
            entity_result
        }
    }

    /// find iterator to find an entity.
    /// The "find" iterator accepts a function that is invoked for each matching entity and checks if the condition is true.
    /// if it is, it returns that entity.
    /// The following function signatures is valid:
    ///  - func(iter : `TableIter`, index : usize, comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// # Returns
    ///
    /// * `Some(EntityView<'_>)` if the entity was found, `None` if no entity was found.
    fn find_iter(
        &self,
        mut func: impl FnMut(TableIter<false, P>, usize, T::TupleType<'_>) -> bool,
    ) -> Option<EntityView<'a>>
    where
        P: ComponentId,
    {
        unsafe {
            let mut iter = self.retrieve_iter();
            let mut entity_result: Option<EntityView> = None;
            let world_ptr = self.world_ptr_mut();

            while self.iter_next(&mut iter) {
                let world = self.world();
                let (is_any_array, mut components_data) = T::create_ptrs(&iter);
                let iter_count = {
                    if iter.count == 0 {
                        1_usize
                    } else {
                        iter.count as usize
                    }
                };

                sys::ecs_table_lock(world_ptr, iter.table);

                if !is_any_array.a_ref && !is_any_array.a_row {
                    for i in 0..iter_count {
                        let tuple = components_data.get_tuple(i);
                        let iter_t = TableIter::new(&mut iter);

                        if func(iter_t, i, tuple) {
                            entity_result =
                                Some(EntityView::new_from(world, *iter.entities.add(i)));
                            break;
                        }
                    }
                } else if is_any_array.a_row {
                    for i in 0..iter_count {
                        let tuple = components_data.get_tuple_with_row(&iter, i);
                        let iter_t = TableIter::new(&mut iter);

                        if func(iter_t, i, tuple) {
                            entity_result =
                                Some(EntityView::new_from(world, *iter.entities.add(i)));
                            break;
                        }
                    }
                } else {
                    for i in 0..iter_count {
                        let tuple = components_data.get_tuple_with_ref(i);
                        let iter_t = TableIter::new(&mut iter);

                        if func(iter_t, i, tuple) {
                            entity_result =
                                Some(EntityView::new_from(world, *iter.entities.add(i)));
                            break;
                        }
                    }
                }

                sys::ecs_table_unlock(world_ptr, iter.table);
            }
            entity_result
        }
    }

    /// Run iterator.
    ///
    /// The "run" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    ///  - func(it: &mut `TableIter`)
    ///
    /// allows for more control over how entities
    /// are iterated as it provides multiple entities in the same callback
    /// and allows to determine what should happen before and past iteration.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Tag;
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Position {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Velocity {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world.entity().add(Tag).add(Position::id());
    /// world.entity().add(Tag).add(Position::id());
    /// world
    ///     .entity()
    ///     .add(Tag)
    ///     .add(Position::id())
    ///     .add(Velocity::id());
    ///
    /// let query = world.new_query::<(&Tag, &Position)>();
    ///
    /// let mut count_tables = 0;
    /// let mut count_entities = 0;
    ///
    /// query.run(|mut it| {
    ///     println!("start operations");
    ///     while it.next() {
    ///         count_tables += 1;
    ///         let pos = it.field::<Position>(1).unwrap(); //at index 1 in (&Tag, &Position)
    ///         for i in it.iter() {
    ///             count_entities += 1;
    ///             let entity = it.entity(i).unwrap();
    ///             println!("{:?}: {:?}", entity, pos[i]);
    ///         }
    ///     }
    ///     println!("end operations");
    /// });
    ///
    /// assert_eq!(count_tables, 2);
    /// assert_eq!(count_entities, 3);
    ///
    /// // Output:
    /// //  start operations
    /// //  Entity name:  -- id: 508 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position: Position { x: 0, y: 0 }
    /// //  Entity name:  -- id: 511 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position: Position { x: 0, y: 0 }
    /// //  Entity name:  -- id: 512 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position, flecs_ecs.main.Velocity: Position { x: 0, y: 0 }
    /// //  end operations
    /// ```
    fn run(&self, mut func: impl FnMut(TableIter<true, P>))
    where
        P: ComponentId,
    {
        let mut iter = self.retrieve_iter();
        iter.flags &= !sys::EcsIterIsValid;
        let iter_t = unsafe { TableIter::new(&mut iter) };
        func(iter_t);
    }

    /// Run iterator with each forwarding.
    /// The "iter" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    ///  - `func`: (it: &mut `TableIter`) + `func_each`: (comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// allows for more control over how entities
    /// are iterated as it provides multiple entities in the same callback
    /// and allows to determine what should happen before and past iteration.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Tag;
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Position {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Velocity {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world.entity().add(Tag).add(Position::id());
    /// world.entity().add(Tag).add(Position::id());
    /// world
    ///     .entity()
    ///     .add(Tag)
    ///     .add(Position::id())
    ///     .add(Velocity::id());
    ///
    /// let query = world.query::<(&Position)>().with(Tag).build();
    ///
    /// let mut count_tables = 0;
    /// let mut count_entities = 0;
    ///
    /// query.run_each(
    ///     |mut it| {
    ///         println!("start operations");
    ///         while it.next() {
    ///             count_tables += 1;
    ///             it.each();
    ///         }
    ///         println!("end operations");
    ///     },
    ///     |pos| {
    ///         count_entities += 1;
    ///         println!("{:?}", pos);
    ///     },
    /// );
    ///
    /// assert_eq!(count_tables, 2);
    /// assert_eq!(count_entities, 3);
    ///
    /// // Output:
    /// //  start operations
    /// //  Tag, Position { x: 0, y: 0 }
    /// //  Tag, Position { x: 0, y: 0 }
    /// //  Tag, Position { x: 0, y: 0 }
    /// //  end operations
    /// ```
    fn run_each<FuncEach>(&self, mut func: impl FnMut(TableIter<true, P>), mut func_each: FuncEach)
    where
        P: ComponentId,
        FuncEach: FnMut(T::TupleType<'_>),
    {
        let mut iter = self.retrieve_iter();
        iter.callback_ctx = &mut func_each as *mut _ as *mut core::ffi::c_void;
        iter.callback = Some(
            __internal_query_execute_each::<T, FuncEach>
                as unsafe extern "C" fn(*mut sys::ecs_iter_t),
        );
        let mut iter_t = unsafe { TableIter::new(&mut iter) };
        iter_t.iter_mut().flags &= !sys::EcsIterIsValid;
        func(iter_t);
        iter.callback = None;
        iter.callback_ctx = core::ptr::null_mut();
    }

    /// Run iterator with each entity forwarding.
    ///
    /// The "iter" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    /// - `func`: (it: &mut `TableIter`) + `func_each`: (entity: Entity, comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// allows for more control over how entities
    /// are iterated as it provides multiple entities in the same callback
    /// and allows to determine what should happen before and past iteration.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Tag;
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Position {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Velocity {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world.entity().add(Tag).add(Position::id());
    /// world.entity().add(Tag).add(Position::id());
    /// world
    ///     .entity()
    ///     .add(Tag)
    ///     .add(Position::id())
    ///     .add(Velocity::id());
    ///
    /// let query = world.query::<(&Position)>().with(Tag).build();
    ///
    /// let mut count_tables = 0;
    /// let mut count_entities = 0;
    ///
    /// query.run_each_entity(
    ///     |mut it| {
    ///         println!("start operations");
    ///         while it.next() {
    ///             count_tables += 1;
    ///             it.each();
    ///         }
    ///         println!("end operations");
    ///     },
    ///     |e, pos| {
    ///         count_entities += 1;
    ///         println!("{:?} : {:?}", e, pos);
    ///     },
    /// );
    ///
    /// assert_eq!(count_tables, 2);
    /// assert_eq!(count_entities, 3);
    ///
    /// // Output:
    /// //  start operations
    /// //  Entity name:  -- id: 508 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position : Position { x: 0, y: 0 }
    /// //  Entity name:  -- id: 511 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position : Position { x: 0, y: 0 }
    /// //  Entity name:  -- id: 512 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position, flecs_ecs.main.Velocity : Position { x: 0, y: 0 }
    /// //  end operations
    /// ```
    fn run_each_entity<FuncEachEntity>(
        &self,
        mut func: impl FnMut(TableIter<true, P>),
        mut func_each: FuncEachEntity,
    ) where
        P: ComponentId,
        FuncEachEntity: FnMut(EntityView, T::TupleType<'_>),
    {
        let mut iter = self.retrieve_iter();
        iter.callback_ctx = &mut func_each as *mut _ as *mut core::ffi::c_void;
        iter.callback = Some(
            __internal_query_execute_each_entity::<T, FuncEachEntity>
                as unsafe extern "C" fn(*mut sys::ecs_iter_t),
        );
        let mut iter_t = unsafe { TableIter::new(&mut iter) };
        iter_t.iter_mut().flags &= !sys::EcsIterIsValid;
        func(iter_t);
        iter.callback = None;
        iter.callback_ctx = core::ptr::null_mut();
    }

    /// Get the entity of the current query
    ///
    /// # Arguments
    ///
    /// * `query`: the query to get the entity from
    ///
    /// # Returns
    ///
    /// The entity of the current query
    fn entity(&self) -> EntityView;

    /// Each term iterator.
    /// The `each_term` iterator accepts a function that is invoked for each term
    /// in the query. The following function signature is valid:
    ///  - func(term: &mut Term)
    fn each_term(&self, mut func: impl FnMut(&TermRef)) {
        let query = self.query_ptr();
        ecs_assert!(
            !query.is_null(),
            FlecsErrorCode::InvalidParameter,
            "query filter is null"
        );
        let query = unsafe { &*query };
        for i in 0..query.term_count {
            let term = TermRef::new(unsafe { &*(query.terms.add(i as usize) as *const _) });
            func(&term);
        }
    }

    /// Get a immutable reference of the term of the current query at the given index
    /// This is mostly used for debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `index`: the index of the term to get
    /// * `query`: the query to get the term from
    ///
    /// # Returns
    ///
    /// The term requested
    fn term(&self, index: usize) -> TermRef<'_> {
        let query = self.query_ptr();
        ecs_assert!(
            !query.is_null(),
            FlecsErrorCode::InvalidParameter,
            "query filter is null"
        );
        let query = unsafe { &*query };
        TermRef::new(unsafe { &*(query.terms.add(index) as *const _) })
    }

    /// Get the field count of the current query
    ///
    /// # Arguments
    ///
    /// * `query`: the query to get the field count from
    ///
    /// # Returns
    ///
    /// The field count of the current query
    fn field_count(&self) -> i8 {
        let query = self.query_ptr();
        unsafe { (*query).field_count }
    }

    /// Get the count of terms set of the current query
    fn term_count(&self) -> u32 {
        let query = self.query_ptr();
        unsafe { (*query).term_count as u32 }
    }

    /// Convert query to string expression. Convert query terms to a string expression.
    /// The resulting expression can be parsed to create the same query.
    ///
    /// # Arguments
    ///
    /// * `query`: the query to convert to a string
    ///
    /// # Returns
    ///
    /// The string representation of the query
    #[allow(clippy::inherent_to_string)] // this is a wrapper around a c function
    fn to_string(&self) -> String {
        let query = self.query_ptr();
        let result: *mut c_char = unsafe { sys::ecs_query_str(query as *const _) };
        let rust_string =
            String::from(unsafe { core::ffi::CStr::from_ptr(result).to_str().unwrap() });
        unsafe {
            if let Some(free_func) = sys::ecs_os_api.free_ {
                free_func(result as *mut _);
            }
        }
        rust_string
    }

    fn find_var(&self, name: &str) -> Option<i32> {
        let name = compact_str::format_compact!("{}\0", name);

        let var_index =
            unsafe { sys::ecs_query_find_var(self.query_ptr(), name.as_ptr() as *const _) };
        if var_index == -1 {
            None
        } else {
            Some(var_index)
        }
    }

    fn plan(&self) -> String {
        let query = self.query_ptr();
        let result: *mut c_char = unsafe { sys::ecs_query_plan(query as *const _) };
        let rust_string =
            String::from(unsafe { core::ffi::CStr::from_ptr(result).to_str().unwrap() });
        unsafe {
            if let Some(free_func) = sys::ecs_os_api.free_ {
                free_func(result as *mut _);
            }
        }
        rust_string
    }

    fn iterable(&self) -> QueryIter<P, T> {
        QueryIter::new(self.retrieve_iter(), self.iter_next_func())
    }

    fn iter_stage(&'a self, stage: impl WorldProvider<'a>) -> QueryIter<'a, P, T> {
        QueryIter::new(self.retrieve_iter_stage(stage), self.iter_next_func())
    }

    /// Return first matching entity.
    ///
    /// # Returns
    ///
    /// * `Some(EntityView<'_>)` if the entity was found, `None` if no entity was found.
    ///
    /// # See also
    ///
    ///
    /// ```rust
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Position {
    ///     x: i32,
    ///     y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let query = world.query::<&Position>().build();
    ///
    /// let entity = query.try_first_entity();
    ///
    /// assert!(entity.is_none());
    ///
    /// let ent = world.entity().set(Position { x: 10, y: 20 });
    ///
    /// let entity = query.try_first_entity();
    ///
    /// assert_eq!(entity.unwrap(), ent);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Query::try_first_entity`]
    /// * [`Query::try_first`]
    /// * [`Query::first`]
    /// * [`Query::try_first_only`]
    /// * [`Query::first_only`]
    fn try_first_entity(&self) -> Option<EntityView<'a>> {
        let it = &mut self.retrieve_iter();

        if self.iter_next(it) && it.count > 0 {
            let ent = Some(EntityView::new_from(self.world(), unsafe {
                *it.entities.add(0)
            }));
            unsafe { sys::ecs_iter_fini(it) };
            ent
        } else {
            //unsafe { sys::ecs_iter_fini(it) };
            None
        }
    }

    /// Return first matching entity.
    ///
    /// # Returns
    ///
    /// The first entity in the iterator.
    ///
    /// # Panics
    ///
    /// if there are no entities.
    ///
    /// # See also
    ///
    ///
    /// ```rust
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Position {
    ///     x: i32,
    ///     y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let query = world.query::<&Position>().build();
    ///
    /// let ent = world.entity().set(Position { x: 10, y: 20 });
    ///
    /// let entity = query.first_entity();
    ///
    /// assert_eq!(entity, ent);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Query::try_first_entity`]
    /// * [`Query::try_first`]
    /// * [`Query::first`]
    /// * [`Query::try_first_only`]
    /// * [`Query::first_only`]
    fn first_entity(&self) -> EntityView<'a> {
        self.try_first_entity()
            .expect("Expected at least one entity, but none were found.")
    }

    /// iterates over the first entity in the iterator and returns a user-defined result.
    ///
    /// # Returns
    ///
    /// * `Some(result)` if there is at least one entity, otherwise returns `None`.
    ///
    /// ```rust
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Position {
    ///     x: i32,
    ///     y: i32,
    /// }
    ///
    /// let world = World::new();
    /// let query = world.new_query::<&Position>();
    ///
    /// assert_eq!(query.try_first(|pos| pos.x), None);
    ///
    /// world.entity().set(Position { x: 10, y: 20 });
    /// world.entity().set(Position { x: 40, y: 20 });
    ///
    /// let x_pos = query.try_first(|pos| pos.x);
    ///
    /// assert_eq!(x_pos, Some(10));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Query::try_first_entity`]
    /// * [`Query::first_entity`]
    /// * [`Query::first`]
    /// * [`Query::try_first_only`]
    /// * [`Query::first_only`]
    fn try_first<R>(&self, func: impl FnOnce(T::TupleType<'_>) -> R) -> Option<R> {
        let mut it = self.retrieve_iter();

        // Proceed only if there is at least one entity in the iterator
        if self.iter_next(&mut it) && it.count > 0 {
            let (is_any_array, mut components_data) = T::create_ptrs(&it);
            let tuple = if !is_any_array.a_row && !is_any_array.a_ref {
                components_data.get_tuple(0)
            } else if is_any_array.a_row {
                components_data.get_tuple_with_row(&it, 0)
            } else {
                components_data.get_tuple_with_ref(0)
            };

            let result = Some(func(tuple));
            // Clean up iterator resources safely
            unsafe { sys::ecs_iter_fini(&mut it) };

            result
        } else {
            None
        }
    }

    /// iterates over the first entity in the iterator and returns a user-defined result.
    ///
    /// # Returns
    ///
    /// The result of the callback function.
    ///
    /// # Panics
    ///
    /// if there are no entities.
    ///
    /// ```rust
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Position {
    ///     x: i32,
    ///     y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let query = world.new_query::<&Position>();
    ///
    /// world.entity().set(Position { x: 10, y: 20 });
    ///
    /// let x_pos = query.first(|pos| pos.x);
    ///
    /// assert_eq!(x_pos, 10);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Query::try_first_entity`]
    /// * [`Query::first_entity`]
    /// * [`Query::try_first`]
    /// * [`Query::try_first_only`]
    /// * [`Query::first_only`]
    fn first<R>(&self, func: impl FnOnce(T::TupleType<'_>) -> R) -> R {
        self.try_first(func)
            .expect("Expected at least one entity, but none were found.")
    }

    /// Iterates over the first entity in the iterator and returns a user-defined result.
    ///
    /// # Returns
    ///
    /// * `Ok(result)` if there is exactly one entity.
    /// * `Err(FirstOnlyError)` if there are no entities or more than one entity.
    ///
    /// ```rust
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Position {
    ///     x: i32,
    ///     y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let query = world.new_query::<&Position>();
    ///
    /// assert_eq!(
    ///     query.try_first_only(|pos| pos.x),
    ///     Err(FirstOnlyError::NoEntities)
    /// );
    ///
    /// world.entity().set(Position { x: 10, y: 20 });
    ///
    /// let x_pos = query.try_first_only(|pos| pos.x);
    ///
    /// assert_eq!(x_pos, Ok(10));
    ///
    /// world.entity().set(Position { x: 40, y: 20 });
    ///
    /// let x_pos = query.try_first_only(|pos| pos.x);
    ///
    /// assert_eq!(x_pos, Err(FirstOnlyError::MoreThanOneEntity));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Query::try_first_entity`]
    /// * [`Query::first_entity`]
    /// * [`Query::try_first`]
    /// * [`Query::first`]
    fn try_first_only<R>(
        &self,
        func: impl FnOnce(T::TupleType<'_>) -> R,
    ) -> Result<R, FirstOnlyError> {
        let mut it = self.retrieve_iter();

        // Proceed only if we can iterate
        if self.iter_next(&mut it) {
            if it.count == 1 {
                let (is_any_array, mut components_data) = T::create_ptrs(&it);

                let tuple = if !is_any_array.a_row && !is_any_array.a_ref {
                    components_data.get_tuple(0)
                } else if is_any_array.a_row {
                    components_data.get_tuple_with_row(&it, 0)
                } else {
                    components_data.get_tuple_with_ref(0)
                };

                // Clean up iterator resources safely
                let result = func(tuple);
                unsafe { sys::ecs_iter_fini(&mut it) };

                Ok(result)
            } else {
                // More than one entity
                unsafe { sys::ecs_iter_fini(&mut it) };
                Err(FirstOnlyError::MoreThanOneEntity)
            }
        } else {
            // No entities in the iterator
            Err(FirstOnlyError::NoEntities)
        }
    }

    /// iterates over the first entity in the iterator and returns a user-defined result.
    ///
    /// # Returns
    ///
    /// The result of the callback function.
    ///
    /// # Panics
    ///  
    /// if there are no entities, or if there are more than one entity.
    ///
    /// ```rust
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Position {
    ///     x: i32,
    ///     y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let query = world.new_query::<&Position>();
    ///
    /// world.entity().set(Position { x: 10, y: 20 });
    ///
    /// let x_pos = query.first_only(|pos| pos.x);
    ///
    /// assert_eq!(x_pos, 10);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Query::try_first_entity`]
    /// * [`Query::first_entity`]
    /// * [`Query::try_first`]
    /// * [`Query::first`]
    /// * [`Query::try_first_only`]
    fn first_only<R>(&self, func: impl FnOnce(T::TupleType<'_>) -> R) -> R {
        match self.try_first_only(func) {
            Ok(result) => result,
            Err(FirstOnlyError::MoreThanOneEntity) => {
                panic!("Expected exactly one entity, but found more than one.");
            }
            Err(FirstOnlyError::NoEntities) => {
                panic!("Expected one entity, but none were found.");
            }
        }
    }

    /// Returns true if iterator yields at least once result.
    fn is_true(&mut self) -> bool {
        let mut it = self.retrieve_iter();

        let result = self.iter_next(&mut it);
        if result {
            unsafe { sys::ecs_iter_fini(&mut it) };
        }
        result
    }

    /// Return total number of entities in result.
    ///
    /// # Returns
    ///
    /// The total number of entities in the result
    fn count(&self) -> i32 {
        let mut it = self.retrieve_iter();
        let mut result = 0;
        while self.iter_next(&mut it) {
            result += it.count;
        }
        result
    }

    /// Limit results to tables with specified group id (grouped queries only)
    ///
    /// # Arguments
    ///
    /// * `group_id`: the group id to set
    fn set_group(&self, group_id: impl IntoEntity) -> QueryIter<P, T> {
        let mut iter = self.iterable();
        QueryIter::<P, T>::set_group(&mut iter, group_id);
        iter
    }

    /// set variable of iter
    ///
    /// # Arguments
    ///
    /// * `var_id`: the variable id to set
    ///
    /// * `value`: the value to set
    #[must_use = "This method returns a new query iterator that should be used"]
    fn set_var(&self, var_id: i32, value: impl Into<Entity>) -> QueryIter<P, T> {
        let mut iter = self.iterable();
        QueryIter::<P, T>::set_var(&mut iter, var_id, value);
        iter
    }

    /// set variable of iter as table
    ///
    /// # Arguments
    ///
    /// * `var_id`: the variable id to set
    ///
    /// * `range`: the range to set
    fn set_var_table(&self, var_id: i32, table: impl IntoTableRange) -> QueryIter<P, T> {
        let mut iter = self.iterable();
        QueryIter::<P, T>::set_var_table(&mut iter, var_id, table);
        iter
    }

    /// set variable for rule iter
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the variable to set
    /// * `value`: the value to set
    fn set_var_expr(&self, name: &str, value: impl Into<Entity>) -> QueryIter<P, T> {
        let mut iter = self.iterable();
        QueryIter::<P, T>::set_var_expr(&mut iter, name, value);
        iter
    }

    /// set variable for rule iter as table
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the variable to set
    /// * `range`: the range to set
    fn set_var_table_expr(&self, name: &str, table: impl IntoTableRange) -> QueryIter<P, T> {
        let mut iter = self.iterable();
        QueryIter::<P, T>::set_var_table_expr(&mut iter, name, table);
        iter
    }

    /// Serialize iterator result to JSON.
    #[cfg(feature = "flecs_json")]
    fn to_json(&self, desc: Option<&crate::prelude::json::IterToJsonDesc>) -> Option<String> {
        let desc_ptr = desc
            .map(|d| d as *const crate::prelude::json::IterToJsonDesc)
            .unwrap_or(core::ptr::null());

        let mut iter = self.retrieve_iter();

        unsafe {
            let json_ptr = sys::ecs_iter_to_json(&mut iter, desc_ptr);
            if json_ptr.is_null() {
                return None;
            }
            let json = core::ffi::CStr::from_ptr(json_ptr)
                .to_str()
                .unwrap()
                .to_string();
            sys::ecs_os_api.free_.expect("os api is missing")(json_ptr as *mut core::ffi::c_void);
            Some(json)
        }
    }

    fn cache_query(&self) -> Option<Query<()>> {
        let query = self.query_ptr();
        unsafe {
            let cache_query = sys::ecs_query_get_cache_query(query);
            if cache_query.is_null() {
                None
            } else {
                Some(Query::new_from(core::ptr::NonNull::new_unchecked(
                    cache_query as *mut _,
                )))
            }
        }
    }
}

unsafe extern "C" fn __internal_query_execute_each<T, Func>(iter: *mut sys::ecs_iter_t)
where
    T: QueryTuple,
    Func: FnMut(T::TupleType<'_>),
{
    let iter = unsafe { &mut *iter };
    unsafe {
        let func = &mut *(iter.callback_ctx as *mut Func);

        let (is_any_array, mut components_data) = T::create_ptrs(iter);
        let iter_count = iter.count as usize;

        if !is_any_array.a_row && !is_any_array.a_ref {
            for i in 0..iter_count {
                let tuple = components_data.get_tuple(i);
                func(tuple);
            }
        } else if is_any_array.a_row {
            for i in 0..iter_count {
                let tuple = components_data.get_tuple_with_row(iter, i);
                func(tuple);
            }
        } else if is_any_array.a_ref {
            for i in 0..iter_count {
                let tuple = components_data.get_tuple_with_ref(i);
                func(tuple);
            }
        }
    }
}

unsafe extern "C" fn __internal_query_execute_each_entity<T, Func>(iter: *mut sys::ecs_iter_t)
where
    T: QueryTuple,
    Func: FnMut(EntityView, T::TupleType<'_>),
{
    unsafe {
        let iter = &mut *iter;
        let func = &mut *(iter.callback_ctx as *mut Func);

        let (is_any_array, mut components_data) = T::create_ptrs(iter);
        let iter_count = iter.count as usize;
        let world = WorldRef::from_ptr(iter.world);

        if !is_any_array.a_row && !is_any_array.a_ref {
            for i in 0..iter_count {
                let tuple = components_data.get_tuple(i);
                func(EntityView::new_from(world, *iter.entities.add(i)), tuple);
            }
        } else if is_any_array.a_row {
            for i in 0..iter_count {
                let tuple = components_data.get_tuple_with_row(iter, i);
                func(EntityView::new_from(world, *iter.entities.add(i)), tuple);
            }
        } else if is_any_array.a_ref {
            for i in 0..iter_count {
                let tuple = components_data.get_tuple_with_ref(i);
                func(EntityView::new_from(world, *iter.entities.add(i)), tuple);
            }
        }
    }
}
