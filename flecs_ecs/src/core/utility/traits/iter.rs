use std::ffi::c_char;

#[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
use crate::core::FlecsErrorCode;
use crate::{
    core::{ComponentPointers, Entity, FilterT, Iter, IterIterable, IterT, Iterable, Term},
    ecs_assert,
};
use flecs_ecs_sys::{ecs_filter_str, ecs_iter_fini, ecs_os_api, ecs_table_lock, ecs_table_unlock};

use super::IntoWorld;

pub trait IterOperations {
    #[doc(hidden)]
    fn retrieve_iter(&self) -> IterT;

    #[doc(hidden)]
    fn iter_next(&self, iter: &mut IterT) -> bool;

    #[doc(hidden)]
    fn iter_next_func(&self) -> unsafe extern "C" fn(*mut IterT) -> bool;

    #[doc(hidden)]
    fn filter_ptr(&self) -> *const FilterT;
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

            while self.iter_next(&mut iter) {
                let mut components_data = T::create_ptrs(&iter);
                let iter_count = iter.count as usize;

                ecs_table_lock(self.world_ptr_mut(), iter.table);

                for i in 0..iter_count {
                    let tuple = components_data.get_tuple(i);
                    func(tuple);
                }

                ecs_table_unlock(self.world_ptr_mut(), iter.table);
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
            let world = self.world_ptr_mut();
            while self.iter_next(&mut iter) {
                let mut components_data = T::create_ptrs(&iter);
                let iter_count = {
                    if iter.count == 0 {
                        1_usize
                    } else {
                        iter.count as usize
                    }
                };

                ecs_table_lock(world, iter.table);

                // TODO random thought, I think I can determine the elements is a ref or not before the for loop and then pass two arrays with the indices of the ref and non ref elements
                // I will come back to this in the future, my thoughts are somewhere else right now. If my assumption is correct, this will get rid of the branch in the for loop
                // and potentially allow for more conditions for vectorization to happen. This could potentially offer a (small) performance boost since the branch predictor avoids probably
                // most of the cost since the branch is almost always the same.
                // update: I believe it's not possible due to not knowing the order of the components in the tuple. I will leave this here for now, maybe I will come back to it in the future.
                for i in 0..iter_count {
                    let mut entity = Entity::new_from_existing_raw(world, *iter.entities.add(i));
                    let tuple = components_data.get_tuple(i);

                    func(&mut entity, tuple);
                }

                ecs_table_unlock(world, iter.table);
            }
        }
    }

    fn each_iter(&self, mut func: impl FnMut(&mut Iter, usize, T::TupleType)) {
        unsafe {
            let mut iter = self.retrieve_iter();
            let world = self.world_ptr_mut();

            while self.iter_next(&mut iter) {
                let mut components_data = T::create_ptrs(&iter);
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
                    let tuple = components_data.get_tuple(i);

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
            let world = self.world_ptr_mut();

            while self.iter_next(&mut iter) {
                let mut components_data = T::create_ptrs(&iter);
                let iter_count = iter.count as usize;

                ecs_table_lock(world, iter.table);

                for i in 0..iter_count {
                    let tuple = components_data.get_tuple(i);
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
            let world = self.world_ptr_mut();

            while self.iter_next(&mut iter) {
                let mut components_data = T::create_ptrs(&iter);
                let iter_count = iter.count as usize;

                ecs_table_lock(world, iter.table);

                for i in 0..iter_count {
                    let mut entity =
                        Entity::new_from_existing_raw(iter.world, *iter.entities.add(i));

                    let tuple = components_data.get_tuple(i);
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
            let world = self.world_ptr_mut();

            while self.iter_next(&mut iter) {
                let mut components_data = T::create_ptrs(&iter);
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
                    let tuple = components_data.get_tuple(i);
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
            let world = self.world_ptr_mut();

            while self.iter_next(&mut iter) {
                let mut components_data = T::create_ptrs(&iter);
                let iter_count = iter.count as usize;

                ecs_table_lock(world, iter.table);

                let tuple = components_data.get_slice(iter_count);
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
            let world = self.world_ptr_mut();
            while self.iter_next(&mut iter) {
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
        let filter = self.filter_ptr();
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
        let filter = self.filter_ptr();
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
        let filter = self.filter_ptr();
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
        let filter = self.filter_ptr();
        let world = self.world_ptr_mut();
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

    fn iterable(&self) -> IterIterable<'a, T> {
        IterIterable::new(self.retrieve_iter(), self.iter_next_func())
    }

    /// Return first matching entity.
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::first`
    /// * C++ API: `iter_iterable::first`
    #[doc(alias = "iterable::first")]
    #[doc(alias = "iter_iterable::first")]
    fn first(&mut self) -> Entity {
        let mut entity = Entity::default();

        let world = self.world_ptr_mut();
        let it = &mut self.retrieve_iter();

        if self.iter_next(it) && it.count > 0 {
            entity = Entity::new_from_existing_raw(world, unsafe { *it.entities.add(0) });
            unsafe { ecs_iter_fini(it) };
        }
        entity
    }

    /// Returns true if iterator yields at least once result.
    fn is_true(&mut self) -> bool {
        let mut it = self.retrieve_iter();

        let result = self.iter_next(&mut it);
        if result {
            unsafe { ecs_iter_fini(&mut it) };
        }
        result
    }

    /// Return total number of entities in result.
    ///
    /// # Returns
    ///
    /// The total number of entities in the result
    ///
    /// # See also
    ///
    /// * C++ API: `iter_iterable::count`
    #[doc(alias = "iter_iterable::count")]
    fn count(&mut self) -> i32 {
        let mut it = self.retrieve_iter();
        let mut result = 0;
        while self.iter_next(&mut it) {
            result += it.count;
        }
        result
    }
}
