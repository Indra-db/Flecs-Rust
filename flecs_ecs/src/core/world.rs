use std::ffi::CStr;
use std::ops::Deref;
use std::os::raw::c_void;

#[cfg(feature = "flecs_system")]
use crate::addons::system::system_builder::SystemBuilder;
#[cfg(feature = "flecs_system")]
use crate::addons::System;

#[cfg(feature = "flecs_pipeline")]
use crate::addons::PipelineBuilder;

use crate::core::c_binding::bindings::{_ecs_poly_is, ecs_stage_t_magic, ecs_world_t_magic};
use crate::core::utility::errors::FlecsErrorCode;
use crate::core::utility::functions::ecs_pair;

use crate::ecs_assert;

use super::c_binding::bindings::{
    ecs_async_stage_free, ecs_async_stage_new, ecs_atfini, ecs_count_id, ecs_defer_begin,
    ecs_defer_end, ecs_defer_resume, ecs_defer_suspend, ecs_delete_with, ecs_dim,
    ecs_enable_range_check, ecs_ensure, ecs_exists, ecs_fini, ecs_fini_action_t, ecs_frame_begin,
    ecs_frame_end, ecs_get_alive, ecs_get_context, ecs_get_name, ecs_get_scope, ecs_get_stage,
    ecs_get_stage_count, ecs_get_stage_id, ecs_get_world, ecs_get_world_info, ecs_init,
    ecs_is_alive, ecs_is_deferred, ecs_is_valid, ecs_lookup_path_w_sep, ecs_merge, ecs_quit,
    ecs_readonly_begin, ecs_readonly_end, ecs_remove_all, ecs_run_post_frame, ecs_set_alias,
    ecs_set_automerge, ecs_set_context, ecs_set_entity_range, ecs_set_lookup_path, ecs_set_scope,
    ecs_set_stage_count, ecs_set_with, ecs_should_quit, ecs_stage_is_async, ecs_stage_is_readonly,
    ecs_system_desc_t,
};
use super::c_types::{EntityT, IdT, WorldT, SEPARATOR};
use super::component::{Component, UntypedComponent};
use super::component_ref::Ref;
use super::component_registration::{
    register_entity_w_component_explicit, CachedComponentData, ComponentType, Enum, Struct,
};

use super::entity::Entity;
use super::enum_type::CachedEnumData;
use super::event::EventData;
use super::event_builder::{EventBuilder, EventBuilderTyped};
use super::id::{Id, IdType};
use super::iterable::Iterable;
use super::observer::Observer;
use super::observer_builder::ObserverBuilder;
use super::scoped_world::ScopedWorld;
use super::term::Term;
use super::utility::functions::set_helper;

pub struct World {
    pub raw_world: *mut WorldT,
    pub is_owned: bool,
}

impl Clone for World {
    fn clone(&self) -> Self {
        World {
            raw_world: self.raw_world,
            // Set is_owned to false to prevent double free, meaning that the new world is not owned.
            is_owned: false,
        }
    }
}

impl Default for World {
    fn default() -> Self {
        let world = Self {
            raw_world: unsafe { ecs_init() },
            is_owned: true,
        };
        world.init_builtin_components();
        world
    }
}

impl Deref for World {
    type Target = *mut WorldT;

    fn deref(&self) -> &Self::Target {
        &self.raw_world
    }
}

impl Drop for World {
    fn drop(&mut self) {
        if self.is_owned && unsafe { ecs_stage_is_async(self.raw_world) } {
            unsafe { ecs_async_stage_free(self.raw_world) }
        } else if self.is_owned && !self.raw_world.is_null() {
            unsafe { ecs_fini(self.raw_world) };
        }
    }
}

impl World {
    pub fn new_from_world(world: *mut WorldT) -> Self {
        let world = Self {
            raw_world: world,
            is_owned: true,
        };
        world.init_builtin_components();
        world
    }

    fn init_builtin_components(&self) {
        //#[cfg(feature = "flecs_system")]
        //System::system_init(self);
        //#[cfg(feature = "flecs_timer")]
        //todo!();
        //#[cfg(feature = "flecs_doc")]
        //todo!();
        //#[cfg(feature = "flecs_rest")]
        //todo!();
        //#[cfg(feature = "flecs_meta")]
        //todo!();
    }

    /// deletes and recreates the world
    ///
    /// # See also
    ///
    /// `world::reset`
    pub fn reset(&mut self) {
        // can only reset the world if we own the world object.
        ecs_assert!(
            self.is_owned,
            FlecsErrorCode::InvalidOperation,
            "Cannot reset a borrowed world"
        );
        unsafe { ecs_fini(self.raw_world) };
        self.raw_world = unsafe { ecs_init() };
    }

    /// obtain pointer to C world object
    ///
    /// # Returns
    ///
    /// Returns a pointer to the C world object.
    ///
    /// # See also
    ///
    /// `world::c_ptr`
    pub fn get_as_ptr(&self) -> *mut WorldT {
        self.raw_world
    }

    /// Gets the last delta_time.
    ///
    /// Returns the time that has passed since the last frame.
    ///
    /// # See also
    ///
    /// `world::delta_time`
    pub fn delta_time(&self) -> f32 {
        unsafe {
            let stats = ecs_get_world_info(self.raw_world);
            (*stats).delta_time
        }
    }

    /// Gets the current tick.
    ///
    /// Returns the total number of frames that have passed.
    ///
    /// # See also
    ///
    /// `world::tick`
    pub fn tick(&self) -> i64 {
        unsafe {
            let stats = ecs_get_world_info(self.raw_world);
            (*stats).frame_count_total
        }
    }

    /// Gets the current simulation time.
    ///
    /// Returns the total time that has passed in the simulation.
    ///
    /// # See also
    ///
    /// `world::time`
    pub fn time(&self) -> f32 {
        unsafe {
            let stats = ecs_get_world_info(self.raw_world);
            (*stats).world_time_total
        }
    }

    /// Signals the application to quit.
    ///
    /// After calling this function, the next call to progress() returns false.
    ///
    /// # See also
    ///
    /// `world::quit`
    pub fn quit(&self) {
        unsafe {
            ecs_quit(self.raw_world);
        }
    }

    /// Registers an action to be executed when the world is destroyed.
    ///
    /// # See also
    ///
    /// `world::atfini`
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // this doesn't actually deref the pointer
    pub fn on_destroyed(&self, action: ecs_fini_action_t, ctx: *mut std::ffi::c_void) {
        unsafe {
            ecs_atfini(self.raw_world, action, ctx);
        }
    }

    /// Tests if `quit` has been called.
    ///
    /// # Returns
    ///
    /// True if quit has been called, false otherwise.
    ///
    /// # See also
    ///
    /// `world::should_quit`
    pub fn should_quit(&self) -> bool {
        unsafe { ecs_should_quit(self.raw_world) }
    }

    /// Begins a frame.
    ///
    /// When an application does not use progress() to control the main loop, it
    /// can still use Flecs features such as FPS limiting and time measurements.
    /// can still use Flecs features such as FPS limiting and time measurements processed.
    ///
    /// Calls to frame_begin must always be followed by frame_end.
    ///
    /// The function accepts a delta_time parameter, which will get passed to
    /// systems. This value is also used to compute the amount of time the
    /// function needs to sleep to ensure it does not exceed the target_fps, when
    /// it is set. When 0 is provided for delta_time, the time will be measured.
    ///
    /// # Safety
    /// This function should only be ran from the main thread.
    ///
    /// # Parameters
    /// * `delta_time`: Time elapsed since the last frame.
    ///
    /// # Returns
    /// The provided `delta_time`, or the measured time if 0 was provided.
    ///
    /// # See also
    ///
    /// `world::frame_begin`
    pub fn frame_begin(&self, delta_time: f32) -> f32 {
        unsafe { ecs_frame_begin(self.raw_world, delta_time) }
    }

    /// Ends a frame.
    ///
    /// This operation must be called at the end of the frame, and always after
    /// `frame_begin`.
    ///
    /// # Safety
    /// The function should only be run from the main thread.
    ///
    /// # See also
    ///
    /// `world::frame_end`
    pub fn frame_end(&self) {
        unsafe {
            ecs_frame_end(self.raw_world);
        }
    }

    /// Begin staging.
    ///
    /// When an application does not use `ecs_progress` to control the main loop, it
    /// can still use Flecs features such as the defer queue. To stage changes, this function
    /// must be called after `ecs_frame_begin`.
    ///
    /// A call to `ecs_readonly_begin` must be followed by a call to `ecs_readonly_end`.
    ///
    /// When staging is enabled, modifications to entities are stored to a stage.
    /// This ensures that arrays are not modified while iterating. Modifications are
    /// merged back to the "main stage" when `ecs_readonly_end` is invoked.
    ///
    /// While the world is in staging mode, no structural changes (add/remove/...) can
    /// be made to the world itself. Operations must be executed on a stage instead (see `ecs_get_stage`).
    ///
    /// ## safety
    /// This function should only be ran from the main thread.
    ///
    /// # Returns
    /// Whether the world is currently staged.
    ///
    /// # See also
    ///
    /// `world::readonly_begin`
    pub fn readonly_begin(&self) -> bool {
        unsafe { ecs_readonly_begin(self.raw_world) }
    }

    /// End staging.
    ///
    /// Leaves staging mode. After this operation, the world may be directly mutated again.
    /// By default, this operation also merges data back into the world, unless automerging
    /// was disabled explicitly.
    ///
    /// ## safety
    /// This function should only be run from the main thread.
    ///
    /// # Returns
    ///
    /// Whether the world is currently staged.
    pub fn readonly_end(&self) {
        unsafe {
            ecs_readonly_end(self.raw_world);
        }
    }

    /// Defers operations until the end of the frame.
    ///
    /// When this operation is invoked while iterating, the operations between
    /// `defer_begin` and `defer_end` are executed at the end of the frame.
    ///
    /// ## safety
    /// this operation is thread safe
    ///
    /// # Returns
    /// Whether the operation was successful.
    ///
    /// # See also
    ///
    /// `world::defer_begin`
    pub fn defer_begin(&self) -> bool {
        unsafe { ecs_defer_begin(self.raw_world) }
    }

    /// Ends a block of operations to defer.
    ///
    /// This should follow a `defer_begin` call.
    ///
    /// ## safety
    /// this operation is thread safe
    ///
    /// # Returns
    /// Whether the operation was successful.
    ///
    /// # See also
    ///
    /// `world::defer_end`
    pub fn defer_end(&self) -> bool {
        unsafe { ecs_defer_end(self.raw_world) }
    }

    /// Test whether deferring is enabled.
    ///
    /// # Returns
    ///
    /// Whether deferring is enabled.
    ///
    /// # See also
    ///
    /// `world::is_deferred`
    pub fn is_deferred(&self) -> bool {
        unsafe { ecs_is_deferred(self.raw_world) }
    }

    /// Configure world to have N stages.
    ///
    /// This initializes N stages, which allows applications to defer operations to
    /// multiple isolated defer queues. This is typically used for applications with
    /// multiple threads, where each thread gets its own queue, and commands are
    /// merged when threads are synchronized.
    ///
    /// Note that `set_threads()` already creates the appropriate number of stages.
    /// The `set_stage_count()` operation is useful for applications that want to manage
    /// their own stages and/or threads.
    ///
    /// # Parameters
    ///
    /// * `stages`: The number of stages.
    ///
    /// # See also
    ///
    /// `world::set_stage_count`
    pub fn set_stage_count(&self, stages: i32) {
        unsafe {
            ecs_set_stage_count(self.raw_world, stages);
        }
    }

    /// Get number of configured stages.
    ///
    /// Return number of stages set by `set_stage_count`.
    ///
    /// # Returns
    ///
    /// The number of stages used for threading.
    ///
    /// # See also
    ///
    /// `world::get_stage_count`
    pub fn get_stage_count(&self) -> i32 {
        unsafe { ecs_get_stage_count(self.raw_world) }
    }

    /// Get current stage id.
    ///
    /// The stage id can be used by an application to learn about which stage it
    /// is using, which typically corresponds with the worker thread id.
    ///
    /// # Returns
    ///
    /// The stage id.
    ///
    /// # See also
    ///
    /// `world::get_stage_id`
    pub fn get_stage_id(&self) -> i32 {
        unsafe { ecs_get_stage_id(self.raw_world) }
    }

    /// Test if is a stage.
    ///
    /// If this function returns `false`, it is guaranteed that this is a valid
    /// world object.
    ///
    /// # Returns
    ///
    /// True if the world is a stage, false if not.
    ///
    /// # See also
    ///
    /// `world::is_stage`
    pub fn is_stage(&self) -> bool {
        unsafe {
            ecs_assert!(
                _ecs_poly_is(self.raw_world as *const c_void, ecs_world_t_magic as i32)
                    || _ecs_poly_is(self.raw_world as *const c_void, ecs_stage_t_magic as i32),
                FlecsErrorCode::InvalidParameter,
                "Parameter is not a world or stage"
            );
            _ecs_poly_is(self.raw_world as *const c_void, ecs_stage_t_magic as i32)
        }
    }

    /// Enable/disable automerging for world or stage.
    ///
    /// When automerging is enabled, staged data will automatically be merged
    /// with the world when staging ends. This happens at the end of `progress()`,
    /// at a sync point or when `readonly_end()` is called.
    ///
    /// Applications can exercise more control over when data from a stage is
    /// merged by disabling automerging. This requires an application to
    /// explicitly call `merge()` on the stage.
    ///
    /// When this function is invoked on the world, it sets all current stages to
    /// the provided value and sets the default for new stages. When this
    /// function is invoked on a stage, automerging is only set for that specific
    /// stage.
    ///
    /// # Arguments
    ///
    /// * `automerge` - Whether to enable or disable automerging.
    ///
    /// # See also
    ///
    /// `world::set_automerge`
    pub fn set_automerge(&self, automerge: bool) {
        unsafe { ecs_set_automerge(self.raw_world, automerge) };
    }

    /// Merge world or stage.
    ///
    /// When automatic merging is disabled, an application can call this
    /// operation on either an individual stage, or on the world which will merge
    /// all stages. This operation may only be called when staging is not enabled
    /// (either after `progress()` or after `readonly_end()`).
    ///
    /// This operation may be called on an already merged stage or world.
    ///
    /// # See also
    ///
    /// `world::merge`
    pub fn merge(&self) {
        unsafe { ecs_merge(self.raw_world) };
    }

    /// Get stage-specific world pointer.
    ///
    /// Flecs threads can safely invoke the API as long as they have a private
    /// context to write to, also referred to as the stage. This function returns a
    /// pointer to a stage, disguised as a world pointer.
    ///
    /// Note that this function does not(!) create a new world. It simply wraps the
    /// existing world in a thread-specific context, which the API knows how to
    /// unwrap. The reason the stage is returned as an `ecs_world_t` is so that it
    /// can be passed transparently to the existing API functions, vs. having to
    /// create a dedicated API for threading.
    ///
    /// # Arguments
    ///
    /// * `stage_id` - The index of the stage to retrieve.
    ///
    /// # Returns
    ///
    /// A thread-specific pointer to the world.
    ///
    /// # See also
    ///
    /// `world::get_stage`
    pub fn get_stage(&self, stage_id: i32) -> Self {
        Self {
            raw_world: unsafe { ecs_get_stage(self.raw_world, stage_id) },
            is_owned: false,
        }
    }

    /// Create asynchronous stage.
    ///
    /// An asynchronous stage can be used to asynchronously queue operations for
    /// later merging with the world. An asynchronous stage is similar to a regular
    /// stage, except that it does not allow reading from the world.
    ///
    /// Asynchronous stages are never merged automatically, and must therefore be
    /// manually merged with the `ecs_merge` function. It is not necessary to call
    /// `defer_begin` or `defer_end` before and after enqueuing commands, as an
    /// asynchronous stage unconditionally defers operations.
    ///
    /// The application must ensure that no commands are added to the stage while the
    /// stage is being merged.
    ///
    /// An asynchronous stage must be cleaned up by `ecs_async_stage_free`.
    ///
    /// # Returns
    ///
    /// The stage.
    ///
    /// # See also
    ///
    /// `world::async_stage`
    pub fn get_async_stage(&self) -> Self {
        Self {
            raw_world: unsafe { ecs_async_stage_new(self.raw_world) },
            is_owned: true,
        }
    }

    /// Get actual world.
    ///
    /// If the current object points to a stage, this operation will return the
    /// actual world.
    ///
    /// # Returns
    ///
    /// The actual world.
    ///
    /// # See also
    ///
    /// `world::get_world`
    pub fn get_world(&self) -> Self {
        Self {
            raw_world: if !self.raw_world.is_null() {
                unsafe { ecs_get_world(self.raw_world as *const c_void) as *mut WorldT }
            } else {
                std::ptr::null_mut()
            },
            is_owned: false,
        }
    }

    /// Test whether the current world object is readonly.
    ///
    /// This function allows the code to test whether the currently used world
    /// object is readonly or whether it allows for writing.
    ///
    /// # Returns
    ///
    /// True if the world or stage is readonly.
    ///
    /// # See also
    ///
    /// `world::is_readonly`
    pub fn is_readonly(&self) -> bool {
        unsafe { ecs_stage_is_readonly(self.raw_world) }
    }

    /// Set world context.
    ///
    /// Set a context value that can be accessed by anyone that has a reference
    /// to the world.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The world context.
    ///
    /// # See also
    ///
    /// `world::set_context`
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // this doesn't actually deref the pointer
    pub fn set_context(&self, ctx: *mut std::ffi::c_void) {
        unsafe { ecs_set_context(self.raw_world, ctx) }
    }

    /// Get world context.
    ///
    /// # Returns
    ///
    /// The configured world context.
    ///
    /// # See also
    ///
    /// `world::get_context`
    pub fn get_context(&self) -> *mut std::ffi::c_void {
        unsafe { ecs_get_context(self.raw_world) }
    }

    /// Preallocate memory for a number of entities.
    ///
    /// This function preallocates memory for the entity index.
    ///
    /// # Arguments
    ///
    /// * `entity_count` - Number of entities to preallocate memory for.
    ///
    /// # See also
    ///
    /// `world::dim`
    pub fn preallocate_entity_count(&self, entity_count: i32) {
        unsafe { ecs_dim(self.raw_world, entity_count) };
    }

    /// Set the entity range.
    ///
    /// This function limits the range of issued entity IDs between `min` and `max`.
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum entity ID issued.
    /// * `max` - Maximum entity ID issued.
    ///
    /// # See also
    ///
    /// `world::set_entity_range`
    pub fn set_entity_range(&self, min: EntityT, max: EntityT) {
        unsafe { ecs_set_entity_range(self.raw_world, min, max) };
    }

    /// Enforce that operations cannot modify entities outside of the specified range.
    ///
    /// This function ensures that only entities within the specified range can
    /// be modified. Use this function if specific parts of the code are only allowed
    /// to modify a certain set of entities, as could be the case for networked applications.
    ///
    /// # Arguments
    ///
    /// * `enabled` - True if the range check should be enabled, false otherwise.
    ///
    /// # See also
    ///
    /// `world::enable_range_check`
    pub fn enable_range_check(&self, enabled: bool) {
        unsafe { ecs_enable_range_check(self.raw_world, enabled) };
    }

    /// Gets the current scope.
    ///
    /// # Returns
    ///
    /// Returns an `Entity` representing the current scope.
    ///
    /// # See Also
    ///
    /// Refer to `ecs_get_scope` for underlying implementation details.
    ///
    /// # See also
    ///
    /// `world::get_scope`
    #[inline(always)]
    pub fn get_scope<T: CachedComponentData>(&self) -> Entity {
        Entity::new_from_existing_raw(self.raw_world, unsafe { ecs_get_scope(self.raw_world) })
    }

    /// Sets the current scope.
    ///
    /// This method changes the current scope to the entity represented by the provided `id`.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the scope entity to set.
    ///
    /// # Returns
    ///
    /// Returns an `Entity` representing the newly set scope.
    ///
    /// # See Also
    ///
    /// Refer to `ecs_set_scope` for underlying implementation details.
    ///
    /// # See also
    ///
    /// `world::set_scope`
    #[inline(always)]
    pub fn set_scope_with_id(&self, id: EntityT) -> Entity {
        Entity::new_id_only(unsafe { ecs_set_scope(self.raw_world, id) })
    }

    /// Sets the current scope, but allows the scope type to be inferred from the type parameter.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type that implements `CachedComponentData`.
    ///
    /// # Returns
    ///
    /// Returns an `Entity` representing the newly set scope.
    ///
    /// # See Also
    ///
    /// Refer to `ecs_set_scope` for underlying implementation details.
    ///
    /// # See also
    ///
    /// `world::set_scope`
    #[inline(always)]
    pub fn set_scope_with<T: CachedComponentData>(&self) -> Entity {
        self.set_scope_with_id(T::get_id(self.raw_world))
    }

    /// Sets the search path for entity lookup operations.
    ///
    /// This function configures the search path used for looking up entities. The search path is an array of entity IDs that define the scopes within which lookup operations will search for entities.
    ///
    /// # Best Practices
    ///
    /// * It's advisable to restore the previous search path after making temporary changes.
    ///
    /// # Search Path Evaluation
    ///
    /// * The search path is evaluated starting from the last element of the array.
    ///
    /// # Default Behavior
    ///
    /// * The default search path includes `flecs.core`.
    ///
    /// # Overwriting
    ///
    /// * Providing a custom search path will overwrite the existing search path.
    ///
    /// # Considerations
    ///
    /// * If the custom search path doesn't include `flecs.core`, operations that rely on looking up names from `flecs.core` may fail.
    /// * The search path array is not managed by the Rust runtime. Ensure the array remains valid for as long as it is used as the search path.
    ///
    /// # Array Termination
    ///
    /// * The provided array must be terminated with a 0 element. This allows for pushing/popping elements onto/from an existing array without needing to call `ecs_set_lookup_path` again.
    ///
    /// # Arguments
    ///
    /// * `search_path` - A 0-terminated array of entity IDs defining the new search path.
    ///
    /// # Returns
    ///
    /// Returns the current search path after the operation.
    ///
    /// # See also
    ///
    /// `world::set_lookup_path`
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // this doesn't actually deref the pointer
    pub fn set_lookup_path(&self, search_path: *const EntityT) -> *mut EntityT {
        unsafe { ecs_set_lookup_path(self.raw_world, search_path) }
    }

    /// Lookup entity by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    ///
    /// # Returns
    ///
    /// The entity if found, otherwise None.
    ///
    /// # See also
    ///
    /// `world::lookup`
    pub fn lookup_entity_by_name(&self, name: &CStr) -> Option<Entity> {
        let entity_id = unsafe {
            ecs_lookup_path_w_sep(
                self.raw_world,
                0,
                name.as_ptr(),
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
                true,
            )
        };
        if entity_id == 0 {
            None
        } else {
            Some(Entity::new_from_existing_raw(self.raw_world, entity_id))
        }
    }

    /// Sets a singleton component of type `T` on the world.
    ///
    /// # Arguments
    ///
    /// * `component` - The singleton component to set on the world.
    ///  
    /// # See also
    ///
    /// `world::set`
    pub fn set_component<T: CachedComponentData>(&self, component: T) {
        let id = T::get_id(self.raw_world);
        set_helper(self.raw_world, id, component, id);
    }

    /// Set a singleton pair using the second element type and a first id.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// # Parameters
    ///
    /// * `first`: The ID of the first element of the pair.
    /// * `second`: The second element of the pair to be set.
    ///  
    /// # See also
    ///
    /// `world::set`
    pub fn set_pair_first_id<First>(&self, second: EntityT, first: First)
    where
        First: CachedComponentData + ComponentType<Struct>,
    {
        let entity = Entity::new_from_existing_raw(self.raw_world, First::get_id(self.raw_world));
        entity.set_pair_first_id::<First>(second, first);
    }

    /// Set singleton pair.
    /// This operation sets the pair value, and uses First as type. If it does not yet exist, it will be added.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair
    /// * `Second`: The second element of the pair
    ///
    /// # Parameters
    ///
    /// * `first`: The value to set for first component.
    ///  
    /// # See also
    ///
    /// `world::set`
    pub fn set_pair_first<First, Second>(&self, first: First)
    where
        First: CachedComponentData + ComponentType<Struct>,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        let entity = Entity::new_from_existing_raw(self.raw_world, First::get_id(self.raw_world));
        entity.set_pair_first::<First, Second>(first);
    }

    /// Set a singleton pair using the second element type and a first id.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// # Parameters
    ///
    /// * `first`: The ID of the first element of the pair.
    /// * `second`: The second element of the pair to be set.
    ///  
    /// # See also
    ///
    /// `world::set`
    pub fn set_pair_second_id<Second>(&self, first: EntityT, second: Second)
    where
        Second: CachedComponentData + ComponentType<Struct>,
    {
        let entity = Entity::new_from_existing_raw(self.raw_world, Second::get_id(self.raw_world));
        entity.set_pair_second_id::<Second>(first, second);
    }

    /// Set singleton pair.
    /// This operation sets the pair value, and uses Second as type. If it does not yet exist, it will be added.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair
    ///
    /// # Parameters
    ///
    /// * `first`: The first element of the pair.
    /// * `value`: The value to set.
    ///
    /// # See also
    ///
    /// `world::set`
    pub fn set_pair_second<First, Second>(&self, second: Second)
    where
        First: CachedComponentData + ComponentType<Struct>,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        let entity = Entity::new_from_existing_raw(self.raw_world, First::get_id(self.raw_world));
        entity.set_pair_second::<First, Second>(second);
    }

    /// signal that singleton component was modified.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the component that was modified.
    ///
    /// # See also
    ///
    /// * C++ API: `world::modified`
    #[doc(alias = "world::modified")]
    #[inline(always)]
    pub fn mark_component_modified_with_id(&self, id: EntityT) {
        Entity::new_from_existing_raw(self.raw_world, id).mark_component_id_modified(id)
    }

    /// Signal that singleton component was modified.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the component that was modified.
    ///
    /// # See also
    ///
    /// * C++ API: `world::modified`
    #[doc(alias = "world::modified")]
    #[inline(always)]
    pub fn mark_component_modified_with<T>(&self)
    where
        T: CachedComponentData,
    {
        self.mark_component_modified_with_id(T::get_id(self.raw_world))
    }

    /// Get singleton component as const.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the component to get.
    ///
    /// # Returns
    ///
    /// The singleton component as const.
    ///
    /// # See also
    ///
    /// `world::get`
    #[inline(always)]
    pub fn get_component<T>(&self) -> *const T
    where
        T: CachedComponentData + ComponentType<Struct>,
    {
        Entity::new_from_existing_raw(self.raw_world, T::get_id(self.raw_world))
            .get_component::<T>()
    }

    /// Get singleton component as mutable.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the component to get.
    ///
    /// # Returns
    ///
    /// The singleton component as mutable.
    ///
    /// # See also
    ///
    /// `world::get_mut`
    #[inline(always)]
    pub fn get_component_mut<T>(&self) -> *mut T
    where
        T: CachedComponentData + ComponentType<Struct>,
    {
        Entity::new_from_existing_raw(self.raw_world, T::get_id(self.raw_world))
            .get_component_mut::<T>()
    }

    /// Get a reference to a singleton component.
    ///
    /// A reference allows for quick and safe access to a component value, and is
    /// a faster alternative to repeatedly calling `get` for the same component.
    ///
    /// - `T`: Component for which to get a reference.
    ///
    /// Returns: The reference singleton component.
    ///   
    /// # See also
    ///
    /// `world::get_ref`
    #[inline(always)]
    pub fn get_ref_component<T>(&self) -> Ref<T>
    where
        T: CachedComponentData,
    {
        Entity::new_from_existing_raw(self.raw_world, T::get_id(self.raw_world))
            .get_ref_component::<T>()
    }

    /// Get singleton entity for type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to get the singleton entity for.
    ///
    /// # Returns
    ///
    /// The entity representing the component.
    ///
    /// # See also
    ///
    /// `world::singleton`
    #[inline(always)]
    pub fn get_component_as_entity<T: CachedComponentData>(&self) -> Entity {
        Entity::new_from_existing_raw(self.raw_world, T::get_id(self.raw_world))
    }

    /// Get const pointer for the first element of a singleton pair
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first part of the pair.
    ///
    /// # Parameters
    ///
    /// * `second`: The second element of the pair.
    ///   
    /// # See also
    ///
    /// `world::get`
    #[inline(always)]
    pub fn get_pair_first<First>(&self, second: EntityT) -> *const First
    where
        First: CachedComponentData,
    {
        Entity::new_from_existing_raw(self.raw_world, First::get_id(self.raw_world))
            .get_pair_first::<First>(second)
    }

    /// Get mutable pointer for the first element of a singleton pair
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first part of the pair.
    ///
    /// # Parameters
    ///
    /// * `second`: The second element of the pair.
    ///   
    /// # See also
    ///
    /// `world::get_mut`
    #[inline(always)]
    pub fn get_pair_first_mut<First>(&self, second: EntityT) -> *mut First
    where
        First: CachedComponentData,
    {
        Entity::new_from_existing_raw(self.raw_world, First::get_id(self.raw_world))
            .get_pair_first_mut::<First>(second)
    }

    /// Get mutable pointer for the second element of a singleton pair
    ///
    /// # Type Parameters
    ///
    /// * `second`: The second part of the pair.
    ///
    /// # Parameters
    ///
    /// * `first`: The first element of the pair.
    ///   
    /// # See also
    ///
    /// `world::get`
    #[inline(always)]
    pub fn get_pair_second<Second>(&self, first: EntityT) -> *const Second
    where
        Second: CachedComponentData,
    {
        Entity::new_from_existing_raw(self.raw_world, Second::get_id(self.raw_world))
            .get_pair_second::<Second>(first)
    }

    /// Get mutable pointer for the second element of a singleton pair
    ///
    /// # Type Parameters
    ///
    /// * `second`: The second part of the pair.
    ///
    /// # Parameters
    ///
    /// * `first`: The first element of the pair.
    ///  
    /// # See also
    ///
    /// `world::get_mut`
    #[inline(always)]
    pub fn get_pair_second_mut<Second>(&self, first: EntityT) -> *mut Second
    where
        Second: CachedComponentData,
    {
        Entity::new_from_existing_raw(self.raw_world, Second::get_id(self.raw_world))
            .get_pair_second_mut::<Second>(first)
    }

    /// Check if world has the provided struct component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component to check.
    ///
    /// # Returns
    ///
    /// True if the world has the provided component, false otherwise.
    ///  
    /// # See also
    ///
    /// `world::has`
    #[inline(always)]
    pub fn has<T>(&self) -> bool
    where
        T: CachedComponentData + ComponentType<Struct>,
    {
        Entity::new_from_existing_raw(self.raw_world, T::get_id(self.raw_world)).has::<T>()
    }

    /// Check if world has the provided enum component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component to check.
    ///
    /// # Returns
    ///
    /// True if the world has the enum provided component, false otherwise.
    ///  
    /// # See also
    ///
    /// `world::has`
    #[inline(always)]
    pub fn has_enum<T>(&self) -> bool
    where
        T: CachedComponentData + ComponentType<Enum>,
    {
        Entity::new_from_existing_raw(self.raw_world, T::get_id(self.raw_world)).has_enum::<T>()
    }

    /// Check if world has the provided enum constant.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum type.
    ///
    /// # Arguments
    ///
    /// * `constant` - The enum constant to check.
    ///
    /// # Returns
    ///
    /// True if the world has the provided constant, false otherwise.
    ///  
    /// # See also
    ///
    /// `world::has`
    #[inline(always)]
    pub fn has_enum_constant<T>(&self, constant: T) -> bool
    where
        T: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        Entity::new_from_existing_raw(self.raw_world, T::get_id(self.raw_world))
            .has_enum_constant::<T>(constant)
    }

    /// Check if world has the provided pair.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The first element of the pair.
    /// * `U` - The second element of the pair.
    ///
    /// # Returns
    ///
    /// True if the world has the provided component, false otherwise.
    ///
    /// # See also
    ///
    /// `world::has`
    #[inline(always)]
    pub fn has_pair_component<First, Second>(&self) -> bool
    where
        First: CachedComponentData + ComponentType<Struct>,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        Entity::new_from_existing_raw(self.raw_world, First::get_id(self.raw_world))
            .has_pair::<First, Second>()
    }

    /// Check if world has the provided pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    /// * `second`: The second element of the pair.
    ///
    /// # Returns
    ///
    /// True if the world has the provided component, false otherwise.
    ///
    /// # See also
    ///
    /// `world::has`
    #[inline(always)]
    pub fn has_pair_by_id(&self, first: EntityT, second: EntityT) -> bool {
        Entity::new_from_existing_raw(self.raw_world, first).has_pair_by_ids(first, second)
    }

    /// Add a singleton component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component to add.
    ///
    /// # Returns
    ///
    /// Entity handle to the singleton component.
    ///
    /// # See also
    ///
    /// `world::add`
    #[inline(always)]
    pub fn add_component<T: CachedComponentData>(&self) -> Entity {
        Entity::new_from_existing_raw(self.raw_world, T::get_id(self.raw_world))
            .add_component::<T>()
    }

    /// Add a singleton enum component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum component to add.
    ///
    /// # Returns
    ///
    /// Entity handle to the singleton enum component.
    ///
    /// # See also
    ///
    /// `world::add`
    #[inline(always)]
    pub fn add_enum_constant<T: CachedComponentData + ComponentType<Enum> + CachedEnumData>(
        &self,
        enum_value: T,
    ) -> Entity {
        Entity::new_from_existing_raw(self.raw_world, T::get_id(self.raw_world))
            .add_enum_constant::<T>(enum_value)
    }

    /// Add a singleton pair by ids
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    /// * `second`: The second element of the pair.
    ///
    /// # Returns
    ///
    /// Entity handle to the singleton pair.
    ///
    /// # See also
    ///
    /// `world::add`
    #[inline(always)]
    pub fn add_pair_ids(&self, first: EntityT, second: EntityT) -> Entity {
        Entity::new_from_existing_raw(self.raw_world, first).add_pair_ids(first, second)
    }

    /// Add a singleton pair.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    /// * `Second` - The second element of the pair.
    ///
    /// # Returns
    ///
    /// Entity handle to the singleton pair.
    ///
    /// # See also
    ///
    /// `world::add`
    #[inline(always)]
    pub fn add_pair<First, Second>(&self) -> Entity
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        Entity::new_from_existing_raw(self.raw_world, First::get_id(self.raw_world))
            .add_pair::<First, Second>()
    }

    /// Add a singleton pair by first id.
    ///
    /// # Type Parameters
    ///
    /// * `Second` - The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    ///
    /// # Returns
    ///
    /// Entity handle to the singleton pair.
    #[inline(always)]
    pub fn add_pair_first_id<Second: CachedComponentData>(&self, first: EntityT) -> Entity {
        Entity::new_from_existing_raw(self.raw_world, Second::get_id(self.raw_world))
            .add_pair_first_id::<Second>(first)
    }

    /// Add a singleton pair by second id.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    ///
    /// # Returns
    ///
    /// Entity handle to the singleton pair.
    ///
    /// # See also
    ///
    /// `world::add`
    #[inline(always)]
    pub fn add_pair_second_id<First: CachedComponentData>(&self, second: EntityT) -> Entity {
        Entity::new_from_existing_raw(self.raw_world, First::get_id(self.raw_world))
            .add_pair_second_id::<First>(second)
    }

    /// Add a singleton pair with enum tag.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    /// * `Second` - The second element of the pair of type enum.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to add.
    ///
    /// # Returns
    ///
    /// Entity handle to the singleton pair.
    ///
    /// # See also
    ///
    /// `world::add`
    #[inline(always)]
    pub fn add_enum_tag<First, Second>(&self, enum_value: Second) -> Entity
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        Entity::new_from_existing_raw(self.raw_world, First::get_id(self.raw_world))
            .add_enum_tag::<First, Second>(enum_value)
    }

    /// Remove singleton component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component to remove.
    ///
    /// # See also
    ///
    /// `world::remove`
    #[inline(always)]
    pub fn remove_component<T: CachedComponentData + ComponentType<Struct>>(&self) {
        Entity::new_from_existing_raw(self.raw_world, T::get_id(self.raw_world))
            .remove_component::<T>();
    }

    /// Remove singleton enum component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum component to remove.
    ///
    /// # See also
    ///
    /// `world::remove`
    #[inline(always)]
    pub fn remove_component_enum<T: CachedComponentData + ComponentType<Enum>>(&self) {
        Entity::new_from_existing_raw(self.raw_world, T::get_id(self.raw_world))
            .remove_component_enum::<T>();
    }

    /// Remove singleton pair with enum tag.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    /// * `Second` - The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `enum_value` - The enum value to remove.
    ///
    /// # See also
    ///
    /// `world::remove`
    #[inline(always)]
    pub fn remove_enum_tag<First, Second>(&self, enum_value: Second)
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        Entity::new_from_existing_raw(self.raw_world, First::get_id(self.raw_world))
            .remove_enum_tag::<First, Second>(enum_value);
    }

    /// Remove singleton pair by ids.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    /// * `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// `world::remove`
    #[inline(always)]
    pub fn remove_pair_ids(&self, first: EntityT, second: EntityT) {
        Entity::new_from_existing_raw(self.raw_world, first).remove_pair_ids(first, second);
    }

    /// Remove singleton pair.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    /// * `Second` - The second element of the pair.
    ///
    /// # See also
    ///
    /// `world::remove`
    #[inline(always)]
    pub fn remove_pair<First, Second>(&self)
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        Entity::new_from_existing_raw(self.raw_world, First::get_id(self.raw_world))
            .remove_pair::<First, Second>();
    }

    /// Remove singleton pair by first id.
    ///
    /// # Type Parameters
    ///
    /// * `Second` - The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    ///
    /// # See also
    ///
    /// `world::remove`
    #[inline(always)]
    pub fn remove_pair_first_id<Second: CachedComponentData>(&self, first: EntityT) {
        Entity::new_from_existing_raw(self.raw_world, Second::get_id(self.raw_world))
            .remove_pair_first_id::<Second>(first);
    }

    /// Remove singleton pair by second id.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// `world::remove`
    #[inline(always)]
    pub fn remove_pair_second_id<First: CachedComponentData>(&self, second: EntityT) {
        Entity::new_from_existing_raw(self.raw_world, First::get_id(self.raw_world))
            .remove_pair_second_id::<First>(second);
    }

    /// Iterate entities in root of world
    ///
    /// # Arguments
    ///
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(Entity)`.
    ///
    /// # See also
    ///
    /// `world::children`
    #[inline(always)]
    pub fn for_each_children<F: FnMut(Entity)>(&self, callback: F) {
        Entity::new(self).for_each_child_of(callback);
    }

    /// create alias for component
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to create an alias for.
    ///
    /// # Arguments
    ///
    /// * `alias` - The alias to create.
    ///
    /// # Returns
    ///
    /// The entity representing the component.
    ///
    /// # See also
    ///
    /// `world::use`
    #[inline(always)]
    pub fn set_alias_component<T: CachedComponentData>(&self, alias: &CStr) -> Entity {
        let id = T::get_id(self.raw_world);
        if alias.is_empty() {
            unsafe { ecs_set_alias(self.raw_world, id, ecs_get_name(self.raw_world, id)) };
        } else {
            unsafe { ecs_set_alias(self.raw_world, id, alias.as_ptr()) };
        }
        Entity::new_from_existing_raw(self.raw_world, id)
    }

    /// create alias for entity by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to create an alias for.
    /// * `alias` - The alias to create.
    ///
    /// # Returns
    ///
    /// The entity found by name.
    ///
    /// # See also
    ///
    /// `world::use`
    #[inline(always)]
    pub fn set_alias_entity_by_name(&self, name: &CStr, alias: &CStr) -> Entity {
        let id = unsafe {
            ecs_lookup_path_w_sep(
                self.raw_world,
                0,
                name.as_ptr(),
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
                true,
            )
        };
        ecs_assert!(id != 0, FlecsErrorCode::InvalidParameter);
        unsafe { ecs_set_alias(self.raw_world, id, alias.as_ptr()) };
        Entity::new_from_existing_raw(self.raw_world, id)
    }

    /// create alias for entity
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to create an alias for.
    /// * `alias` - The alias to create.
    ///
    /// # See also
    ///
    /// `world::use`
    #[inline(always)]
    pub fn set_alias_entity(&self, entity: Entity, alias: &CStr) {
        if alias.is_empty() {
            unsafe {
                ecs_set_alias(
                    self.raw_world,
                    entity.raw_id,
                    ecs_get_name(self.raw_world, entity.raw_id),
                )
            };
        } else {
            unsafe { ecs_set_alias(self.raw_world, entity.raw_id, alias.as_ptr()) };
        }
    }

    /// Count entities with the provided id.
    ///
    /// # Arguments
    ///
    /// * `id` - The id to count.
    ///
    /// # Returns
    ///
    /// The number of entities with the provided id.
    ///
    /// # See also
    ///
    /// * C++ API: `world::count`
    #[doc(alias = "world::count")]
    pub fn count_id(&self, id: IdT) -> i32 {
        unsafe { ecs_count_id(self.raw_world, id) }
    }

    /// Count entities with the provided component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component to count.
    ///
    /// # Returns
    ///
    /// The number of entities with the provided component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::count`
    #[doc(alias = "world::count")]
    pub fn count_component<T: CachedComponentData + ComponentType<Struct>>(&self) -> i32 {
        self.count_id(T::get_id(self.raw_world))
    }

    /// Count entities with the provided pair ids.
    ///
    /// # Arguments
    ///
    /// * `first` - The ID of the first element of the pair.
    /// * `second` - The ID of the second element of the pair.
    ///
    /// # Returns
    ///
    /// The number of entities with the provided pair.
    ///
    /// # See also
    ///
    /// * C++ API: `world::count`
    #[doc(alias = "world::count")]
    pub fn count_pair_ids(&self, first: EntityT, second: EntityT) -> i32 {
        self.count_id(ecs_pair(first, second))
    }

    /// Count entities with the provided pair.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    /// * `Second` - The second element of the pair.
    ///
    /// # Returns
    ///
    /// The number of entities with the provided pair.
    ///
    /// # See also
    ///
    /// * C++ API: `world::count`
    #[doc(alias = "world::count")]
    pub fn count_pair<First, Second>(&self) -> i32
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        self.count_pair_ids(
            First::get_id(self.raw_world),
            Second::get_id(self.raw_world),
        )
    }

    /// Count entities with the provided pair.
    ///
    /// # Type Parameters
    ///
    /// * `Second` - The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first` - The ID of the first element of the pair.
    ///
    /// # Returns
    ///
    /// The number of entities with the provided pair.
    ///
    /// # See also
    ///
    /// * C++ API: `world::count`
    #[doc(alias = "world::count")]
    pub fn count_pair_first_id<Second: CachedComponentData>(&self, first: EntityT) -> i32 {
        self.count_pair_ids(first, Second::get_id(self.raw_world))
    }

    /// Count entities with the provided pair.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second` - The ID of the second element of the pair.
    ///
    /// # Returns
    ///
    /// The number of entities with the provided pair.
    ///
    /// # See also
    ///
    /// * C++ API: `world::count`
    #[doc(alias = "world::count")]
    pub fn count_pair_second_id<First: CachedComponentData>(&self, second: EntityT) -> i32 {
        self.count_pair_ids(First::get_id(self.raw_world), second)
    }

    /// Count entities with the provided enum constant.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum type.
    ///
    /// # Arguments
    ///
    /// * `constant` - The enum constant to count.
    ///
    /// # Returns
    ///
    /// The number of entities with the provided enum constant.
    ///
    /// # See also
    ///
    /// * C++ API: `world::count`
    #[doc(alias = "world::count")]
    pub fn count_enum_constant<T: CachedComponentData + ComponentType<Enum> + CachedEnumData>(
        &self,
        enum_value: T,
    ) -> i32 {
        unsafe {
            ecs_count_id(
                self.raw_world,
                enum_value.get_entity_id_from_enum_field(self.raw_world),
            )
        }
    }

    /// Count entities with the provided pair enum tag.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    /// * `Second` - The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `enum_value` - The enum value to count.
    ///
    /// # Returns
    ///
    /// The number of entities with the provided pair enum tag.
    ///
    /// # See also
    ///
    /// * C++ API: `world::count`
    #[doc(alias = "world::count")]
    pub fn count_enum_tag_pair<First, Second>(&self, enum_value: Second) -> i32
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        unsafe {
            ecs_count_id(
                self.raw_world,
                ecs_pair(
                    First::get_id(self.raw_world),
                    enum_value.get_entity_id_from_enum_field(self.raw_world),
                ),
            )
        }
    }

    /// All entities created in function are created in scope. All operations
    /// called in function (such as lookup) are relative to scope.
    ///
    /// # Arguments
    ///
    /// * `parent_id` - The id of the scope to use.
    /// * `func` - The function to run.
    ///
    /// # See also
    ///
    /// `world::scope`
    pub fn run_in_scope_with_id<F: FnMut()>(&self, parent_id: IdT, mut func: F) {
        let prev: IdT = unsafe { ecs_set_scope(self.raw_world, parent_id) };
        func();
        unsafe {
            ecs_set_scope(self.raw_world, prev);
        }
    }

    /// All entities created in function are created in scope. All operations
    /// called in function (such as lookup) are relative to scope.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to use as scope / parent.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to run.
    ///
    /// # See also
    ///
    /// `world::scope`
    pub fn run_in_scope_with<T: CachedComponentData, F: FnMut()>(&self, func: F) {
        self.run_in_scope_with_id(T::get_id(self.raw_world), func);
    }

    /// Use provided scope for operations ran on returned world.
    /// Operations need to be ran in a single statement
    ///
    /// # Arguments
    ///
    /// * `parent_id` - The id of the scope to use.
    ///
    /// # Returns
    ///
    /// A scoped world.
    ///
    /// # See also
    ///
    /// `world::scope`
    pub fn get_scoped_world_with_id(&self, parent_id: IdT) -> ScopedWorld {
        ScopedWorld::new(self.raw_world, parent_id)
    }

    /// Use provided scope for operations ran on returned world.
    /// Operations need to be ran in a single statement
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to use as scope.
    ///
    /// # Returns
    ///
    /// A scoped world.
    ///
    /// # See also
    ///
    /// `world::scope`
    pub fn get_scoped_world_with<T: CachedComponentData>(&self) -> ScopedWorld {
        self.get_scoped_world_with_id(T::get_id(self.raw_world))
    }

    /// all entities created in function are created with id
    ///
    /// # Arguments
    ///
    /// * `id`: The id to create entities with.
    /// * `func`: The function to run.
    ///
    /// # See also
    ///
    /// `world::with`
    pub fn create_with_id<F: FnMut()>(&self, id: IdT, mut func: F) {
        let prev: IdT = unsafe { ecs_set_with(self.raw_world, id) };
        func();
        unsafe {
            ecs_set_with(self.raw_world, prev);
        }
    }

    /// Entities created in function are created with component
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component type.
    ///
    /// # Arguments
    ///
    /// * `func`: The function to run.
    ///
    /// # See also
    ///
    /// `world::with`
    pub fn create_with<T: CachedComponentData, F: FnMut()>(&self, func: F) {
        self.create_with_id(T::get_id(self.raw_world), func);
    }

    /// Entities created in function are created with pair
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    /// * `second`: The second element of the pair.
    /// * `func`: The function to run.
    ///
    /// # See also
    ///
    /// `world::with`
    pub fn create_with_pair_ids<F: FnMut()>(&self, first: IdT, second: IdT, func: F) {
        self.create_with_id(ecs_pair(first, second), func);
    }

    /// Entities created in function are created with pair
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    /// * `Second`: The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `func`: The function to run.
    ///
    /// # See also
    ///
    /// `world::with`
    pub fn create_with_pair<First, Second, F: FnMut()>(&self, func: F)
    where
        First: CachedComponentData,
        Second: CachedComponentData,
    {
        self.create_with_id(
            ecs_pair(
                First::get_id(self.raw_world),
                Second::get_id(self.raw_world),
            ),
            func,
        );
    }

    /// Entities created in function are created with pair
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    /// * `func`: The function to run.
    ///
    /// # See also
    ///
    /// `world::with`
    pub fn create_with_pair_first_id<Second: CachedComponentData, F: FnMut()>(
        &self,
        first: IdT,
        func: F,
    ) {
        self.create_with_id(ecs_pair(first, Second::get_id(self.raw_world)), func);
    }

    /// Entities created in function are created with pair
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    /// * `func`: The function to run.
    ///
    /// # See also
    ///
    /// `world::with`
    pub fn create_with_pair_second_id<First: CachedComponentData, F: FnMut()>(
        &self,
        second: IdT,
        func: F,
    ) {
        self.create_with_id(ecs_pair(First::get_id(self.raw_world), second), func);
    }

    /// Entities created in function are created with enum constant
    ///
    /// # Type Parameters
    ///
    /// * `T`: The enum type.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to give the entity.
    /// * `func`: The function to run.
    ///
    /// # See also
    ///
    /// `world::with`
    pub fn create_with_enum_constant<T, F>(&self, enum_value: T, func: F)
    where
        T: CachedComponentData + ComponentType<Enum> + CachedEnumData,
        F: FnMut(),
    {
        self.create_with_id(
            enum_value.get_entity_id_from_enum_field(self.raw_world),
            func,
        );
    }

    /// Entities created in function are created with enum tag pair
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    /// * `Second`: The enum component type.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to give the entity.
    /// * `func`: The function to run.
    ///
    /// # See also
    ///
    /// `world::with`
    pub fn create_with_enum_tag_pair<First, Second, F>(&self, enum_value: Second, func: F)
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Enum> + CachedEnumData,
        F: FnMut(),
    {
        self.create_with_id(
            ecs_pair(
                First::get_id(self.raw_world),
                enum_value.get_entity_id_from_enum_field(self.raw_world),
            ),
            func,
        );
    }

    /// Delete all entities with the given id
    ///
    /// # Arguments
    ///
    /// * `id`: The id to delete.
    ///
    /// # See also
    ///
    /// `world::delete_with`
    pub fn delete_entities_with_id(&self, id: IdT) {
        unsafe {
            ecs_delete_with(self.raw_world, id);
        }
    }

    /// Delete all entities with the given component
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component type to delete.
    ///
    /// # See also
    ///
    /// `world::delete_with`
    pub fn delete_entities_with<T: CachedComponentData>(&self) {
        self.delete_entities_with_id(T::get_id(self.raw_world));
    }

    /// Delete all entities with the given pair ids
    ///
    /// # Arguments
    ///
    /// * `first`: The first id of the pair.
    /// * `second`: The second id of the pair.
    ///
    /// # See also
    ///
    /// `world::delete_with`
    pub fn delete_entities_with_pair_ids(&self, first: IdT, second: IdT) {
        self.delete_entities_with_id(ecs_pair(first, second));
    }

    /// Delete all entities with the given pair
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    /// * `Second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// `world::delete_with`
    pub fn delete_entities_with_pair<First, Second>(&self)
    where
        First: CachedComponentData,
        Second: CachedComponentData,
    {
        self.delete_entities_with_id(ecs_pair(
            First::get_id(self.raw_world),
            Second::get_id(self.raw_world),
        ));
    }

    /// Delete all entities with the given pair
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first id of the pair.
    ///
    /// # See also
    ///
    /// `world::delete_with`
    pub fn delete_entities_with_pair_first_id<Second: CachedComponentData>(&self, first: IdT) {
        self.delete_entities_with_id(ecs_pair(first, Second::get_id(self.raw_world)));
    }

    /// Delete all entities with the given pair
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second`: The second id of the pair.
    ///
    /// # See also
    ///
    /// `world::delete_with`
    pub fn delete_entities_with_pair_second_id<First: CachedComponentData>(&self, second: IdT) {
        self.delete_entities_with_id(ecs_pair(First::get_id(self.raw_world), second));
    }

    /// Delete all entities with the given enum constant
    ///
    /// # Type Parameters
    ///
    /// * `T`: The enum type.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to filter against for deletion.
    ///
    /// # See also
    ///
    /// `world::delete_with`
    pub fn delete_entities_with_enum_constant<
        T: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    >(
        &self,
        enum_value: T,
    ) {
        self.delete_entities_with_id(enum_value.get_entity_id_from_enum_field(self.raw_world));
    }

    /// Delete all entities with the given enum tag pair / relationship
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    /// * `Second`: The enum component type.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to filter against for deletion.
    ///
    /// # See also
    ///
    /// * `world::delete_with`
    pub fn delete_entities_with_enum_tag_pair<First, Second>(&self, enum_value: Second)
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        self.delete_entities_with_id(ecs_pair(
            First::get_id(self.raw_world),
            enum_value.get_entity_id_from_enum_field(self.raw_world),
        ));
    }

    /// Remove all instances of the given id from entities
    ///
    /// # Arguments
    ///
    /// * `id`: The id to remove.
    ///
    /// # See also
    ///
    /// `world::remove_all`
    pub fn remove_all_id(&self, id: IdT) {
        unsafe {
            ecs_remove_all(self.raw_world, id);
        }
    }

    /// Remove all instances of the given component from entities
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component type to remove.
    ///
    /// # See also
    ///
    /// `world::remove_all`
    pub fn remove_all<T: CachedComponentData>(&self) {
        self.remove_all_id(T::get_id(self.raw_world));
    }

    /// Remove all instances of the given pair from entities
    ///
    /// # Arguments
    ///
    /// * `first`: The first id of the pair.
    /// * `second`: The second id of the pair.
    ///
    /// # See also
    ///
    /// `world::remove_all`
    pub fn remove_all_pair_ids(&self, first: IdT, second: IdT) {
        self.remove_all_id(ecs_pair(first, second));
    }

    /// Remove all instances of the given pair from entities
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    /// * `Second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// `world::remove_all`
    pub fn remove_all_pair<First, Second>(&self)
    where
        First: CachedComponentData,
        Second: CachedComponentData,
    {
        self.remove_all_id(ecs_pair(
            First::get_id(self.raw_world),
            Second::get_id(self.raw_world),
        ));
    }

    /// Remove all instances of the given pair from entities
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first id of the pair.
    ///
    /// # See also
    ///
    /// `world::remove_all`
    pub fn remove_all_pair_first_id<Second: CachedComponentData>(&self, first: IdT) {
        self.remove_all_id(ecs_pair(first, Second::get_id(self.raw_world)));
    }

    /// Remove all instances of the given pair from entities
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second`: The second id of the pair.
    ///
    /// # See also
    ///
    /// `world::remove_all`
    pub fn remove_all_pair_second_id<First: CachedComponentData>(&self, second: IdT) {
        self.remove_all_id(ecs_pair(First::get_id(self.raw_world), second));
    }

    /// Remove all instances with the given enum constant from entities
    ///
    /// # Type Parameters
    ///
    /// * `T`: The enum type.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to filter against for removal.
    ///
    /// # See also
    ///
    /// `world::remove_all`
    pub fn remove_all_enum_constant<
        T: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    >(
        &self,
        enum_value: T,
    ) {
        self.remove_all_id(enum_value.get_entity_id_from_enum_field(self.raw_world));
    }

    /// Remove all instances with the given enum tag pair / relationship from entities
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    /// * `Second`: The enum component type.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to filter against for removal.
    ///
    /// # See also
    ///
    /// `world::remove_all`
    pub fn remove_all_enum_tag_pair<First, Second>(&self, enum_value: Second)
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        self.remove_all_id(ecs_pair(
            First::get_id(self.raw_world),
            enum_value.get_entity_id_from_enum_field(self.raw_world),
        ));
    }

    /// Defers all operations executed in the passed-in closure. If the world
    /// is already in deferred mode, does nothing.
    ///
    /// # Arguments
    ///
    /// * `func` - The closure to execute.
    ///
    /// # Examples
    #[cfg_attr(doctest, doc = " ````no_test")]
    /// ```
    /// world.defer(|| {
    ///     // deferred operations here
    /// });
    /// ```
    ///
    /// # See also
    ///
    /// `world::defer`
    pub fn defer<F: FnOnce()>(&self, func: F) {
        unsafe {
            ecs_defer_begin(self.raw_world);
        }
        func();
        unsafe {
            ecs_defer_end(self.raw_world);
        }
    }

    /// Suspends deferring of operations.
    ///
    /// # See also
    ///
    /// `world::defer_suspend`
    pub fn defer_suspend(&self) {
        unsafe {
            ecs_defer_suspend(self.raw_world);
        }
    }

    /// Resumes deferring of operations.
    ///
    /// # See also
    ///
    /// `world::defer_resume`
    pub fn defer_resume(&self) {
        unsafe {
            ecs_defer_resume(self.raw_world);
        }
    }

    /// Checks if the given entity ID exists in the world.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check.
    ///
    /// # Returns
    ///
    /// True if the entity exists, false otherwise.
    ///
    /// # See also
    ///
    /// `world::exists`
    pub fn is_entity_existing(&self, entity: EntityT) -> bool {
        unsafe { ecs_exists(self.raw_world, entity) }
    }

    /// Checks if the given entity ID is alive in the world.
    pub fn is_alive_entity(&self, entity: EntityT) -> bool {
        unsafe { ecs_is_alive(self.raw_world, entity) }
    }

    /// Checks if the given entity ID is valid.
    pub fn is_id_valid(&self, entity: EntityT) -> bool {
        unsafe { ecs_is_valid(self.raw_world, entity) }
    }

    /// Checks if the given entity ID is alive in the world with the current generation.
    pub fn is_entity_alive(&self, entity: EntityT) -> Entity {
        let entity = unsafe { ecs_get_alive(self.raw_world, entity) };
        Entity::new_from_existing_raw(self.raw_world, entity)
    }

    /// get  id of (struct) component.
    pub fn get_id_component<T: CachedComponentData + ComponentType<Struct>>(&self) -> Id {
        Id::new(Some(self), IdType::Id(T::get_id(self.raw_world)))
    }

    /// get pair id from relationship, object.
    pub fn get_id_pair_from_ids(&self, first: EntityT, second: EntityT) -> Id {
        Id::new(Some(self), IdType::Pair(first, second))
    }

    /// get pair id from relationship, object.
    pub fn get_id_pair<First, Second>(&self) -> Id
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        Id::new(
            Some(self),
            IdType::Pair(
                First::get_id(self.raw_world),
                Second::get_id(self.raw_world),
            ),
        )
    }

    /// get pair id from relationship, object.
    pub fn get_id_pair_second_with_id<First: CachedComponentData>(&self, second: EntityT) -> Id {
        Id::new(
            Some(self),
            IdType::Pair(First::get_id(self.raw_world), second),
        )
    }

    /// Ensures that entity with provided generation is alive.
    /// Ths operation will fail if an entity exists with the same id and a
    /// different, non-zero generation.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to ensure is alive.
    ///
    /// # Returns
    ///
    /// The entity with the provided generation.
    ///
    /// # See also
    ///
    /// `world::ensure`
    pub fn ensure_entity_with_generation_is_alive(&self, entity: EntityT) -> Entity {
        unsafe { ecs_ensure(self.raw_world, entity) };
        Entity::new_from_existing_raw(self.raw_world, entity)
    }

    /// Run callback after completing frame
    ///
    /// # Arguments
    ///
    /// * `action` - The action to run.
    /// * `ctx` - The context to pass to the action.
    ///
    /// # See also
    ///
    /// `world::run_post_frame`
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // this doesn't actually deref the pointer
    pub fn run_post_frame(&self, action: ecs_fini_action_t, ctx: *mut c_void) {
        unsafe {
            ecs_run_post_frame(self.raw_world, action, ctx);
        }
    }

    /// Convert enum constant to entity
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum type.
    ///
    /// # Arguments
    ///
    /// * `enum_value` - The enum value to convert.
    ///
    /// # Returns
    ///
    /// Entity wrapping the id of the enum constant.
    ///
    /// # See also
    ///
    /// `world::entity`
    pub fn get_entity_from_enum_constant<T>(&self, enum_value: T) -> Entity
    where
        T: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        Entity::new_from_existing_raw(
            self.raw_world,
            enum_value.get_entity_id_from_enum_field(self.raw_world),
        )
    }

    /// Convert enum constant to id
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum type.
    ///
    /// # Arguments
    ///
    /// * `enum_value` - The enum value to convert.
    ///
    /// # Returns
    ///
    /// The Id type of the enum constant.
    ///
    /// # See also
    ///
    /// `world::id`
    pub fn get_id_from_enum_constant<T>(&self, enum_value: T) -> Id
    where
        T: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        Id::new(
            Some(self),
            IdType::Id(enum_value.get_entity_id_from_enum_field(self.raw_world)),
        )
    }

    /// Convert enum constant to raw id
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum type.
    ///
    /// # Arguments
    ///
    /// * `enum_value` - The enum value to convert.
    ///
    /// # Returns
    ///
    /// The raw id of the enum constant.
    ///
    /// # See also
    ///
    /// `world::id`
    pub fn get_raw_id_from_enum_constant<T>(&self, enum_value: T) -> IdT
    where
        T: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        enum_value.get_entity_id_from_enum_field(self.raw_world)
    }

    /// Create an entity that's associated with a type and name
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to associate with the new entity.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to use for the new entity.
    ///
    /// # See also
    ///
    /// `world::entity`
    pub fn new_entity_named_type<T: CachedComponentData>(&self, name: &CStr) -> Entity {
        Entity::new_from_existing_raw(
            self.raw_world,
            register_entity_w_component_explicit::<T>(self.raw_world, name.as_ptr(), true, 0),
        )
    }

    /// Create an entity that's associated with a name
    ///
    /// # Arguments
    ///
    /// * `name` - The name to use for the new entity.
    ///
    /// # See also
    ///
    /// `world::entity`
    pub fn new_entity_named(&self, name: &CStr) -> Entity {
        Entity::new_named(self, name)
    }

    /// Create a new entity.
    ///
    /// # See also
    ///
    /// `world::entity`
    pub fn new_entity(&self) -> Entity {
        Entity::new(self)
    }

    /// Create a new entity with the provided id.
    ///
    /// # Arguments
    ///
    /// * `id` - The id to use for the new entity.
    ///
    /// # See also
    ///
    /// `world::entity`
    pub fn new_entity_w_id(&self, id: EntityT) -> Entity {
        Entity::new_from_existing_raw(self.raw_world, id)
    }
}

impl World {
    /// Find or register component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    pub fn component<T: CachedComponentData>(&self) -> Component<T> {
        Component::<T>::new(self.raw_world)
    }

    /// Find or register component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the component.
    pub fn component_named<T: CachedComponentData>(&self, name: &CStr) -> Component<T> {
        Component::<T>::new_named(self.raw_world, name)
    }

    /// Find or register untyped component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    pub fn component_untyped<T: CachedComponentData>(&self) -> UntypedComponent {
        UntypedComponent::new(self.raw_world, T::get_id(self.raw_world))
    }

    /// Find or register untyped component.
    ///
    /// # Arguments
    ///
    /// * `id` - The component id.
    pub fn component_untyped_id(&self, id: IdT) -> UntypedComponent {
        UntypedComponent::new(self.raw_world, id)
    }
}

impl World {
    pub fn term_component<T: CachedComponentData>(&self) -> Term {
        Term::new_component::<T>(Some(self))
    }

    pub fn term_pair<First, Second>(&self) -> Term
    where
        First: CachedComponentData,
        Second: CachedComponentData,
    {
        Term::new_pair::<First, Second>(Some(self))
    }
}

// EventBuilder
impl World {
    /// Create a new event.
    ///
    /// # Arguments
    ///
    /// * `event` - The event id
    ///
    /// # Returns
    ///
    /// A new (untyped) event builder.
    ///
    /// # See also
    ///
    /// `world::event`
    pub fn event_untyped(&self, event: EntityT) -> EventBuilder {
        EventBuilder::new(self, event)
    }

    /// Create a new event.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The event type.
    ///
    /// # Returns
    ///
    /// A new (typed) event builder.
    ///
    /// # See also
    ///
    /// `world::event`
    pub fn event<T: EventData + CachedComponentData>(&self) -> EventBuilderTyped<T> {
        EventBuilderTyped::<T>::new(self, T::get_id(self.raw_world))
    }
}

// Observer
impl World {
    /// Upcast entity to an observer.
    /// The provided entity must be an observer.
    ///
    /// # Arguments
    ///
    /// * `e` - The entity.
    ///
    /// # Returns
    ///
    /// An observer object.
    pub fn observer(&self, e: Entity) -> Observer {
        Observer::new_from_existing(self, e)
    }

    /// Create a new observer.
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Returns
    ///
    /// Observer builder.
    pub fn observer_builder<'a, Components>(&self) -> ObserverBuilder<'a, Components>
    where
        Components: Iterable<'a>,
    {
        ObserverBuilder::<'a, Components>::new(self)
    }

    pub fn observer_builder_named<'a, Components>(
        &self,
        name: &CStr,
    ) -> ObserverBuilder<'a, Components>
    where
        Components: Iterable<'a>,
    {
        ObserverBuilder::<'a, Components>::new_named(self, name)
    }
}

/// Systems
#[cfg(feature = "flecs_system")]
impl World {
    /// Constructs a `System` from an existing entity.
    ///
    /// This function upcasts the given `entity` to a `System`, assuming the entity represents a system.
    /// The purpose is to facilitate the interaction with entities that are specifically systems within the ECS.
    ///
    /// # Arguments
    /// * `entity` - An `Entity` that represents a system within the world.
    ///
    /// # See also
    ///
    /// * C++ API: `world::system`
    #[doc(alias = "world::system")]
    pub fn system(&self, entity: Entity) -> System {
        System::new_from_existing(self, entity)
    }

    /// Creates a new `SystemBuilder` instance for constructing systems.
    ///
    /// This function initializes a `SystemBuilder` which is used to create systems that match specific components.
    /// It is a generic method that works with any component types that implement the `Iterable` trait.
    ///
    /// # Type Parameters
    /// - `Components`: The components to match on. Must implement the `Iterable` trait.
    ///
    /// # See also
    ///
    /// * C++ API: `world::system_builder`
    #[doc(alias = "world::system_builder")]
    pub fn system_builder<'a, Components>(&self) -> SystemBuilder<'a, Components>
    where
        Components: Iterable<'a>,
    {
        SystemBuilder::<'a, Components>::new(self)
    }

    /// Creates a new named `SystemBuilder` instance.
    ///
    /// Similar to `system_builder`, but allows naming the system for easier identification and debugging.
    /// The name does not affect the system's behavior.
    ///
    /// # Arguments
    /// * `name` - A string slice representing the name of the system.
    ///
    /// # Type Parameters
    /// - `Components`: The components to match on. Must implement the `Iterable` trait.
    ///
    /// # See also
    ///
    /// * C++ API: `world::system_builder`
    #[doc(alias = "world::system_builder")]
    pub fn system_builder_named<'a, Components>(&self, name: &CStr) -> SystemBuilder<'a, Components>
    where
        Components: Iterable<'a>,
    {
        SystemBuilder::<'a, Components>::new_named(self, name)
    }

    /// Creates a `SystemBuilder` from a system description.
    ///
    /// This function allows creating a system based on a predefined system description,
    /// facilitating more dynamic or configuration-driven system creation.
    ///
    /// # Arguments
    /// * `desc` - A system description that outlines the parameters for the system builder.
    ///
    /// # Type Parameters
    /// - `Components`: The components to match on. Must implement the `Iterable` trait.
    ///
    /// # See also
    ///
    /// * C++ API: `world::system_builder`
    #[doc(alias = "world::system_builder")]
    pub fn system_builder_from_desc<'a, Components>(
        &self,
        desc: ecs_system_desc_t,
    ) -> SystemBuilder<'a, Components>
    where
        Components: Iterable<'a>,
    {
        SystemBuilder::<'a, Components>::new_from_desc(self, desc)
    }
}

/// Pipeline
#[cfg(feature = "flecs_pipeline")]
impl World {
    /// Create a new pipeline.
    ///
    /// # See also
    ///
    /// * C++ API: `world::pipeline`
    #[doc(alias = "world::pipeline")]
    #[inline(always)]
    pub fn pipeline(&self) -> PipelineBuilder<()> {
        PipelineBuilder::<()>::new(self)
    }

    /// Create a new pipeline with the provided associated type
    ///
    /// # Type Parameters
    ///
    /// * `Pipeline` - The associated type to use for the pipeline.
    ///
    /// # See also
    ///
    /// * C++ API: `world::pipeline`
    #[doc(alias = "world::pipeline")]
    #[inline(always)]
    pub fn pipeline_type<Pipeline>(&self) -> PipelineBuilder<()>
    where
        Pipeline: ComponentType<Struct> + CachedComponentData,
    {
        PipelineBuilder::<()>::new_entity(self, Pipeline::get_id(self.raw_world))
    }

    /// Set a custom pipeline. This operation sets the pipeline to run when ecs_progress is invoked.
    ///
    /// # Arguments
    ///
    /// * `pipeline` - The pipeline to set.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_pipeline`
    #[doc(alias = "world::set_pipeline")]
    #[inline(always)]
    pub fn set_pipeline(&self, pipeline: Entity) {
        unsafe {
            super::ecs_set_pipeline(self.raw_world, pipeline.raw_id);
        }
    }

    /// Set a custom pipeline by type. This operation sets the pipeline to run when ecs_progress is invoked.
    ///
    /// # Type Parameters
    ///
    /// * `Pipeline` - The associated type to use for the pipeline.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_pipeline`
    #[doc(alias = "world::set_pipeline")]
    #[inline(always)]
    pub fn set_pipeline_type<Pipeline>(&self)
    where
        Pipeline: ComponentType<Struct> + CachedComponentData,
    {
        unsafe {
            super::ecs_set_pipeline(self.raw_world, Pipeline::get_id(self.raw_world));
        }
    }

    /// Get the current pipeline.
    ///
    /// # Returns
    ///
    /// The current pipeline as an entity.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_pipeline`
    #[doc(alias = "world::get_pipeline")]
    #[inline(always)]
    pub fn get_pipeline(&self) -> Entity {
        Entity::new_from_existing_raw(self.raw_world, unsafe {
            super::ecs_get_pipeline(self.raw_world)
        })
    }

    /// Progress world one tick.
    ///
    /// Progresses the world by running all enabled and periodic systems
    /// on their matching entities. Automatically measures the time passed
    /// since the last frame by passing 0.0 to `delta_time`. This mode is
    /// useful for applications that do not manage time explicitly and want
    /// the system to measure the time automatically.
    ///
    /// # Returns
    ///
    /// True if the world has been progressed, false if `ecs_quit` has been called.
    ///
    /// # See also
    ///
    /// * C++ API: `world::progress`
    #[doc(alias = "world::progress")]
    #[inline(always)]
    pub fn progress(&self) -> bool {
        self.progress_time(0.0)
    }

    /// Progress world by delta time.
    ///
    /// Progresses the world by running all enabled and periodic systems
    /// on their matching entities for the specified time since the last frame.
    /// When `delta_time` is 0, `ecs_progress` will automatically measure the time passed
    /// since the last frame. For applications not using time management, passing a
    /// non-zero `delta_time` (1.0 recommended) skips automatic time measurement to avoid overhead.
    ///
    /// # Arguments
    ///
    /// * `delta_time` - The time to progress the world by. Pass 0.0 for automatic time measurement.
    ///
    /// # Returns
    ///
    /// True if the world has been progressed, false if `ecs_quit` has been called.
    ///
    /// # See also
    ///
    /// * C++ API: `world::progress`
    #[doc(alias = "world::progress")]
    #[inline(always)]
    pub fn progress_time(&self, delta_time: f32) -> bool {
        unsafe { super::ecs_progress(self.raw_world, delta_time) }
    }

    /// Run pipeline.
    /// Runs all systems in the specified pipeline. Can be invoked from multiple
    /// threads if staging is disabled, managing staging and, if needed, thread
    /// synchronization.
    ///
    /// Providing 0 for pipeline id runs the default pipeline (builtin or set via
    /// set_pipeline()). Using progress() auto-invokes this for the default pipeline.
    /// Additional pipelines may be run explicitly.
    ///
    /// # Note
    ///
    /// Only supports single-threaded applications with a single stage when called from an application.
    ///
    /// # Arguments
    ///
    /// * `pipeline` - Pipeline to run.
    ///
    /// # See also
    ///
    /// * C++ API: `world::run_pipeline`
    #[doc(alias = "world::run_pipeline")]
    #[inline(always)]
    pub fn run_pipeline(&self, pipeline: Entity) {
        Self::run_pipeline_time(self, pipeline, 0.0)
    }

    /// Run pipeline.
    /// Runs all systems in the specified pipeline. Can be invoked from multiple
    /// threads if staging is disabled, managing staging and, if needed, thread
    /// synchronization.
    ///
    /// Providing 0 for pipeline id runs the default pipeline (builtin or set via
    /// set_pipeline()). Using progress() auto-invokes this for the default pipeline.
    /// Additional pipelines may be run explicitly.
    ///
    /// # Note
    ///
    /// Only supports single-threaded applications with a single stage when called from an application.
    ///
    /// # Arguments
    ///
    /// * `pipeline` - Pipeline to run.
    /// * `delta_time` - Time to advance the world.
    ///
    /// # See also
    ///
    /// * C++ API: `world::run_pipeline`
    #[doc(alias = "world::run_pipeline")]
    #[inline(always)]
    pub fn run_pipeline_time(&self, pipeline: Entity, delta_time: super::FTime) {
        unsafe {
            super::ecs_run_pipeline(self.raw_world, pipeline.raw_id, delta_time);
        }
    }

    /// Set time scale. Increase or decrease simulation speed by the provided multiplier.
    ///
    /// # Arguments
    ///
    /// * `mul` - The multiplier to set the time scale to.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_time_scale`
    #[doc(alias = "world::set_time_scale")]
    #[inline(always)]
    pub fn set_time_scale(&self, mul: super::FTime) {
        unsafe {
            super::ecs_set_time_scale(self.raw_world, mul);
        }
    }

    /// Get time scale.
    ///
    /// Retrieves the current time scale of the world, which affects the speed
    /// at which time passes within the simulation. A time scale of 1.0 means
    /// real-time, values greater than 1.0 speed up the simulation, and values
    /// less than 1.0 slow it down.
    ///
    /// # Returns
    ///
    /// The current time scale as a floating point number.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_time_scale`
    #[doc(alias = "world::get_time_scale")]
    #[inline(always)]
    pub fn get_time_scale(&self) -> super::FTime {
        let stats = unsafe { super::ecs_get_world_info(self.raw_world) };
        unsafe { (*stats).time_scale }
    }

    /// Get current tick.
    ///
    /// Retrieves the total number of simulation ticks (frames) that have been
    /// processed by the world. This can be used to track the simulation's
    /// progress over time.
    ///
    /// # Returns
    ///
    /// The total number of ticks as an integer.
    #[inline(always)]
    pub fn get_tick(&self) -> i64 {
        let stats = unsafe { super::ecs_get_world_info(self.raw_world) };
        unsafe { (*stats).frame_count_total }
    }

    /// Get target frames per second (FPS).
    ///
    /// Retrieves the target FPS for the world. This value is used to calculate
    /// the time step for each simulation tick when the automatic time step is
    /// enabled. Adjusting the target FPS can be used to control simulation
    /// speed.
    ///
    /// # Returns
    ///
    /// The target FPS as a floating point number.
    #[inline(always)]
    pub fn get_target_fps(&self) -> super::FTime {
        let stats = unsafe { super::ecs_get_world_info(self.raw_world) };
        unsafe { (*stats).target_fps }
    }

    /// Set target frames per second (FPS).
    ///
    /// Configures the world to run at the specified target FPS, ensuring that
    /// `ecs_progress` is not called more frequently than this rate. This mechanism
    /// enables tracking the elapsed time since the last `ecs_progress` call and
    /// sleeping for any remaining time in the frame, if applicable.
    ///
    /// Utilizing this feature promotes consistent system execution intervals and
    /// conserves CPU resources by avoiding more frequent system runs than necessary.
    ///
    /// It's important to note that `ecs_progress` will only introduce sleep periods
    /// when there is surplus time within a frame. This accounts for time consumed both
    /// within Flecs and in external operations.
    ///
    /// # Arguments
    ///
    /// * `world` - The world context.
    /// * `fps` - The desired target FPS as a floating-point number.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_target_fps`
    #[doc(alias = "world::set_target_fps")]
    #[inline(always)]
    pub fn set_target_fps(&self, target_fps: super::FTime) {
        unsafe {
            super::ecs_set_target_fps(self.raw_world, target_fps);
        }
    }

    /// Reset world clock. Reset the clock that keeps track of the total time passed in the simulation.
    ///
    /// # See also
    ///
    /// * C++ API: `world::reset_clock`
    #[doc(alias = "world::reset_clock")]
    #[inline(always)]
    pub fn reset_clock(&self) {
        unsafe {
            super::ecs_reset_clock(self.raw_world);
        }
    }

    /// Set number of worker threads.
    ///
    /// Setting this value to a value higher than 1 will start as many threads and
    /// will cause systems to evenly distribute matched entities across threads.
    /// The operation may be called multiple times to reconfigure the number of threads used,
    /// but never while running a system / pipeline. Calling ecs_set_threads will also end the use
    /// of task threads setup with ecs_set_task_threads and vice-versa
    ///
    /// # Arguments
    ///
    /// * `threads` - The number of threads to use.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_threads`
    #[doc(alias = "world::set_threads")]
    #[inline(always)]
    pub fn set_threads(&self, threads: i32) {
        unsafe {
            super::ecs_set_threads(self.raw_world, threads);
        }
    }

    /// Get number of configured stages. Return number of stages set by ecs_set_stage_count.
    ///
    /// # Returns
    ///
    /// The number of stages as an integer.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_threads`
    #[doc(alias = "world::get_threads")]
    #[inline(always)]
    pub fn get_threads(&self) -> i32 {
        unsafe { super::ecs_get_stage_count(self.raw_world) }
    }

    /// Set number of worker task threads.
    ///
    /// Configures the world to use a specified number of short-lived task threads,
    /// distinct from `ecs_set_threads` where threads persist. Here, threads are
    /// created and joined for each world update, leveraging the `os_api_t` tasks
    /// APIs for task management instead of traditional thread APIs. This approach
    /// is advantageous for integrating with external asynchronous job systems,
    /// allowing for the dynamic creation and synchronization of tasks specific to
    /// each world update.
    ///
    /// This function can be invoked multiple times to adjust the count of task threads,
    /// but must not be called concurrently with system or pipeline execution. Switching
    /// to `ecs_set_task_threads` from `ecs_set_threads` (or vice versa) will terminate
    /// the use of the previously configured threading model.
    ///
    /// # Arguments
    ///
    /// * `task_threads` - The number of task threads to use.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_task_threads`
    #[doc(alias = "world::set_task_threads")]
    #[inline(always)]
    pub fn set_task_threads(&self, task_threads: i32) {
        unsafe {
            super::ecs_set_task_threads(self.raw_world, task_threads);
        }
    }

    /// Returns true if task thread use have been requested.
    ///
    /// # Returns
    ///
    /// True if task threads are being used, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `world::using_task_threads`
    #[doc(alias = "world::using_task_threads")]
    #[inline(always)]
    pub fn using_task_threads(&self) -> bool {
        unsafe { super::ecs_using_task_threads(self.raw_world) }
    }
}
