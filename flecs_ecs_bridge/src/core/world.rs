use std::ffi::c_char;
use std::ops::Deref;

use libc::c_void;

use crate::core::c_binding::bindings::{
    _ecs_poly_is, ecs_get_mut_id, ecs_stage_t_magic, ecs_world_t_magic,
};
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
};
use super::c_types::{EntityT, IdT, WorldT, SEPARATOR};
use super::component::{
    register_entity_w_component_explicit, CachedComponentData, ComponentType, Enum, Struct,
};
use super::component_ref::Ref;
use super::entity::Entity;
use super::enum_type::CachedEnumData;
use super::id::Id;
use super::scoped_world::ScopedWorld;
use super::utility::functions::set_helper;

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
        //#[cfg(feature = "flecs_system")]
        //todo!();
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

    /// Sets a singleton component of type `T` on the world.
    ///
    /// ### Arguments
    ///
    /// * `component` - The singleton component to set on the world.
    pub fn set_component<T: CachedComponentData>(self, component: T) -> Self {
        let id = T::get_id(self.world);
        set_helper(self.world, id, component, id);
        self
    }

    /// Set a singleton pair using the second element type and a first id.
    ///
    /// ### Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// ### Parameters
    ///
    /// * `first`: The ID of the first element of the pair.
    /// * `second`: The second element of the pair to be set.
    pub fn set_pair_first_id<First>(self, second: EntityT, first: First) -> Self
    where
        First: CachedComponentData + ComponentType<Struct>,
    {
        let entity = Entity::new_from_existing(self.world, First::get_id(self.world));
        entity.set_pair_first_id::<First>(second, first);
        self
    }

    /// Set singleton pair.
    /// This operation sets the pair value, and uses First as type. If it does not yet exist, it will be added.
    ///
    /// ### Type Parameters
    ///
    /// * `First`: The first element of the pair
    /// * `Second`: The second element of the pair
    ///
    /// ### Parameters
    ///
    /// * `first`: The value to set for first component.
    pub fn set_pair_first<First, Second>(self, first: First) -> Self
    where
        First: CachedComponentData + ComponentType<Struct>,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        let entity = Entity::new_from_existing(self.world, First::get_id(self.world));
        entity.set_pair_first::<First, Second>(first);
        self
    }

    /// Set a singleton pair using the second element type and a first id.
    ///
    /// ### Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// ### Parameters
    ///
    /// * `first`: The ID of the first element of the pair.
    /// * `second`: The second element of the pair to be set.
    pub fn set_pair_second_id<Second>(self, first: EntityT, second: Second) -> Self
    where
        Second: CachedComponentData + ComponentType<Struct>,
    {
        let entity = Entity::new_from_existing(self.world, Second::get_id(self.world));
        entity.set_pair_second_id::<Second>(first, second);
        self
    }

    /// Set singleton pair.
    /// This operation sets the pair value, and uses Second as type. If it does not yet exist, it will be added.
    ///
    /// ### Type Parameters
    ///
    /// * `Second`: The second element of the pair
    ///
    /// ### Parameters
    ///
    /// * `first`: The first element of the pair.
    /// * `value`: The value to set.
    pub fn set_pair_second<First, Second>(self, second: Second) -> Self
    where
        First: CachedComponentData + ComponentType<Struct>,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        let entity = Entity::new_from_existing(self.world, First::get_id(self.world));
        entity.set_pair_second::<First, Second>(second);
        self
    }

    /// Signal that singleton component was modified.
    ///
    /// ### Type Parameters
    ///
    /// * `T` - The type of the component that was modified.
    ///
    #[inline(always)]
    pub fn mark_component_modified<T>(self) -> Self
    where
        T: CachedComponentData,
    {
        Entity::new_from_existing(self.world, T::get_id(self.world)).mark_component_modified::<T>();
        self
    }

    /// signal that singleton component was modified.
    ///
    /// ### Arguments
    ///
    /// * `id` - The id of the component that was modified.
    #[inline(always)]
    pub fn mark_component_modified_with_id(&self, id: EntityT) {
        Entity::new_from_existing(self.world, id).mark_component_id_modified(id)
    }

    /// Get a reference to a singleton component.
    ///
    /// A reference allows for quick and safe access to a component value, and is
    /// a faster alternative to repeatedly calling `get` for the same component.
    ///
    /// - `T`: Component for which to get a reference.
    ///
    /// Returns: The reference singleton component.
    #[inline(always)]
    pub fn get_ref_component<T>(&self) -> Ref<T>
    where
        T: CachedComponentData,
    {
        Entity::new_from_existing(self.world, T::get_id(self.world)).get_ref_component::<T>()
    }

    /// Get const pointer for the first element of a singleton pair
    ///
    /// ### Type Parameters
    ///
    /// * `First`: The first part of the pair.
    ///
    /// ### Parameters
    ///
    /// * `second`: The second element of the pair.
    #[inline(always)]
    pub fn get_pair_first<First>(&self, second: EntityT) -> *const First
    where
        First: CachedComponentData,
    {
        Entity::new_from_existing(self.world, First::get_id(self.world))
            .get_pair_first::<First>(second)
    }

    /// Get mutable pointer for the first element of a singleton pair
    ///
    /// ### Type Parameters
    ///
    /// * `First`: The first part of the pair.
    ///
    /// ### Parameters
    ///
    /// * `second`: The second element of the pair.
    #[inline(always)]
    pub fn get_pair_first_mut<First>(&self, second: EntityT) -> *mut First
    where
        First: CachedComponentData,
    {
        Entity::new_from_existing(self.world, First::get_id(self.world))
            .get_pair_first_mut::<First>(second)
    }

    /// Get mutable pointer for the second element of a singleton pair
    ///
    /// ### Type Parameters
    ///
    /// * `second`: The second part of the pair.
    ///
    /// ### Parameters
    ///
    /// * `first`: The first element of the pair.
    #[inline(always)]
    pub fn get_pair_second<Second>(&self, first: EntityT) -> *const Second
    where
        Second: CachedComponentData,
    {
        Entity::new_from_existing(self.world, Second::get_id(self.world))
            .get_pair_second::<Second>(first)
    }

    /// Get mutable pointer for the second element of a singleton pair
    ///
    /// ### Type Parameters
    ///
    /// * `second`: The second part of the pair.
    ///
    /// ### Parameters
    ///
    /// * `first`: The first element of the pair.
    #[inline(always)]
    pub fn get_pair_second_mut<Second>(&self, first: EntityT) -> *mut Second
    where
        Second: CachedComponentData,
    {
        Entity::new_from_existing(self.world, Second::get_id(self.world))
            .get_pair_second_mut::<Second>(first)
    }

    /// Check if world has the provided struct component.
    ///
    /// ### Type Parameters
    ///
    /// * `T` - The component to check.
    ///
    /// ### Returns
    ///
    /// True if the world has the provided component, false otherwise.
    #[inline(always)]
    pub fn has_struct_component<T>(&self) -> bool
    where
        T: CachedComponentData + ComponentType<Struct>,
    {
        Entity::new_from_existing(self.world, T::get_id(self.world)).has_struct_component::<T>()
    }

    /// Check if world has the provided enum component.
    ///
    /// ### Type Parameters
    ///
    /// * `T` - The component to check.
    ///
    /// ### Returns
    ///
    /// True if the world has the enum provided component, false otherwise.
    #[inline(always)]
    pub fn has_enum_component<T>(&self) -> bool
    where
        T: CachedComponentData + ComponentType<Enum>,
    {
        Entity::new_from_existing(self.world, T::get_id(self.world)).has_enum_component::<T>()
    }

    /// Check if world has the provided enum constant.
    ///
    /// ### Type Parameters
    ///
    /// * `T` - The enum type.
    ///
    /// ### Arguments
    ///
    /// * `constant` - The enum constant to check.
    ///
    /// ### Returns
    ///
    /// True if the world has the provided constant, false otherwise.
    #[inline(always)]
    pub fn has_enum_constant_component<T>(&self, constant: T) -> bool
    where
        T: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        Entity::new_from_existing(self.world, T::get_id(self.world))
            .has_enum_constant::<T>(constant)
    }

    /// Check if world has the provided pair.
    ///
    /// ### Type Parameters
    ///
    /// * `T` - The first element of the pair.
    /// * `U` - The second element of the pair.
    ///
    /// ### Returns
    ///
    /// True if the world has the provided component, false otherwise.
    #[inline(always)]
    pub fn has_pair_component<First, Second>(&self) -> bool
    where
        First: CachedComponentData + ComponentType<Struct>,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        Entity::new_from_existing(self.world, First::get_id(self.world)).has_pair::<First, Second>()
    }

    /// Check if world has the provided pair.
    ///
    /// ### Arguments
    ///
    /// * `first`: The first element of the pair.
    /// * `second`: The second element of the pair.
    ///
    /// ### Returns
    ///
    /// True if the world has the provided component, false otherwise.
    #[inline(always)]
    pub fn has_pair_by_id(&self, first: EntityT, second: EntityT) -> bool {
        Entity::new_from_existing(self.world, first).has_pair_by_ids(first, second)
    }

    #[inline(always)]
    pub fn add_component<T: CachedComponentData>(self) -> Self {
        Entity::new_from_existing(self.world, T::get_id(self.world)).add_component::<T>();
        self
    }

    #[inline(always)]
    pub fn add_pair_ids(self, first: EntityT, second: EntityT) -> Self {
        Entity::new_from_existing(self.world, first).add_pair_ids(first, second);
        self
    }

    #[inline(always)]
    pub fn add_pair<First, Second>(self) -> Self
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        Entity::new_from_existing(self.world, First::get_id(self.world))
            .add_pair::<First, Second>();
        self
    }

    #[inline(always)]
    pub fn add_pair_first_id<Second: CachedComponentData>(self, first: EntityT) -> Self {
        Entity::new_from_existing(self.world, Second::get_id(self.world))
            .add_pair_first_id::<Second>(first);
        self
    }

    #[inline(always)]
    pub fn add_pair_second_id<First: CachedComponentData>(self, second: EntityT) -> Self {
        Entity::new_from_existing(self.world, First::get_id(self.world))
            .add_pair_second_id::<First>(second);
        self
    }

    #[inline(always)]
    pub fn add_enum_tag<First, Second>(self, enum_value: Second) -> Self
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        Entity::new_from_existing(self.world, First::get_id(self.world))
            .add_enum_tag::<First, Second>(enum_value);
        self
    }

    #[inline(always)]
    pub fn add_enum_constant<T: CachedComponentData + ComponentType<Enum> + CachedEnumData>(
        self,
        enum_value: T,
    ) -> Self {
        Entity::new_from_existing(self.world, T::get_id(self.world))
            .add_enum_constant::<T>(enum_value);
        self
    }

    #[inline(always)]
    pub fn remove_component<T: CachedComponentData + ComponentType<Struct>>(self) -> Self {
        Entity::new_from_existing(self.world, T::get_id(self.world)).remove_component::<T>();
        self
    }

    #[inline(always)]
    pub fn remove_component_enum<T: CachedComponentData + ComponentType<Enum>>(self) -> Self {
        Entity::new_from_existing(self.world, T::get_id(self.world)).remove_component_enum::<T>();
        self
    }

    #[inline(always)]
    pub fn remove_enum_tag<First, Second>(self, enum_value: Second) -> Self
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        Entity::new_from_existing(self.world, First::get_id(self.world))
            .remove_enum_tag::<First, Second>(enum_value);
        self
    }

    #[inline(always)]
    pub fn remove_pair_ids(self, first: EntityT, second: EntityT) -> Self {
        Entity::new_from_existing(self.world, first).remove_pair_ids(first, second);
        self
    }

    #[inline(always)]
    pub fn remove_pair<First, Second>(self) -> Self
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        Entity::new_from_existing(self.world, First::get_id(self.world))
            .remove_pair::<First, Second>();
        self
    }

    #[inline(always)]
    pub fn remove_pair_first_id<Second: CachedComponentData>(self, first: EntityT) -> Self {
        Entity::new_from_existing(self.world, Second::get_id(self.world))
            .remove_pair_first_id::<Second>(first);
        self
    }

    #[inline(always)]
    pub fn remove_pair_second_id<First: CachedComponentData>(self, second: EntityT) -> Self {
        Entity::new_from_existing(self.world, First::get_id(self.world))
            .remove_pair_second_id::<First>(second);
        self
    }

    /// Iterate children in root of world
    /// This operation follows the ChildOf relationship.
    /// ### Arguments
    ///
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(Entity)`.
    #[inline(always)]
    pub fn for_each_children_by_relationship<T: CachedComponentData, F: FnMut(Entity)>(
        &self,
        callback: F,
    ) {
        Entity::new(self.world).for_each_child_of(callback);
    }

    /// create alias for component
    ///
    /// ### Type Parameters
    ///
    /// * `T` - The component type to create an alias for.
    ///
    /// ### Arguments
    ///
    /// * `alias` - The alias to create.
    ///
    /// ### Returns
    ///
    /// The entity representing the component.
    #[inline(always)]
    pub fn set_alias_component<T: CachedComponentData>(&self, alias: &str) -> Entity {
        let id = T::get_id(self.world);
        if alias.is_empty() {
            unsafe { ecs_set_alias(self.world, id, ecs_get_name(self.world, id)) };
        } else {
            let c_alias = std::ffi::CString::new(alias).unwrap();
            let c_alias_ptr = c_alias.as_ptr();
            unsafe { ecs_set_alias(self.world, id, c_alias_ptr) };
        }
        Entity::new_from_existing(self.world, id)
    }

    /// create alias for entity by name
    ///
    /// ### Arguments
    ///
    /// * `name` - The name of the entity to create an alias for.
    /// * `alias` - The alias to create.
    ///
    /// ### Returns
    ///
    /// The entity found by name.
    #[inline(always)]
    pub fn set_alias_entity_by_name(&self, name: &str, alias: &str) -> Entity {
        let c_name = std::ffi::CString::new(name).unwrap();
        let c_name_ptr = c_name.as_ptr();
        let c_alias = std::ffi::CString::new(alias).unwrap();
        let c_alias_ptr = c_alias.as_ptr();
        let id = unsafe {
            ecs_lookup_path_w_sep(
                self.world,
                0,
                c_name_ptr,
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
                true,
            )
        };
        ecs_assert!(id != 0, FlecsErrorCode::InvalidParameter);
        unsafe { ecs_set_alias(self.world, id, c_alias_ptr) };
        Entity::new_from_existing(self.world, id)
    }

    /// create alias for entity
    ///
    /// ### Arguments
    ///
    /// * `entity` - The entity to create an alias for.
    /// * `alias` - The alias to create.
    #[inline(always)]
    pub fn set_alias_entity(&self, entity: Entity, alias: &str) {
        if alias.is_empty() {
            unsafe {
                ecs_set_alias(
                    self.world,
                    entity.raw_id,
                    ecs_get_name(self.world, entity.raw_id),
                )
            };
        } else {
            let c_alias = std::ffi::CString::new(alias).unwrap();
            let c_alias_ptr = c_alias.as_ptr();
            unsafe { ecs_set_alias(self.world, entity.raw_id, c_alias_ptr) };
        }
    }

    pub fn count_component_id(&self, component_id: IdT) -> i32 {
        unsafe { ecs_count_id(self.world, component_id) }
    }

    pub fn count_component<T: CachedComponentData + ComponentType<Struct>>(&self) -> i32 {
        self.count_component_id(T::get_id(self.world))
    }

    pub fn count_pair_ids(&self, first: EntityT, second: EntityT) -> i32 {
        self.count_component_id(ecs_pair(first, second))
    }

    pub fn count_pair<First, Second>(&self) -> i32
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        self.count_pair_ids(First::get_id(self.world), Second::get_id(self.world))
    }

    pub fn count_pair_first_id<Second: CachedComponentData>(&self, first: EntityT) -> i32 {
        self.count_pair_ids(first, Second::get_id(self.world))
    }

    pub fn count_pair_second_id<First: CachedComponentData>(&self, second: EntityT) -> i32 {
        self.count_pair_ids(First::get_id(self.world), second)
    }

    pub fn count_enum_constant<T: CachedComponentData + ComponentType<Enum> + CachedEnumData>(
        &self,
        enum_value: T,
    ) -> i32 {
        unsafe {
            ecs_count_id(
                self.world,
                enum_value.get_entity_id_from_enum_field(self.world),
            )
        }
    }

    pub fn count_enum_tag_pair<First, Second>(&self, enum_value: Second) -> i32
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        unsafe {
            ecs_count_id(
                self.world,
                ecs_pair(
                    First::get_id(self.world),
                    enum_value.get_entity_id_from_enum_field(self.world),
                ),
            )
        }
    }

    pub fn scope<F: FnMut()>(&self, parent_id: IdT, mut func: F) {
        let prev: IdT = unsafe { ecs_set_scope(self.world, parent_id) };
        func();
        unsafe {
            ecs_set_scope(self.world, prev);
        }
    }

    pub fn scope_with<T: CachedComponentData, F: FnMut()>(&self, func: F) {
        self.scope(T::get_id(self.world), func);
    }

    pub fn get_scoped_world_id(&self, parent_id: IdT) -> ScopedWorld {
        ScopedWorld::new(self.world, parent_id)
    }

    pub fn get_scoped_world<T: CachedComponentData>(&self) -> ScopedWorld {
        self.get_scoped_world_id(T::get_id(self.world))
    }

    /// all entities created in function are created with id
    pub fn with_id<F: FnMut()>(&self, with_id: IdT, mut func: F) {
        let prev: IdT = unsafe { ecs_set_with(self.world, with_id) };
        func();
        unsafe {
            ecs_set_with(self.world, prev);
        }
    }

    pub fn with<T: CachedComponentData, F: FnMut()>(&self, func: F) {
        self.with_id(T::get_id(self.world), func);
    }

    pub fn with_pair_ids<F: FnMut()>(&self, first: IdT, second: IdT, func: F) {
        self.with_id(ecs_pair(first, second), func);
    }

    pub fn with_pair<First, Second, F: FnMut()>(&self, func: F)
    where
        First: CachedComponentData,
        Second: CachedComponentData,
    {
        self.with_id(
            ecs_pair(First::get_id(self.world), Second::get_id(self.world)),
            func,
        );
    }

    pub fn with_pair_first_id<Second: CachedComponentData, F: FnMut()>(&self, first: IdT, func: F) {
        self.with_id(ecs_pair(first, Second::get_id(self.world)), func);
    }

    pub fn with_pair_second_id<First: CachedComponentData, F: FnMut()>(
        &self,
        second: IdT,
        func: F,
    ) {
        self.with_id(ecs_pair(First::get_id(self.world), second), func);
    }

    pub fn with_enum_constant<T, F>(&self, enum_value: T, func: F)
    where
        T: CachedComponentData + ComponentType<Enum> + CachedEnumData,
        F: FnMut(),
    {
        self.with_id(enum_value.get_entity_id_from_enum_field(self.world), func);
    }

    pub fn with_enum_tag_pair<First, Second, F>(&self, enum_value: Second, func: F)
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Enum> + CachedEnumData,
        F: FnMut(),
    {
        self.with_id(
            ecs_pair(
                First::get_id(self.world),
                enum_value.get_entity_id_from_enum_field(self.world),
            ),
            func,
        );
    }

    pub fn delete_with_id(&self, with_id: IdT) {
        unsafe {
            ecs_delete_with(self.world, with_id);
        }
    }

    pub fn delete<T: CachedComponentData>(&self) {
        self.delete_with_id(T::get_id(self.world));
    }

    pub fn delete_pair_ids(&self, first: IdT, second: IdT) {
        self.delete_with_id(ecs_pair(first, second));
    }

    pub fn delete_pair<First, Second>(&self)
    where
        First: CachedComponentData,
        Second: CachedComponentData,
    {
        self.delete_with_id(ecs_pair(
            First::get_id(self.world),
            Second::get_id(self.world),
        ));
    }

    pub fn delete_pair_first_id<Second: CachedComponentData>(&self, first: IdT) {
        self.delete_with_id(ecs_pair(first, Second::get_id(self.world)));
    }

    pub fn delete_pair_second_id<First: CachedComponentData>(&self, second: IdT) {
        self.delete_with_id(ecs_pair(First::get_id(self.world), second));
    }

    pub fn delete_enum_constant<T: CachedComponentData + ComponentType<Enum> + CachedEnumData>(
        &self,
        enum_value: T,
    ) {
        self.delete_with_id(enum_value.get_entity_id_from_enum_field(self.world));
    }

    pub fn delete_enum_tag_pair<First, Second>(&self, enum_value: Second)
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        self.delete_with_id(ecs_pair(
            First::get_id(self.world),
            enum_value.get_entity_id_from_enum_field(self.world),
        ));
    }

    pub fn remove_all_ids(&self, id: IdT) {
        unsafe {
            ecs_remove_all(self.world, id);
        }
    }

    pub fn remove_all<T: CachedComponentData>(&self) {
        self.remove_all_ids(T::get_id(self.world));
    }

    pub fn remove_all_pair_ids(&self, first: IdT, second: IdT) {
        self.remove_all_ids(ecs_pair(first, second));
    }

    pub fn remove_all_pair<First, Second>(&self)
    where
        First: CachedComponentData,
        Second: CachedComponentData,
    {
        self.remove_all_ids(ecs_pair(
            First::get_id(self.world),
            Second::get_id(self.world),
        ));
    }

    pub fn remove_all_pair_first_id<Second: CachedComponentData>(&self, first: IdT) {
        self.remove_all_ids(ecs_pair(first, Second::get_id(self.world)));
    }

    pub fn remove_all_pair_second_id<First: CachedComponentData>(&self, second: IdT) {
        self.remove_all_ids(ecs_pair(First::get_id(self.world), second));
    }

    pub fn remove_all_enum_constant<
        T: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    >(
        &self,
        enum_value: T,
    ) {
        self.remove_all_ids(enum_value.get_entity_id_from_enum_field(self.world));
    }

    pub fn remove_all_enum_tag_pair<First, Second>(&self, enum_value: Second)
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        self.remove_all_ids(ecs_pair(
            First::get_id(self.world),
            enum_value.get_entity_id_from_enum_field(self.world),
        ));
    }

    /// Defers all operations executed in the passed-in closure. If the world
    /// is already in deferred mode, does nothing.
    //
    /// # Examples
    #[cfg_attr(doctest, doc = " ````no_test")]
    /// ```
    /// world.defer(|| {
    ///     // deferred operations here
    /// });
    /// ```

    pub fn defer<F: FnOnce()>(&self, func: F) {
        unsafe {
            ecs_defer_begin(self.world);
        }
        func();
        unsafe {
            ecs_defer_end(self.world);
        }
    }

    /// Suspends deferring of operations.
    pub fn defer_suspend(&self) {
        unsafe {
            ecs_defer_suspend(self.world);
        }
    }

    /// Resumes deferring of operations.
    pub fn defer_resume(&self) {
        unsafe {
            ecs_defer_resume(self.world);
        }
    }

    /// Checks if the given entity ID exists in the world.
    pub fn exists(&self, entity: EntityT) -> bool {
        unsafe { ecs_exists(self.world, entity) }
    }

    /// Checks if the given entity ID is alive in the world.
    pub fn is_alive_entity(&self, entity: EntityT) -> bool {
        unsafe { ecs_is_alive(self.world, entity) }
    }

    /// Checks if the given entity ID is valid.
    pub fn is_valid(&self, entity: EntityT) -> bool {
        unsafe { ecs_is_valid(self.world, entity) }
    }

    /// Checks if the given entity ID is alive in the world with the current generation.
    pub fn get_alive_entity(&self, entity: EntityT) -> Entity {
        let entity = unsafe { ecs_get_alive(self.world, entity) };
        Entity::new_from_existing(self.world, entity)
    }

    /// get  id of (struct) component.
    pub fn get_id_component<T: CachedComponentData + ComponentType<Struct>>(&self) -> Id {
        Id::new_from_existing(self.world, T::get_id(self.world))
    }

    /// get pair id from relationship, object.
    pub fn get_id_pair_from_ids(&self, first: EntityT, second: EntityT) -> Id {
        Id::new_world_pair(self.world, first, second)
    }

    /// get pair id from relationship, object.
    pub fn get_id_pair<First, Second>(&self) -> Id
    where
        First: CachedComponentData,
        Second: CachedComponentData + ComponentType<Struct>,
    {
        Id::new_world_pair(
            self.world,
            First::get_id(self.world),
            Second::get_id(self.world),
        )
    }

    /// get pair id from relationship, object.
    pub fn get_id_pair_with_id<First: CachedComponentData>(&self, second: EntityT) -> Id {
        Id::new_world_pair(self.world, First::get_id(self.world), second)
    }

    /// Ensures that entity with provided generation is alive.
    /// Ths operation will fail if an entity exists with the same id and a
    /// different, non-zero generation.
    pub fn ensure_entity(&self, entity: EntityT) -> Entity {
        unsafe { ecs_ensure(self.world, entity) };
        Entity::new_from_existing(self.world, entity)
    }

    /// Run callback after completing frame
    pub fn run_post_frame(&self, action: ecs_fini_action_t, ctx: *mut c_void) {
        unsafe {
            ecs_run_post_frame(self.world, action, ctx);
        }
    }

    //convert enum constant to entity
    pub fn get_entity_from_enum_constant<T>(&self, enum_value: T) -> Entity
    where
        T: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        Entity::new_from_existing(
            self.world,
            enum_value.get_entity_id_from_enum_field(self.world),
        )
    }

    //convert enum constant to id
    pub fn get_id_from_enum_constant<T>(&self, enum_value: T) -> Id
    where
        T: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        Id::new_from_existing(
            self.world,
            enum_value.get_entity_id_from_enum_field(self.world),
        )
    }

    //convert enum constant to raw id
    pub fn get_raw_id_from_enum_constant<T>(&self, enum_value: T) -> IdT
    where
        T: CachedComponentData + ComponentType<Enum> + CachedEnumData,
    {
        enum_value.get_entity_id_from_enum_field(self.world)
    }

    //create an entity that's associated with a type and name
    pub fn new_entity_named_type<T: CachedComponentData>(&self, name: &str) -> Entity {
        let c_name = std::ffi::CString::new(name).unwrap();
        let c_name_ptr = c_name.as_ptr();

        Entity::new_from_existing(
            self.world,
            register_entity_w_component_explicit::<T>(self.world, c_name_ptr, true, 0),
        )
    }

    //create an entity that's associated with a name
    pub fn new_entity_named(&self, name: &str) -> Entity {
        Entity::new_named(self.world, name)
    }

    pub fn new_entity(&self) -> Entity {
        Entity::new(self.world)
    }
}
