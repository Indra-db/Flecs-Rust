use std::ops::Deref;

use libc::c_void;

use crate::core::c_binding::bindings::{
    _ecs_poly_is, ecs_get_mut_id, ecs_stage_t_magic, ecs_world_t_magic,
};
use crate::core::utility::errors::FlecsErrorCode;
use crate::ecs_assert;

use super::c_binding::bindings::{
    ecs_async_stage_free, ecs_async_stage_new, ecs_atfini, ecs_defer_begin, ecs_defer_end, ecs_dim,
    ecs_enable_range_check, ecs_fini, ecs_fini_action_t, ecs_frame_begin, ecs_frame_end,
    ecs_get_context, ecs_get_scope, ecs_get_stage, ecs_get_stage_count, ecs_get_stage_id,
    ecs_get_world, ecs_get_world_info, ecs_init, ecs_is_deferred, ecs_lookup_path_w_sep, ecs_merge,
    ecs_quit, ecs_readonly_begin, ecs_readonly_end, ecs_set_automerge, ecs_set_context,
    ecs_set_entity_range, ecs_set_lookup_path, ecs_set_scope, ecs_set_stage_count, ecs_should_quit,
    ecs_stage_is_async, ecs_stage_is_readonly,
};
use super::c_types::{EntityT, IdT, WorldT, SEPARATOR};
use super::component::CachedComponentData;
use super::entity::Entity;
use super::id::Id;

pub struct World {
    pub world: *mut WorldT,
    pub is_owned: bool,
}

impl Default for World {
    fn default() -> Self {
        let world = Self {
            world: unsafe { ecs_init() },
            is_owned: true,
        };
        world.init_builtin_components();
        world
    }
}

impl Deref for World {
    type Target = *mut WorldT;

    fn deref(&self) -> &Self::Target {
        &self.world
    }
}

impl Drop for World {
    fn drop(&mut self) {
        if self.is_owned && unsafe { ecs_stage_is_async(self.world) } {
            unsafe { ecs_async_stage_free(self.world) }
        } else if self.is_owned && !self.world.is_null() {
            unsafe { ecs_fini(self.world) };
        }
    }
}

impl World {
    pub fn new_from_world(world: *mut WorldT) -> Self {
        let world = Self {
            world,
            is_owned: true,
        };
        world.init_builtin_components();
        world
    }

    fn init_builtin_components(&self) {
        #[cfg(feature = "flecs_system")]
        todo!();
        #[cfg(feature = "flecs_timer")]
        todo!();
        #[cfg(feature = "flecs_doc")]
        todo!();
        #[cfg(feature = "flecs_rest")]
        todo!();
        #[cfg(feature = "flecs_meta")]
        todo!();
    }

    /// deletes and recreates the world
    pub fn reset(&mut self) {
        // can only reset the world if we own the world object.
        ecs_assert!(
            self.is_owned,
            FlecsErrorCode::InvalidOperation,
            "Cannot reset a borrowed world"
        );
        unsafe { ecs_fini(self.world) };
        self.world = unsafe { ecs_init() };
    }

    /// obtain pointer to C world object
    pub fn get_as_ptr(&self) -> *mut WorldT {
        self.world
    }

    /// Gets the last delta_time.
    ///
    /// Returns the time that has passed since the last frame.
    pub fn delta_time(&self) -> f32 {
        unsafe {
            let stats = ecs_get_world_info(self.world);
            (*stats).delta_time
        }
    }

    /// Gets the current tick.
    ///
    /// Returns the total number of frames that have passed.
    pub fn tick(&self) -> i64 {
        unsafe {
            let stats = ecs_get_world_info(self.world);
            (*stats).frame_count_total
        }
    }

    /// Gets the current simulation time.
    ///
    /// Returns the total time that has passed in the simulation.
    pub fn time(&self) -> f32 {
        unsafe {
            let stats = ecs_get_world_info(self.world);
            (*stats).world_time_total
        }
    }

    /// Signals the application to quit.
    ///
    /// After calling this function, the next call to progress() returns false.
    pub fn quit(&self) {
        unsafe {
            ecs_quit(self.world);
        }
    }

    /// Registers an action to be executed when the world is destroyed.
    pub fn atfini(&self, action: ecs_fini_action_t, ctx: *mut std::ffi::c_void) {
        unsafe {
            ecs_atfini(self.world, action, ctx);
        }
    }

    /// Tests if `quit` has been called.
    pub fn should_quit(&self) -> bool {
        unsafe { ecs_should_quit(self.world) }
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
    /// ### safety
    /// This function should only be ran from the main thread.
    ///
    /// # Parameters
    /// * `delta_time`: Time elapsed since the last frame.
    ///
    /// # Returns
    /// The provided `delta_time`, or the measured time if 0 was provided.
    pub fn frame_begin(&self, delta_time: f32) -> f32 {
        unsafe { ecs_frame_begin(self.world, delta_time) }
    }

    /// Ends a frame.
    ///
    /// This operation must be called at the end of the frame, and always after
    /// `frame_begin`.
    ///
    /// ### safety
    /// The function should only be run from the main thread.
    pub fn frame_end(&self) {
        unsafe {
            ecs_frame_end(self.world);
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
    /// ### Returns
    /// Whether the world is currently staged.
    pub fn readonly_begin(&self) -> bool {
        unsafe { ecs_readonly_begin(self.world) }
    }

    /// End staging.
    ///
    /// Leaves staging mode. After this operation, the world may be directly mutated again.
    /// By default, this operation also merges data back into the world, unless automerging
    /// was disabled explicitly.
    ///
    /// ## safety
    /// This function should only be run from the main thread.
    pub fn readonly_end(&self) {
        unsafe {
            ecs_readonly_end(self.world);
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
    pub fn defer_begin(&self) -> bool {
        unsafe { ecs_defer_begin(self.world) }
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
    pub fn defer_end(&self) -> bool {
        unsafe { ecs_defer_end(self.world) }
    }

    /// Test whether deferring is enabled.
    pub fn is_deferred(&self) -> bool {
        unsafe { ecs_is_deferred(self.world) }
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
    pub fn set_stage_count(&self, stages: i32) {
        unsafe {
            ecs_set_stage_count(self.world, stages);
        }
    }

    /// Get number of configured stages.
    ///
    /// Return number of stages set by `set_stage_count`.
    ///
    /// # Returns
    ///
    /// The number of stages used for threading.
    pub fn get_stage_count(&self) -> i32 {
        unsafe { ecs_get_stage_count(self.world) }
    }

    /// Get current stage id.
    ///
    /// The stage id can be used by an application to learn about which stage it
    /// is using, which typically corresponds with the worker thread id.
    ///
    /// # Returns
    ///
    /// The stage id.
    pub fn get_stage_id(&self) -> i32 {
        unsafe { ecs_get_stage_id(self.world) }
    }

    /// Test if is a stage.
    ///
    /// If this function returns `false`, it is guaranteed that this is a valid
    /// world object.
    ///
    /// # Returns
    ///
    /// True if the world is a stage, false if not.
    pub fn is_stage(&self) -> bool {
        unsafe {
            ecs_assert!(
                _ecs_poly_is(self.world as *const c_void, ecs_world_t_magic as i32)
                    || _ecs_poly_is(self.world as *const c_void, ecs_stage_t_magic as i32),
                FlecsErrorCode::InvalidParameter,
                "Parameter is not a world or stage"
            );
            _ecs_poly_is(self.world as *const c_void, ecs_stage_t_magic as i32)
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
    pub fn set_automerge(&self, automerge: bool) {
        unsafe { ecs_set_automerge(self.world, automerge) };
    }

    /// Merge world or stage.
    ///
    /// When automatic merging is disabled, an application can call this
    /// operation on either an individual stage, or on the world which will merge
    /// all stages. This operation may only be called when staging is not enabled
    /// (either after `progress()` or after `readonly_end()`).
    ///
    /// This operation may be called on an already merged stage or world.
    pub fn merge(&self) {
        unsafe { ecs_merge(self.world) };
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
    pub fn get_stage(&self, stage_id: i32) -> Self {
        Self {
            world: unsafe { ecs_get_stage(self.world, stage_id) },
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
    pub fn get_async_stage(&self) -> Self {
        let result = Self {
            world: unsafe { ecs_async_stage_new(self.world) },
            is_owned: true,
        };
        result
    }

    /// Get actual world.
    ///
    /// If the current object points to a stage, this operation will return the
    /// actual world.
    ///
    /// # Returns
    ///
    /// The actual world.
    pub fn get_world(&self) -> Self {
        Self {
            world: if !self.world.is_null() {
                unsafe { ecs_get_world(self.world as *const c_void) as *mut WorldT }
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
    pub fn is_readonly(&self) -> bool {
        unsafe { ecs_stage_is_readonly(self.world) }
    }

    /// Set world context.
    ///
    /// Set a context value that can be accessed by anyone that has a reference
    /// to the world.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The world context.
    pub fn set_context(&self, ctx: *mut std::ffi::c_void) {
        unsafe { ecs_set_context(self.world, ctx) }
    }

    /// Get world context.
    ///
    /// # Returns
    ///
    /// The configured world context.
    pub fn get_context(&self) -> *mut std::ffi::c_void {
        unsafe { ecs_get_context(self.world) }
    }

    /// Preallocate memory for a number of entities.
    ///
    /// This function preallocates memory for the entity index.
    ///
    /// # Arguments
    ///
    /// * `entity_count` - Number of entities to preallocate memory for.
    pub fn preallocate_entity_count(&self, entity_count: i32) {
        unsafe { ecs_dim(self.world, entity_count) };
    }

    /// Set the entity range.
    ///
    /// This function limits the range of issued entity IDs between `min` and `max`.
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum entity ID issued.
    /// * `max` - Maximum entity ID issued.
    pub fn set_entity_range(&self, min: EntityT, max: EntityT) {
        unsafe { ecs_set_entity_range(self.world, min, max) };
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
    pub fn enable_range_check(&self, enabled: bool) {
        unsafe { ecs_enable_range_check(self.world, enabled) };
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
    #[inline(always)]
    pub fn set_scope_id(&self, id: EntityT) -> Entity {
        Entity::new_only_id(unsafe { ecs_set_scope(self.world, id) })
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
    #[inline(always)]
    pub fn get_scope_id(&self) -> Entity {
        Entity::new_from_existing(self.world, unsafe { ecs_get_scope(self.world) })
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
    #[inline(always)]
    pub fn set_scope<T: CachedComponentData>(&self) -> Entity {
        Entity::new_only_id(unsafe { ecs_set_scope(self.world, T::get_id(self.world)) })
    }

    /// Sets the search path for entity lookup operations.
    ///
    /// This function configures the search path used for looking up entities. The search path is an array of entity IDs that define the scopes within which lookup operations will search for entities.
    ///
    /// ### Best Practices
    ///
    /// * It's advisable to restore the previous search path after making temporary changes.
    ///
    /// ### Search Path Evaluation
    ///
    /// * The search path is evaluated starting from the last element of the array.
    ///
    /// ### Default Behavior
    ///
    /// * The default search path includes `flecs.core`.
    ///
    /// ### Overwriting
    ///
    /// * Providing a custom search path will overwrite the existing search path.
    ///
    /// ### Considerations
    ///
    /// * If the custom search path doesn't include `flecs.core`, operations that rely on looking up names from `flecs.core` may fail.
    /// * The search path array is not managed by the Rust runtime. Ensure the array remains valid for as long as it is used as the search path.
    ///
    /// ### Array Termination
    ///
    /// * The provided array must be terminated with a 0 element. This allows for pushing/popping elements onto/from an existing array without needing to call `ecs_set_lookup_path` again.
    ///
    /// ### Arguments
    ///
    /// * `search_path` - A 0-terminated array of entity IDs defining the new search path.
    ///
    /// ### Returns
    ///
    /// Returns the current search path after the operation.
    pub fn set_lookup_path(&self, search_path: *const EntityT) -> *mut EntityT {
        unsafe { ecs_set_lookup_path(self.world, search_path) }
    }

    /// Lookup entity by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    pub fn lookup_entity_by_name(&self, name: &str) -> Option<Entity> {
        let c_name = std::ffi::CString::new(name).unwrap();
        let c_name_ptr = c_name.as_ptr();
        let entity_id = unsafe {
            ecs_lookup_path_w_sep(
                self.world,
                0,
                c_name_ptr,
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
                true,
            )
        };
        if entity_id == 0 {
            None
        } else {
            Some(Entity::new_from_existing(self.world, entity_id))
        }
    }

    /// Get id from a type.
    fn get_id<T: CachedComponentData>(&self) -> Id {
        Id::new_from_existing(self.world, T::get_id(self.world))
    }

    /// get pair id from relationship, object.
    fn get_id_pair<T: CachedComponentData, U: CachedComponentData>(&self) -> Id {
        Id::new_world_pair(self.world, T::get_id(self.world), U::get_id(self.world))
    }

    /// get pair id from relationship, object.
    fn get_id_pair_with_id<T: CachedComponentData>(&self, id: EntityT) -> Id {
        Id::new_world_pair(self.world, T::get_id(self.world), id)
    }

    /// get pair id from relationship, object.
    fn get_id_pair_from_ids(&self, id: EntityT, id2: EntityT) -> Id {
        Id::new_world_pair(self.world, id, id2)
    }
}
