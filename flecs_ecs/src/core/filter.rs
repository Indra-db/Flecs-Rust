//! Filters are cheaper to create, but slower to iterate than queries.

use crate::sys::{
    ecs_abort_, ecs_filter_copy, ecs_filter_desc_t, ecs_filter_fini, ecs_filter_init,
    ecs_filter_iter, ecs_filter_move, ecs_filter_next, ecs_get_entity, ecs_os_api,
};

use super::{
    c_types::FilterT, entity::Entity, iterable::Iterable, world::World, FlecsErrorCode, IntoWorld,
    IterAPI, IterOperations,
};

pub struct FilterView<T>
where
    T: Iterable,
{
    world: World,
    filter_ptr: *const FilterT,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Clone for FilterView<T>
where
    T: Iterable,
{
    fn clone(&self) -> Self {
        Self {
            world: self.world.clone(),
            filter_ptr: self.filter_ptr,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> FilterView<T>
where
    T: Iterable,
{
    /// Create a new filter view
    ///
    /// # Arguments
    ///
    /// * `world`: the world to create the filter view from
    /// * `filter`: the filter to create the view from
    ///
    /// # See also
    ///
    /// * C++ API: `filter_view::filter_view`
    #[doc(alias = "filter_view::filter_view")]
    pub fn new(world: &World, filter: *const FilterT) -> Self {
        Self {
            world: world.clone(),
            _phantom: std::marker::PhantomData,
            filter_ptr: filter as *const FilterT,
        }
    }
}

/// Filters are cheaper to create, but slower to iterate than queries.
pub struct Filter<T>
where
    T: Iterable,
{
    world: World,
    _phantom: std::marker::PhantomData<T>,
    filter: FilterT,
}

impl<T> Filter<T>
where
    T: Iterable,
{
    /// Create a new filter
    ///
    /// # Arguments
    ///
    /// * `world`: the world to create the filter from
    ///
    /// # See also
    ///
    /// * C++ API: `filter::filter`
    #[doc(alias = "filter::filter")]
    pub fn new(world: &World) -> Self {
        let mut desc = ecs_filter_desc_t::default();
        T::register_ids_descriptor(world.raw_world, &mut desc);
        let mut filter: FilterT = Default::default();
        desc.storage = &mut filter;
        unsafe { ecs_filter_init(world.raw_world, &desc) };
        Filter {
            world: world.clone(),
            _phantom: std::marker::PhantomData,
            filter,
        }
    }

    /// Wrap an existing raw filter
    ///
    /// # Arguments
    ///
    /// * `world`: the world to create the filter from
    /// * `filter`: the filter to wrap
    ///
    /// # See also
    ///
    /// * C++ API: `filter::filter`
    #[doc(alias = "filter::filter")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new_ownership(world: &World, filter: *mut FilterT) -> Self {
        let mut filter_obj = Filter {
            world: world.clone(),
            _phantom: std::marker::PhantomData,
            filter: Default::default(),
        };

        unsafe { ecs_filter_move(&mut filter_obj.filter, filter) };

        filter_obj
    }

    //TODO: this needs testing -> desc.storage pointer becomes invalid after this call as it re-allocates after this new
    // determine if this is a problem
    /// Create a new filter from a filter descriptor
    ///
    /// # Arguments
    ///
    /// * `world`: the world to create the filter from
    /// * `desc`: the filter descriptor to create the filter from
    ///
    /// # See also
    ///
    /// * C++ API: `filter::filter`
    #[doc(alias = "filter::filter")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new_from_desc(world: &World, desc: *mut ecs_filter_desc_t) -> Self {
        let mut filter_obj = Filter {
            world: world.clone(),
            _phantom: std::marker::PhantomData,
            filter: Default::default(),
        };

        unsafe {
            (*desc).storage = &mut filter_obj.filter;
        }

        unsafe {
            if ecs_filter_init(filter_obj.world.raw_world, desc).is_null() {
                ecs_abort_(
                    FlecsErrorCode::InvalidParameter.to_int(),
                    file!().as_ptr() as *const i8,
                    line!() as i32,
                    std::ptr::null(),
                );

                if let Some(abort_func) = ecs_os_api.abort_ {
                    abort_func();
                }
            }

            if !(*desc).terms_buffer.is_null() {
                if let Some(free_func) = ecs_os_api.free_ {
                    free_func((*desc).terms_buffer as *mut _);
                }
            }
        }

        filter_obj
    }
}

impl<T> Drop for Filter<T>
where
    T: Iterable,
{
    fn drop(&mut self) {
        // this is a hack to prevent ecs_filter_fini from freeing the memory of our stack allocated filter
        // we do actually own this filter. ecs_filter_fini is called to free the memory of the terms
        //self.filter.owned = false;
        //TODO the above code, `.owned` got removed in upgrading flecs from 3.2.4 to 3.2.11,
        // so we need to find a new? way to prevent the memory from being freed if it's stack allocated
        unsafe { ecs_filter_fini(&mut self.filter) }
    }
}

impl<T> Clone for Filter<T>
where
    T: Iterable,
{
    fn clone(&self) -> Self {
        let mut new_filter = Filter::<T> {
            world: self.world.clone(),
            _phantom: std::marker::PhantomData,
            filter: Default::default(),
        };

        unsafe { ecs_filter_copy(&mut new_filter.filter, &self.filter) };
        new_filter
    }
}

impl<T> IntoWorld for Filter<T>
where
    T: Iterable,
{
    fn world_ptr_mut(&self) -> *mut super::WorldT {
        self.world.raw_world
    }
}

impl<T> IterOperations for Filter<T>
where
    T: Iterable,
{
    fn retrieve_iter(&self) -> super::IterT {
        unsafe { ecs_filter_iter(self.world.raw_world, &self.filter) }
    }

    fn iter_next(&self, iter: &mut super::IterT) -> bool {
        unsafe { ecs_filter_next(iter) }
    }

    fn filter_ptr(&self) -> *const FilterT {
        &self.filter
    }

    fn iter_next_func(&self) -> unsafe extern "C" fn(*mut super::IterT) -> bool {
        ecs_filter_next
    }
}

impl<T> IntoWorld for FilterView<T>
where
    T: Iterable,
{
    fn world_ptr_mut(&self) -> *mut super::WorldT {
        self.world.raw_world
    }
}

impl<T> IterAPI<T> for FilterView<T>
where
    T: Iterable,
{
    fn as_entity(&self) -> Entity {
        Entity::new_from_existing_raw(&self.world, unsafe {
            ecs_get_entity(self.filter_ptr as *const _)
        })
    }
}

impl<T> IterOperations for FilterView<T>
where
    T: Iterable,
{
    fn retrieve_iter(&self) -> super::IterT {
        unsafe { ecs_filter_iter(self.world.raw_world, self.filter_ptr) }
    }

    fn iter_next(&self, iter: &mut super::IterT) -> bool {
        unsafe { ecs_filter_next(iter) }
    }

    fn filter_ptr(&self) -> *const FilterT {
        self.filter_ptr
    }

    fn iter_next_func(&self) -> unsafe extern "C" fn(*mut super::IterT) -> bool {
        ecs_filter_next
    }
}

impl<T> IterAPI<T> for Filter<T>
where
    T: Iterable,
{
    fn as_entity(&self) -> Entity {
        Entity::new_from_existing_raw(&self.world, unsafe {
            ecs_get_entity(&self.filter as *const _ as *const _)
        })
    }
}
