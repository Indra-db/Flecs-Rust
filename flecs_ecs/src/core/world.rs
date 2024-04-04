//! World operations.

use std::{ffi::CStr, ops::Deref, os::raw::c_void};

#[cfg(feature = "flecs_app")]
use crate::addons::app::App;

#[cfg(feature = "flecs_system")]
use crate::{
    addons::system::{System, SystemBuilder},
    sys::ecs_system_desc_t,
};

#[cfg(feature = "flecs_pipeline")]
use crate::{addons::pipeline::PipelineBuilder, sys};

use crate::ecs_assert;
use crate::sys::{
    ecs_async_stage_free, ecs_async_stage_new, ecs_atfini, ecs_count_id, ecs_ctx_free_t,
    ecs_defer_begin, ecs_defer_end, ecs_defer_resume, ecs_defer_suspend, ecs_delete_with, ecs_dim,
    ecs_enable_range_check, ecs_ensure, ecs_exists, ecs_fini, ecs_fini_action_t, ecs_frame_begin,
    ecs_frame_end, ecs_get_alive, ecs_get_ctx, ecs_get_id, ecs_get_mut_id, ecs_get_name,
    ecs_get_scope, ecs_get_stage, ecs_get_stage_count, ecs_get_stage_id, ecs_get_target,
    ecs_get_world, ecs_get_world_info, ecs_init, ecs_is_alive, ecs_is_deferred, ecs_is_valid,
    ecs_lookup_path_w_sep, ecs_merge, ecs_poly_is_, ecs_quit, ecs_readonly_begin, ecs_readonly_end,
    ecs_remove_all, ecs_run_post_frame, ecs_set_alias, ecs_set_automerge, ecs_set_ctx,
    ecs_set_entity_range, ecs_set_lookup_path, ecs_set_scope, ecs_set_stage_count, ecs_set_with,
    ecs_should_quit, ecs_stage_is_async, ecs_stage_is_readonly, ecs_stage_t_magic,
    ecs_world_info_t,
};
#[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
use crate::{
    core::{ecs_is_pair, FlecsErrorCode},
    sys::ecs_world_t_magic,
};

use super::{
    c_types::{EntityT, IdT, WorldT, SEPARATOR},
    component::{Component, UntypedComponent},
    component_ref::Ref,
    component_registration::{ComponentId, ComponentType, Enum, Struct},
    IntoComponentId, IntoEntityId, IntoEntityIdExt, IterAPI, ECS_PREFAB,
};
use super::{EmptyComponent, NotEmptyComponent};

use super::{
    ecs_pair,
    entity::Entity,
    event::EventData,
    event_builder::{EventBuilder, EventBuilderTyped},
    id::Id,
    iterable::Iterable,
    observer::Observer,
    observer_builder::ObserverBuilder,
    scoped_world::ScopedWorld,
    set_helper,
    term::Term,
    Builder, CachedEnumData, Filter, FilterBuilder, Query, QueryBuilder,
};

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
    pub fn new() -> Self {
        Self::default()
    }

    /// Wrapper around raw world, takes no ownership.
    pub fn new_wrap_raw_world(world: *mut WorldT) -> Self {
        Self {
            raw_world: world,
            is_owned: false,
        }
    }

    fn init_builtin_components(&self) {
        #[cfg(feature = "flecs_system")]
        System::system_init(self);
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
    /// * C++ API: `world::reset`
    #[doc(alias = "world::reset")]
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
    /// * C++ API: `world::c_ptr`
    #[doc(alias = "world::c_ptr")]
    pub fn get_as_ptr(&self) -> *mut WorldT {
        self.raw_world
    }

    /// Get the world's info.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_info`
    #[doc(alias = "world::get_info")]
    fn get_world_info(&self) -> &ecs_world_info_t {
        // SAFETY: The pointer is valid for the lifetime of the world.
        unsafe { &*ecs_get_world_info(self.raw_world) }
    }

    /// Gets the last `delta_time`.
    ///
    /// Returns the time that has passed since the last frame.
    ///
    /// # See also
    ///
    /// * C++ API: `world::delta_time`
    #[doc(alias = "world::delta_time")]
    pub fn get_delta_time(&self) -> f32 {
        self.get_world_info().delta_time
    }

    /// Signals the application to quit.
    ///
    /// After calling this function, the next call to `progress()` returns false.
    ///
    /// # See also
    ///
    /// * C++ API: `world::quit`
    #[doc(alias = "world::quit")]
    pub fn quit(&self) {
        unsafe {
            ecs_quit(self.raw_world);
        }
    }

    /// Registers an action to be executed when the world is destroyed.
    ///
    /// # See also
    ///
    /// * C++ API: `world::atfini`
    #[doc(alias = "world::atfini")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // this doesn't actually deref the pointer
    pub fn on_destroyed(&self, action: ecs_fini_action_t, ctx: *mut c_void) {
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
    /// * C++ API: `world::should_quit`
    #[doc(alias = "world::should_quit")]
    pub fn should_quit(&self) -> bool {
        unsafe { ecs_should_quit(self.raw_world) }
    }

    /// Begins a frame.
    ///
    /// When an application does not use `progress()` to control the main loop, it
    /// can still use Flecs features such as FPS limiting and time measurements.
    /// can still use Flecs features such as FPS limiting and time measurements processed.
    ///
    /// Calls to `frame_begin` must always be followed by `frame_end`.
    ///
    /// The function accepts a `delta_time` parameter, which will get passed to
    /// systems. This value is also used to compute the amount of time the
    /// function needs to sleep to ensure it does not exceed the `target_fps`, when
    /// it is set. When 0 is provided for `delta_time`, the time will be measured.
    ///
    /// # Safety
    /// This function should only be ran from the main thread.
    ///
    /// # Arguments
    /// * `delta_time`: Time elapsed since the last frame.
    ///
    /// # Returns
    /// The provided `delta_time`, or the measured time if 0 was provided.
    ///
    /// # See also
    ///
    /// * C++ API: `world::frame_begin`
    #[doc(alias = "world::frame_begin")]
    pub fn frame_begin(&self, delta_time: f32) -> f32 {
        unsafe { ecs_frame_begin(self.raw_world, delta_time) }
    }

    /// Ends a frame.
    ///
    /// This operation must be called at the end of the frame, and always after `frame_begin`
    ///
    /// # Safety
    /// The function should only be run from the main thread.
    ///
    /// # See also
    ///
    /// * C++ API: `world::frame_end`
    #[doc(alias = "world::frame_end")]
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
    /// * C++ API: `world::readonly_begin`
    #[doc(alias = "world::readonly_begin")]
    pub fn readonly_begin(&self) -> bool {
        unsafe { ecs_readonly_begin(self.raw_world) }
    }

    /// End staging.
    ///
    /// Leaves staging mode. After this operation, the world may be directly mutated again.
    /// By default, this operation also merges data back into the world, unless auto-merging
    /// was disabled explicitly.
    ///
    /// ## safety
    /// This function should only be run from the main thread.
    ///
    /// # Returns
    ///
    /// Whether the world is currently staged.
    ///
    /// # See also
    ///
    /// * C++ API: `world::readonly_end`
    #[doc(alias = "world::readonly_end")]
    pub fn readonly_end(&self) {
        unsafe {
            ecs_readonly_end(self.raw_world);
        }
    }

    /// Defers operations until the end of the frame.
    ///
    /// When this operation is invoked while iterating, the operations between `defer_begin`
    /// and `defer_end` are executed at the end of the frame.
    ///
    /// ## safety
    /// this operation is thread safe
    ///
    /// # Returns
    /// Whether the operation was successful.
    ///
    /// # See also
    ///
    /// * C++ API: `world::defer_begin`
    #[doc(alias = "world::defer_begin")]
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
    /// * C++ API: `world::defer_end`
    #[doc(alias = "world::defer_end")]
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
    /// * C++ API: `world::is_deferred`
    #[doc(alias = "world::is_deferred")]
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
    /// # Arguments
    ///
    /// * `stages`: The number of stages.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_stage_count`
    #[doc(alias = "world::set_stage_count")]
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
    /// * C++ API: `world::get_stage_count`
    #[doc(alias = "world::get_stage_count")]
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
    /// * C++ API: `world::get_stage_id`
    #[doc(alias = "world::get_stage_id")]
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
    /// * C++ API: `world::is_stage`
    #[doc(alias = "world::is_stage")]
    pub fn is_stage(&self) -> bool {
        unsafe {
            ecs_assert!(
                ecs_poly_is_(self.raw_world as *const c_void, ecs_world_t_magic as i32)
                    || ecs_poly_is_(self.raw_world as *const c_void, ecs_stage_t_magic as i32),
                FlecsErrorCode::InvalidParameter,
                "Parameter is not a world or stage"
            );
            ecs_poly_is_(self.raw_world as *const c_void, ecs_stage_t_magic as i32)
        }
    }

    /// Enable/disable auto-merging for world or stage.
    ///
    /// When auto-merging is enabled, staged data will automatically be merged
    /// with the world when staging ends. This happens at the end of `progress()`,
    /// at a sync point or when `readonly_end()` is called.
    ///
    /// Applications can exercise more control over when data from a stage is
    /// merged by disabling auto-merging. This requires an application to
    /// explicitly call `merge()` on the stage.
    ///
    /// When this function is invoked on the world, it sets all current stages to
    /// the provided value and sets the default for new stages. When this
    /// function is invoked on a stage, auto-merging is only set for that specific
    /// stage.
    ///
    /// # Arguments
    ///
    /// * `automerge` - Whether to enable or disable auto-merging.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_automerge`
    #[doc(alias = "world::set_automerge")]
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
    /// * C++ API: `world::merge`
    #[doc(alias = "world::merge")]
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
    /// * C++ API: `world::get_stage`
    #[doc(alias = "world::get_stage")]
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
    /// manually merged with the `ecs_merge` function. It is not necessary to call `defer_begin`
    /// or `defer_end` before and after enqueuing commands, as an
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
    /// * C++ API: `world::async_stage`
    #[doc(alias = "world::async_stage")]
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
    /// * C++ API: `world::get_world`
    #[doc(alias = "world::get_world")]
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
    /// * C++ API: `world::is_readonly`
    #[doc(alias = "world::is_readonly")]
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
    /// * `ctx_free` - The free function for the context. Can pass `None` if no free function is needed.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_ctx`
    #[doc(alias = "world::set_ctx")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // this doesn't actually deref the pointer
    pub fn set_context(&self, ctx: *mut c_void, ctx_free: ecs_ctx_free_t) {
        unsafe { ecs_set_ctx(self.raw_world, ctx, ctx_free) }
    }

    /// Get world context.
    ///
    /// # Returns
    ///
    /// The configured world context.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_ctx`
    #[doc(alias = "world::get_ctx")]
    pub fn get_context(&self) -> *mut c_void {
        unsafe { ecs_get_ctx(self.raw_world) }
    }

    /// Set world binding context
    /// Set a context value that can be accessed by anyone that has a reference to the world.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The world context.
    /// * `ctx_free` - The free function for the context. Can pass `None` if no free function is needed.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_binding_ctx`
    #[doc(alias = "world::set_binding_ctx")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn set_binding_context(&self, ctx: *mut c_void, ctx_free: ecs_ctx_free_t) {
        unsafe { ecs_set_ctx(self.raw_world, ctx, ctx_free) }
    }

    /// Get world binding context.
    ///
    /// # Returns
    ///
    /// The configured world context.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_binding_ctx`
    #[doc(alias = "world::get_binding_ctx")]
    pub fn get_binding_context(&self) -> *mut c_void {
        unsafe { ecs_get_ctx(self.raw_world) }
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
    /// * C++ API: `world::dim`
    #[doc(alias = "world::dim")]
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
    /// * C++ API: `world::set_entity_range`
    #[doc(alias = "world::set_entity_range")]
    pub fn set_entity_range(&self, min: impl IntoEntityId, max: impl IntoEntityId) {
        unsafe { ecs_set_entity_range(self.raw_world, min.get_id(), max.get_id()) };
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
    /// * C++ API: `world::enable_range_check`
    #[doc(alias = "world::enable_range_check")]
    pub fn enable_range_check(&self, enabled: bool) {
        unsafe { ecs_enable_range_check(self.raw_world, enabled) };
    }

    /// Get the current scope. Get the scope set by `ecs_set_scope`. If no scope is set, this operation will return 0.
    ///
    /// # Returns
    ///
    /// Returns an `Entity` representing the current scope.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_scope`
    #[doc(alias = "world::get_scope")]
    #[inline(always)]
    pub fn get_scope<T: ComponentId>(&self) -> Entity {
        Entity::new_from_existing_raw(self.raw_world, unsafe { ecs_get_scope(self.raw_world) })
    }

    /// Set the current scope. This operation sets the scope of the current stage to the provided entity.
    /// As a result new entities will be created in this scope, and lookups will be relative to the provided scope.
    /// It is considered good practice to restore the scope to the old value.
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
    /// # See also
    ///
    /// * C++ API: `world::set_scope`
    #[doc(alias = "world::set_scope")]
    #[inline(always)]
    pub fn set_scope_with_id(&self, id: impl IntoEntityId) -> Entity {
        Entity::new_id_only(unsafe { ecs_set_scope(self.raw_world, id.get_id()) })
    }

    /// Sets the current scope, but allows the scope type to be inferred from the type parameter.
    /// This operation sets the scope of the current stage to the provided entity.
    /// As a result new entities will be created in this scope, and lookups will be relative to the provided scope.
    /// It is considered good practice to restore the scope to the old value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type that implements `ComponentId`.
    ///
    /// # Returns
    ///
    /// Returns an `Entity` representing the newly set scope.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_scope`
    #[doc(alias = "world::set_scope")]
    #[inline(always)]
    pub fn set_scope_with<T: ComponentId>(&self) -> Entity {
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
    /// * C++ API: `world::set_lookup_path`
    /// * C API: `ecs_set_lookup_path`
    #[doc(alias = "world::set_lookup_path")]
    #[doc(alias = "wecs_set_lookup_path")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // this doesn't actually deref the pointer
    pub fn set_lookup_path(&self, search_path: *const EntityT) -> *mut EntityT {
        unsafe { ecs_set_lookup_path(self.raw_world, search_path) }
    }

    /// Lookup entity by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    /// * `search_path` - When false, only the current scope is searched.
    ///
    /// # Returns
    ///
    /// The entity if found, otherwise None.
    ///
    /// # See also
    ///
    /// * C++ API: `world::lookup`
    #[doc(alias = "world::lookup")]
    pub fn lookup_name(&self, name: &CStr, search_path: bool) -> Entity {
        let entity_id = unsafe {
            ecs_lookup_path_w_sep(
                self.raw_world,
                0,
                name.as_ptr(),
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
                search_path,
            )
        };

        Entity::new_from_existing_raw(self.raw_world, entity_id)
    }

    /// Lookup entity by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    /// * `search_path` - When false, only the current scope is searched.
    ///
    /// # Returns
    ///
    /// The entity if found, otherwise None.
    ///
    /// # See also
    ///
    /// * C++ API: `world::lookup`
    #[doc(alias = "world::lookup")]
    pub fn lookup_name_optional_optional(&self, name: &CStr, search_path: bool) -> Option<Entity> {
        let entity_id = unsafe {
            ecs_lookup_path_w_sep(
                self.raw_world,
                0,
                name.as_ptr(),
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
                search_path,
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
    /// * C++ API: `world::set`
    #[doc(alias = "world::set")]
    pub fn set<T: ComponentId>(&self, component: T) {
        let id = T::get_id(self.raw_world);
        set_helper(self.raw_world, id, component, id);
    }

    /// Set a singleton pair using the second element type and a first id.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The ID of the first element of the pair.
    /// * `second`: The second element of the pair to be set.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set`
    #[doc(alias = "world::set")]
    pub fn set_pair_first_id<First>(&self, second: impl IntoEntityId, first: First)
    where
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        let entity = Entity::new_from_existing_raw(self.raw_world, First::get_id(self.raw_world));
        entity.set_pair_first_id::<First>(first, second);
    }

    /// Set singleton pair.
    /// This operation sets the pair value, and uses First as type. If it does not yet exist, it will be added.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair
    /// * `Second`: The second element of the pair
    ///
    /// # Arguments
    ///
    /// * `first`: The value to set for first component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set`
    #[doc(alias = "world::set")]
    pub fn set_pair_first<First, Second>(&self, first: First)
    where
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
        Second: ComponentId + ComponentType<Struct>,
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
    /// # Arguments
    ///
    /// * `first`: The ID of the first element of the pair.
    /// * `second`: The second element of the pair to be set.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set`
    #[doc(alias = "world::set")]
    pub fn set_pair_second_id<Second>(&self, first: impl IntoEntityId, second: Second)
    where
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        let entity = Entity::new_from_existing_raw(self.raw_world, Second::get_id(self.raw_world));
        entity.set_pair_second_id::<Second>(second, first);
    }

    /// Set singleton pair.
    /// This operation sets the pair value, and uses Second as type. If it does not yet exist, it will be added.
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    /// * `value`: The value to set.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set`
    #[doc(alias = "world::set")]
    pub fn set_pair_second<First, Second>(&self, second: Second)
    where
        First: ComponentId + ComponentType<Struct> + EmptyComponent,
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
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
    pub fn modified_id(&self, id: impl IntoEntityId) {
        let id = id.get_id();
        Entity::new_from_existing_raw(self.raw_world, id).modified_id(id);
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
    pub fn modified<T>(&self)
    where
        T: ComponentId,
    {
        self.modified_id(T::get_id(self.raw_world));
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
    /// * C++ API: `world::get`
    #[doc(alias = "world::get")]
    #[inline(always)]
    pub fn get<T>(&self) -> Option<&T>
    where
        T: ComponentId + ComponentType<Struct>,
    {
        let component_id = T::get_id(self.raw_world);
        let singleton_entity = Entity::new_from_existing_raw(self.raw_world, component_id);
        unsafe {
            (ecs_get_id(self.raw_world, singleton_entity.raw_id, component_id) as *const T).as_ref()
        }
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
    /// * C++ API: `world::get_mut`
    #[doc(alias = "world::get_mut")]
    #[inline(always)]
    pub fn get_mut<T>(&mut self) -> &mut T
    where
        T: ComponentId + ComponentType<Struct>,
    {
        let component_id = T::get_id(self.raw_world);
        let singleton_entity = Entity::new_from_existing_raw(self.raw_world, component_id);

        ecs_assert!(
            std::mem::size_of::<T>() != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            std::any::type_name::<T>()
        );
        // SAFETY: The pointer is valid because ecs_get_mut_id adds the component if not present, so
        // it is guaranteed to be valid
        unsafe {
            &mut *(ecs_get_mut_id(self.raw_world, singleton_entity.raw_id, component_id) as *mut T)
        }
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
    /// * C++ API: `world::get_ref`
    #[doc(alias = "world::get_ref")]
    #[inline(always)]
    pub fn get_ref_component<T>(&self) -> Ref<T::UnderlyingType>
    where
        T: ComponentId,
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
    /// * C++ API: `world::singleton`
    #[doc(alias = "world::singleton")]
    #[inline(always)]
    pub fn get_singleton<T: ComponentId>(&self) -> Entity {
        Entity::new_from_existing_raw(self.raw_world, T::get_id(self.raw_world))
    }

    /// Gets the target for a given pair from a singleton entity.
    ///
    /// This operation returns the target for a given pair. The optional
    /// `index` can be used to iterate through targets, in case the entity has
    /// multiple instances for the same relationship.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `index` - The index (None for the first instance of the relationship).
    ///
    /// # See also
    ///
    /// * C++ API: `world::target`
    #[doc(alias = "world::target")]
    pub fn get_target_rel<First>(&self, index: Option<i32>) -> Entity
    where
        First: ComponentId,
    {
        let id = First::get_id(self.raw_world);
        Entity::new_from_existing_raw(self.raw_world, unsafe {
            ecs_get_target(self.raw_world, id, id, index.unwrap_or(0))
        })
    }

    /// Retrieves the target for a given pair from a singleton entity.
    ///
    /// This operation fetches the target associated with a specific pair. An optional
    /// `index` parameter allows iterating through multiple targets if the entity
    /// has more than one instance of the same relationship.
    ///
    /// # Arguments
    ///
    /// * `first` - The first element of the pair for which to retrieve the target.
    /// * `index` - The index (0 for the first instance of the relationship).
    ///
    /// # See also
    ///
    /// * C++ API: `world::target`
    #[doc(alias = "world::target")]
    pub fn get_target_rel_id(
        &self,
        relationship: impl IntoEntityId,
        index: Option<usize>,
    ) -> Entity {
        let relationship = relationship.get_id();
        Entity::new_from_existing_raw(self.raw_world, unsafe {
            ecs_get_target(
                self.raw_world,
                relationship,
                relationship,
                index.unwrap_or(0) as i32,
            )
        })
    }

    /// Get immutable reference for the first element of a singleton pair
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first part of the pair.
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    ///
    /// # Returns
    ///
    /// An option containing the reference to the first element of the pair if it exists, otherwise None.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get`
    #[doc(alias = "world::get")]
    #[inline(always)]
    pub fn get_pair_first_id<First>(&self, second: impl IntoEntityId) -> Option<&First>
    where
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        let component_id = First::get_id(self.raw_world);

        ecs_assert!(
            std::mem::size_of::<First>() != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            std::any::type_name::<First>()
        );

        // SAFETY: The pointer is valid because ecs_get_mut_id adds the component if not present, so
        // it is guaranteed to be valid
        unsafe {
            (ecs_get_id(self.raw_world, component_id, ecs_pair(component_id, second))
                as *const First)
                .as_ref()
        }
    }

    /// Get mutable reference for the first element of a singleton pair
    /// If the pair does not exist, it will be created.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first part of the pair.
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_mut`
    #[doc(alias = "world::get_mut")]
    #[inline(always)]
    pub fn get_pair_first_id_mut<First>(&mut self, second: impl IntoEntityId) -> &mut First
    where
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        let component_id = First::get_id(self.raw_world);

        ecs_assert!(
            std::mem::size_of::<First>() != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            std::any::type_name::<First>()
        );

        // SAFETY: The pointer is valid because ecs_get_mut_id adds the component if not present, so
        // it is guaranteed to be valid
        unsafe {
            &mut *(ecs_get_mut_id(self.raw_world, component_id, ecs_pair(component_id, second))
                as *mut First)
        }
    }

    /// Get an immutable reference for the first element of a singleton pair
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first part of the pair.
    /// * `Second`: The second part of the pair.
    ///
    /// # Returns
    ///
    /// An option containing the reference to the first element of the pair if it exists, otherwise None.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get`
    #[doc(alias = "world::get")]
    pub fn get_pair_first<First, Second>(&self) -> Option<&First>
    where
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
        Second: ComponentId + ComponentType<Struct>,
    {
        self.get_pair_first_id(Second::get_id(self.raw_world))
    }

    /// Get a mutable reference for the first element of a singleton pair
    /// If the pair does not exist, it will be created.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first part of the pair.
    /// * `Second`: The second part of the pair.
    ///
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_mut`
    #[doc(alias = "world::get_mut")]
    pub fn get_pair_first_mut<First, Second>(&mut self) -> &mut First
    where
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
        Second: ComponentId + ComponentType<Struct>,
    {
        self.get_pair_first_id_mut(Second::get_id(self.raw_world))
    }

    /// Get immutable reference for the second element of a singleton pair
    ///
    /// # Type Parameters
    ///
    /// * `second`: The second part of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    ///
    /// # Returns
    ///
    /// An option containing the reference to the second element of the pair if it exists, otherwise None.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get`
    #[doc(alias = "world::get")]
    #[inline(always)]
    pub fn get_pair_second_id<Second>(&self, first: impl IntoEntityId) -> Option<&Second>
    where
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        let component_id = Second::get_id(self.raw_world);

        ecs_assert!(
            std::mem::size_of::<Second>() != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            std::any::type_name::<Second>()
        );

        unsafe {
            (ecs_get_id(self.raw_world, component_id, ecs_pair(first, component_id))
                as *const Second)
                .as_ref()
        }
    }

    /// Get mutable reference for the second element of a singleton pair
    /// If the pair does not exist, it will be created.
    ///
    /// # Type Parameters
    ///
    /// * `second`: The second part of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_mut`
    #[doc(alias = "world::get_mut")]
    #[inline(always)]
    pub fn get_pair_second_id_mut<Second>(&mut self, first: impl IntoEntityId) -> &mut Second
    where
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        let component_id = Second::get_id(self.raw_world);

        ecs_assert!(
            std::mem::size_of::<Second>() != 0,
            FlecsErrorCode::InvalidParameter,
            "invalid type: {}",
            std::any::type_name::<Second>()
        );

        // SAFETY: The pointer is valid because ecs_get_mut_id adds the component if not present, so
        // it is guaranteed to be valid
        unsafe {
            &mut *(ecs_get_mut_id(self.raw_world, component_id, ecs_pair(first, component_id))
                as *mut Second)
        }
    }

    /// Get an immutable reference for the second element of a singleton pair.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    /// * `Second`: The second element of the pair.
    ///
    /// # Returns
    ///
    /// An option containing the reference to the second element of the pair if it exists, otherwise None.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get`
    #[doc(alias = "world::get")]
    pub fn get_pair_second<First, Second>(&self) -> Option<&Second>
    where
        First: ComponentId + ComponentType<Struct>,
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        self.get_pair_second_id(First::get_id(self.raw_world))
    }

    /// Get a mutable reference for the second element of a singleton pair.
    /// If the pair does not exist, it will be created.
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    /// * `Second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_mut`
    #[doc(alias = "world::get_mut")]
    pub fn get_pair_second_mut<First, Second>(&mut self) -> &mut Second
    where
        First: ComponentId + ComponentType<Struct>,
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        self.get_pair_second_id_mut(First::get_id(self.raw_world))
    }

    /// Check if world has the provided id.
    ///
    /// # Arguments
    ///
    /// * `id`: The id to check of a pair, entity or component.
    ///
    /// # Returns
    ///
    /// True if the world has the provided id, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `world::has`
    #[doc(alias = "world::has")]
    #[inline(always)]
    pub fn has_id(&self, id: impl IntoEntityIdExt) -> bool {
        let id = id.get_id();
        Entity::new_from_existing_raw(self.raw_world, id).has_id(id)
    }

    /// Check if world has the provided type (enum,pair,struct).
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to check.
    ///
    /// # Returns
    ///
    /// True if the world has the provided type, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `world::has`
    #[doc(alias = "world::has")]
    #[inline(always)]
    pub fn has<T>(&self) -> bool
    where
        T: IntoComponentId,
    {
        Entity::new_from_existing_raw(self.raw_world, T::get_id(self.raw_world)).has::<T>()
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
    /// * C++ API: `world::has`
    #[doc(alias = "world::has")]
    #[inline(always)]
    pub fn has_enum<T>(&self, constant: T) -> bool
    where
        T: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        let id = T::get_id(self.raw_world);
        Entity::new_from_existing_raw(self.raw_world, id).has_enum_id::<T>(id, constant)
    }

    /// Add a singleton component by id.
    /// id can be a component, entity or pair id.
    ///
    /// # Arguments
    ///
    /// * `id`: The id of the component to add.
    ///
    /// # Returns
    ///
    /// Entity handle to the singleton component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::add`
    #[doc(alias = "world::add")]
    #[inline(always)]
    pub fn add_id<T>(&self, id: T) -> Entity
    where
        T: IntoEntityIdExt,
    {
        let id = id.get_id();
        // this branch will compile out in release mode
        if T::IS_PAIR {
            let first_id = id.get_id_first();
            Entity::new_from_existing_raw(self.raw_world, first_id).add_id(id)
        } else {
            Entity::new_from_existing_raw(self.raw_world, id).add_id(id)
        }
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
    /// * C++ API: `world::add`
    #[doc(alias = "world::add")]
    #[inline(always)]
    pub fn add<T: IntoComponentId>(&self) -> Entity {
        if T::IS_PAIR {
            let first_id = <T::First as ComponentId>::get_id(self.raw_world);
            Entity::new_from_existing_raw(self.raw_world, first_id).add::<T>()
        } else {
            let id = T::get_id(self.raw_world);
            Entity::new_from_existing_raw(self.raw_world, id).add::<T>()
        }
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
    /// * C++ API: `world::add`
    #[doc(alias = "world::add")]
    #[inline(always)]
    pub fn add_enum<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &self,
        enum_value: T,
    ) -> Entity {
        Entity::new_from_existing_raw(self.raw_world, T::get_id(self.raw_world))
            .add_enum::<T>(enum_value)
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
    pub fn add_pair_second<Second: ComponentId>(&self, first: impl IntoEntityId) -> Entity {
        Entity::new_from_existing_raw(self.raw_world, Second::get_id(self.raw_world))
            .add_pair_second::<Second>(first)
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
    /// * C++ API: `world::add`
    #[doc(alias = "world::add")]
    #[inline(always)]
    pub fn add_pair_first<First: ComponentId>(&self, second: impl IntoEntityId) -> Entity {
        Entity::new_from_existing_raw(self.raw_world, First::get_id(self.raw_world))
            .add_pair_first::<First>(second)
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
    /// * C++ API: `world::add`
    #[doc(alias = "world::add")]
    #[inline(always)]
    pub fn add_enum_tag<First, Second>(&self, enum_value: Second) -> Entity
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        Entity::new_from_existing_raw(self.raw_world, First::get_id(self.raw_world))
            .add_enum_tag::<First, Second>(enum_value)
    }

    /// Remove singleton component by id.
    /// id can be a component, entity or pair id.
    ///
    /// # Arguments
    ///
    /// * `id`: The id of the component to remove.
    ///
    /// # See also
    ///
    /// * C++ API: `world::remove`
    #[doc(alias = "world::remove")]
    pub fn remove_id<T>(&self, id: T) -> Entity
    where
        T: IntoEntityIdExt,
    {
        let id = id.get_id();
        if T::IS_PAIR {
            let first_id = id.get_id_first();
            Entity::new_from_existing_raw(self.raw_world, first_id).remove_id(id)
        } else {
            Entity::new_from_existing_raw(self.raw_world, id).remove_id(id)
        }
    }

    /// Remove singleton component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component to remove.
    ///
    /// # See also
    ///
    /// * C++ API: `world::remove`
    #[doc(alias = "world::remove")]
    #[inline(always)]
    pub fn remove<T: IntoComponentId>(&self) {
        if T::IS_PAIR {
            let first_id = <T::First as ComponentId>::get_id(self.raw_world);
            Entity::new_from_existing_raw(self.raw_world, first_id).remove::<T>();
        } else {
            Entity::new_from_existing_raw(self.raw_world, T::get_id(self.raw_world)).remove::<T>();
        }
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
    /// * C++ API: `world::remove`
    #[doc(alias = "world::remove")]
    #[inline(always)]
    pub fn remove_enum_tag<First, Second>(&self, enum_value: Second)
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        Entity::new_from_existing_raw(self.raw_world, First::get_id(self.raw_world))
            .remove_enum_tag::<First, Second>(enum_value);
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
    /// * C++ API: `world::remove`
    #[doc(alias = "world::remove")]
    #[inline(always)]
    pub fn remove_pair_second<Second: ComponentId>(&self, first: impl IntoEntityId) {
        Entity::new_from_existing_raw(self.raw_world, Second::get_id(self.raw_world))
            .remove_pair_second::<Second>(first);
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
    /// * C++ API: `world::remove`
    #[doc(alias = "world::remove")]
    #[inline(always)]
    pub fn remove_pair_first<First: ComponentId>(&self, second: impl IntoEntityId) {
        Entity::new_from_existing_raw(self.raw_world, First::get_id(self.raw_world))
            .remove_pair_first::<First>(second);
    }

    /// Iterate entities in root of world
    ///
    /// # Arguments
    ///
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(Entity)`.
    ///
    /// # See also
    ///
    /// * C++ API: `world::children`
    #[doc(alias = "world::children")]
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
    /// * C++ API: `world::use`
    #[doc(alias = "world::use")]
    #[inline(always)]
    pub fn set_alias_component<T: ComponentId>(&self, alias: &CStr) -> Entity {
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
    /// * C++ API: `world::use`
    #[doc(alias = "world::use")]
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
    /// * C++ API: `world::use`
    #[doc(alias = "world::use")]
    #[inline(always)]
    pub fn set_alias_entity(&self, entity: impl IntoEntityId, alias: &CStr) {
        if alias.is_empty() {
            unsafe {
                ecs_set_alias(
                    self.raw_world,
                    entity.get_id(),
                    ecs_get_name(self.raw_world, entity.get_id()),
                );
            };
        } else {
            unsafe { ecs_set_alias(self.raw_world, entity.get_id(), alias.as_ptr()) };
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
    pub fn count_id(&self, id: impl IntoEntityIdExt) -> i32 {
        unsafe { ecs_count_id(self.raw_world, id.get_id()) }
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
    pub fn count<T: IntoComponentId>(&self) -> i32 {
        self.count_id(T::get_id(self.raw_world))
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
    pub fn count_pair_second<Second: ComponentId>(&self, first: impl IntoEntityId) -> i32 {
        self.count_id((first, Second::get_id(self.raw_world)))
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
    pub fn count_pair_first<First: ComponentId>(&self, second: impl IntoEntityId) -> i32 {
        self.count_id((First::get_id(self.raw_world), second))
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
    pub fn count_enum<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &self,
        enum_value: T,
    ) -> i32 {
        unsafe {
            ecs_count_id(
                self.raw_world,
                enum_value
                    .get_entity_id_from_enum_field(self.raw_world)
                    .raw_id,
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
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + CachedEnumData,
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
    /// * C++ API: `world::scope`
    #[doc(alias = "world::scope")]
    pub fn run_in_scope_with_id<F: FnMut()>(&self, parent_id: impl IntoEntityId, mut func: F) {
        let prev: IdT = unsafe { ecs_set_scope(self.raw_world, parent_id.get_id()) };
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
    /// * C++ API: `world::scope`
    #[doc(alias = "world::scope")]
    pub fn run_in_scope_with<T: ComponentId, F: FnMut()>(&self, func: F) {
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
    /// * C++ API: `world::scope`
    #[doc(alias = "world::scope")]
    pub fn scope_id(&self, parent_id: impl IntoEntityId) -> ScopedWorld {
        ScopedWorld::new(self, parent_id.get_id())
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
    /// * C++ API: `world::scope`
    #[doc(alias = "world::scope")]
    pub fn scope<T: ComponentId>(&self) -> ScopedWorld {
        self.scope_id(T::get_id(self.raw_world))
    }

    /// Use provided scope of name for operations ran on returned world.
    /// Operations need to be ran in a single statement
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the scope to use.
    ///
    /// # Returns
    ///
    /// A scoped world.
    ///
    /// # See also
    ///
    /// * C++ API: `world::scope`
    #[doc(alias = "world::scope")]
    pub fn scope_name(&self, name: &CStr) -> ScopedWorld {
        self.scope_id(Entity::new_named(self, name).raw_id)
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
    /// * C++ API: `world::with`
    #[doc(alias = "world::with")]
    pub fn with_id<F: FnMut()>(&self, id: impl IntoEntityIdExt, mut func: F) {
        let prev: IdT = unsafe { ecs_set_with(self.raw_world, id.get_id()) };
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
    /// * C++ API: `world::with`
    #[doc(alias = "world::with")]
    pub fn with<T: IntoComponentId, F: FnMut()>(&self, func: F) {
        self.with_id(T::get_id(self.raw_world), func);
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
    /// * C++ API: `world::with`
    #[doc(alias = "world::with")]
    pub fn with_pair_second<Second: ComponentId, F: FnMut()>(
        &self,
        first: impl IntoEntityId,
        func: F,
    ) {
        self.with_id(
            ecs_pair(first.get_id(), Second::get_id(self.raw_world)),
            func,
        );
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
    /// * C++ API: `world::with`
    #[doc(alias = "world::with")]
    pub fn with_pair_first<First: ComponentId, F: FnMut()>(
        &self,
        second: impl IntoEntityId,
        func: F,
    ) {
        self.with_id(
            ecs_pair(First::get_id(self.raw_world), second.get_id()),
            func,
        );
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
    /// * C++ API: `world::with`
    #[doc(alias = "world::with")]
    pub fn with_enum<T, F>(&self, enum_value: T, func: F)
    where
        T: ComponentId + ComponentType<Enum> + CachedEnumData,
        F: FnMut(),
    {
        self.with_id(
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
    /// * C++ API: `world::with`
    #[doc(alias = "world::with")]
    pub fn with_enum_pair<First, Second, F>(&self, enum_value: Second, func: F)
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + CachedEnumData,
        F: FnMut(),
    {
        self.with_id(
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
    /// * C++ API: `world::delete_with`
    #[doc(alias = "world::delete_with")]
    pub fn delete_with_id(&self, id: impl IntoEntityIdExt) {
        unsafe {
            ecs_delete_with(self.raw_world, id.get_id());
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
    /// * C++ API: `world::delete_with`
    #[doc(alias = "world::delete_with")]
    pub fn delete_entities_with<T: IntoComponentId>(&self) {
        self.delete_with_id(T::get_id(self.raw_world));
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
    /// * C++ API: `world::delete_with`
    #[doc(alias = "world::delete_with")]
    pub fn delete_with_pair_second<Second: ComponentId>(&self, first: impl IntoEntityId) {
        self.delete_with_id(ecs_pair(first.get_id(), Second::get_id(self.raw_world)));
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
    /// * C++ API: `world::delete_with`
    #[doc(alias = "world::delete_with")]
    pub fn delete_entities_with_pair_second_id<First: ComponentId>(
        &self,
        second: impl IntoEntityId,
    ) {
        self.delete_with_id(ecs_pair(First::get_id(self.raw_world), second.get_id()));
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
    /// * C++ API: `world::delete_with`
    #[doc(alias = "world::delete_with")]
    pub fn delete_with_enum<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &self,
        enum_value: T,
    ) {
        self.delete_with_id(enum_value.get_entity_id_from_enum_field(self.raw_world));
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
    pub fn delete_with_enum_pair<First, Second>(&self, enum_value: Second)
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        self.delete_with_id(ecs_pair(
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
    /// * C++ API: `world::remove_all`
    #[doc(alias = "world::remove_all")]
    pub fn remove_all_id(&self, id: impl IntoEntityIdExt) {
        unsafe {
            ecs_remove_all(self.raw_world, id.get_id());
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
    /// * C++ API: `world::remove_all`
    #[doc(alias = "world::remove_all")]
    pub fn remove_all<T: IntoComponentId>(&self) {
        self.remove_all_id(T::get_id(self.raw_world));
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
    /// * C++ API: `world::remove_all`
    #[doc(alias = "world::remove_all")]
    pub fn remove_all_pair_second<Second: ComponentId>(&self, first: impl IntoEntityId) {
        self.remove_all_id((first, Second::get_id(self.raw_world)));
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
    /// * C++ API: `world::remove_all`
    #[doc(alias = "world::remove_all")]
    pub fn remove_all_pair_first<First: ComponentId>(&self, second: impl IntoEntityId) {
        self.remove_all_id((First::get_id(self.raw_world), second));
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
    /// * C++ API: `world::remove_all`
    #[doc(alias = "world::remove_all")]
    pub fn remove_all_enum<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
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
    /// * C++ API: `world::remove_all`
    #[doc(alias = "world::remove_all")]
    pub fn remove_all_enum_pair<First, Second>(&self, enum_value: Second)
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        self.remove_all_id((
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
    /// * C++ API: `world::defer`
    #[doc(alias = "world::defer")]
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
    /// * C++ API: `world::defer_suspend`
    #[doc(alias = "world::defer_suspend")]
    pub fn defer_suspend(&self) {
        unsafe {
            ecs_defer_suspend(self.raw_world);
        }
    }

    /// Resumes deferring of operations.
    ///
    /// # See also
    ///
    /// * C++ API: `world::defer_resume`
    #[doc(alias = "world::defer_resume")]
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
    /// * C++ API: `world::exists`
    #[doc(alias = "world::exists")]
    pub fn exists(&self, entity: impl IntoEntityId) -> bool {
        unsafe { ecs_exists(self.raw_world, entity.get_id()) }
    }

    /// Checks if the given entity ID is alive in the world.
    ///
    /// # See also
    ///
    /// * C++ API: `world::is_alive`
    #[doc(alias = "world::is_alive")]
    pub fn is_alive(&self, entity: impl IntoEntityId) -> bool {
        unsafe { ecs_is_alive(self.raw_world, entity.get_id()) }
    }

    /// Checks if the given entity ID is valid.
    /// Invalid entities cannot be used with API functions.
    ///
    /// # See also
    ///
    /// * C++ API: `world::is_valid`
    #[doc(alias = "world::is_valid")]
    pub fn is_valid(&self, entity: impl IntoEntityId) -> bool {
        unsafe { ecs_is_valid(self.raw_world, entity.get_id()) }
    }

    /// Get alive entity for id.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check
    ///
    /// # Returns
    ///
    /// She entity with the current generation.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_alive`
    #[doc(alias = "world::get_alive")]
    pub fn get_alive(&self, entity: impl IntoEntityId) -> Entity {
        let entity = unsafe { ecs_get_alive(self.raw_world, entity.get_id()) };
        Entity::new_from_existing_raw(self.raw_world, entity)
    }

    /// Ensures that entity with provided generation is alive.
    /// This operation will fail if an entity exists with the same id and a
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
    /// * C++ API: `world::ensure`
    #[doc(alias = "world::ensure")]
    pub fn ensure(&self, entity: impl IntoEntityId) -> Entity {
        let entity = entity.get_id();
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
    /// * C++ API: `world::run_post_frame`
    #[doc(alias = "world::run_post_frame")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // this doesn't actually deref the pointer
    pub fn run_post_frame(&self, action: ecs_fini_action_t, ctx: *mut c_void) {
        unsafe {
            ecs_run_post_frame(self.raw_world, action, ctx);
        }
    }
}

/// Entity mixin implementation
impl World {
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
    /// * C++ API: `world::entity`
    #[doc(alias = "world::entity")]
    #[doc(alias = "world::id")] //enum mixin implementation
    pub fn get_id_from_enum<T>(&self, enum_value: T) -> Entity
    where
        T: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        Entity::new_from_existing_raw(
            self.raw_world,
            enum_value.get_entity_id_from_enum_field(self.raw_world),
        )
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
    /// * C++ API: `world::entity`
    #[doc(alias = "world::entity")]
    pub fn new_entity_named_type<T: ComponentId>(&self, name: &CStr) -> Entity {
        Entity::new_from_existing_raw(
            self.raw_world,
            T::register_explicit_named(self.raw_world, name),
        )
    }

    pub fn new_entity_type<T: ComponentId>(&self) -> Entity {
        Entity::new_from_existing_raw(self.raw_world, T::get_id(self.raw_world))
    }

    /// Create an entity that's associated with a name
    ///
    /// # Arguments
    ///
    /// * `name` - The name to use for the new entity.
    ///
    /// # See also
    ///
    /// * C++ API: `world::entity`
    #[doc(alias = "world::entity")]
    pub fn new_entity_named(&self, name: &CStr) -> Entity {
        Entity::new_named(self, name)
    }

    /// Create a new entity.
    ///
    /// # See also
    ///
    /// * C++ API: `world::entity`
    #[doc(alias = "world::entity")]
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
    /// * C++ API: `world::entity`
    #[doc(alias = "world::entity")]
    pub fn new_entity_from_id(&self, id: impl IntoEntityId) -> Entity {
        Entity::new_from_existing_raw(self.raw_world, id.get_id())
    }

    /// Creates a prefab
    ///
    /// # Returns
    ///
    /// The prefab entity.
    ///
    /// # See also
    ///
    /// * C++ API: `world::prefab`
    #[doc(alias = "world::prefab")]
    pub fn prefab(&self) -> Entity {
        let result = Entity::new(self);
        result.add_id(ECS_PREFAB);
        result
    }

    /// Creates a named prefab
    ///
    /// # Arguments
    ///
    /// * `name` - The name to use for the new prefab.
    ///
    /// # Returns
    ///
    /// The prefab entity.
    ///
    /// # See also
    ///
    /// * C++ API: `world::prefab`
    #[doc(alias = "world::prefab")]
    pub fn prefab_named(&self, name: &CStr) -> Entity {
        let result = Entity::new_named(self, name);
        result.add_id(ECS_PREFAB);
        result
    }

    /// Creates a prefab that's associated with a type
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to associate with the new prefab.
    ///
    /// # Returns
    ///
    /// The prefab entity.
    ///
    /// # See also
    ///
    /// * C++ API: `world::prefab`
    #[doc(alias = "world::prefab")]
    pub fn prefab_type<T: ComponentId>(&self) -> Entity {
        let result = Component::<T>::new(self).to_entity();
        result.add_id(ECS_PREFAB);
        result.add::<T>();
        result
    }

    /// Creates a named prefab that's associated with a type
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to associate with the new prefab.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to use for the new prefab.
    ///
    /// # Returns
    ///
    /// The prefab entity.
    ///
    /// # See also
    ///
    /// * C++ API: `world::prefab`
    #[doc(alias = "world::prefab")]
    pub fn prefab_type_named<T: ComponentId>(&self, name: &CStr) -> Entity {
        let result = Component::<T>::new_named(self, name).to_entity();
        result.add_id(ECS_PREFAB);
        result.add::<T>();
        result
    }
}
/// Id mixin implementation
impl World {
    /// Get  id of component / pair.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    ///
    /// # Returns
    ///
    /// The id of the component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::id`
    /// * C++ API: `world::pair`
    #[doc(alias = "world::id")]
    #[doc(alias = "world::pair")]
    pub fn get_id<T: IntoComponentId>(&self) -> Id {
        Id::new(Some(self), T::get_id(self.raw_world))
    }

    /// get pair id from relationship, object.
    ///
    /// # Arguments
    ///
    /// * `first` - The id of the first element of the pair.
    /// * `second` - The id of the second element of the pair.
    ///
    /// # Returns
    ///
    /// The pair as Id
    ///
    /// # See also
    ///
    /// * C++ API: `world::pair`
    #[doc(alias = "world::pair")]
    pub fn get_id_pair(&self, first: impl IntoEntityId, second: impl IntoEntityId) -> Id {
        ecs_assert!(
            !ecs_is_pair(first.get_id()) && !ecs_is_pair(second.get_id()),
            FlecsErrorCode::InvalidParameter,
            "cannot create nested pairs"
        );
        Id::new(Some(self), (first, second))
    }

    /// get pair id from relationship, object.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second` - The id of the second element of the pair.
    ///
    /// # Returns
    ///
    /// The pair as Id
    ///
    /// # See also
    ///
    /// * C++ API: `world::pair`
    #[doc(alias = "world::pair")]
    pub fn get_id_pair_first<First: ComponentId>(&self, second: impl IntoEntityId) -> Id {
        ecs_assert!(
            !ecs_is_pair(second.get_id()),
            FlecsErrorCode::InvalidParameter,
            "cannot create nested pairs"
        );
        Id::new(Some(self), (First::get_id(self.raw_world), second))
    }
}

/// Component mixin implementation
impl World {
    /// Find or register component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    ///
    /// # Returns
    ///
    /// The found or registered component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::component`
    #[doc(alias = "world::component")]
    pub fn component<T: ComponentId>(&self) -> Component<T::UnderlyingType> {
        Component::<T::UnderlyingType>::new(self)
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
    ///
    /// # Returns
    ///
    /// The found or registered component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::component`
    #[doc(alias = "world::component")]
    pub fn component_named<T: ComponentId>(&self, name: &CStr) -> Component<T::UnderlyingType> {
        Component::<T::UnderlyingType>::new_named(self, name)
    }

    /// Find or register untyped component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    ///
    /// # Returns
    ///
    /// The found or registered untyped component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::component`
    #[doc(alias = "world::component")]
    pub fn component_untyped<T: ComponentId>(&self) -> UntypedComponent {
        UntypedComponent::new(self, T::get_id(self.raw_world))
    }

    /// Find or register untyped component.
    ///
    /// # Arguments
    ///
    /// * `id` - The component id.
    ///
    /// # Returns
    ///
    /// The found or registered untyped component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::component`
    #[doc(alias = "world::component")]
    pub fn component_untyped_id(&self, id: impl IntoEntityId) -> UntypedComponent {
        UntypedComponent::new(self, id)
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
    /// # See also
    ///
    /// * C++ API: `world::to_entity`
    #[doc(alias = "world::to_entity")]
    pub fn to_entity<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &self,
        enum_value: T,
    ) -> Entity {
        Entity::new_from_existing_raw(
            self.raw_world,
            enum_value.get_entity_id_from_enum_field(self.raw_world),
        )
    }
}

/// Term mixin implementation
impl World {
    /// Creates a term for a (component) type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    ///
    /// # Returns
    ///
    /// The term for the component type.
    ///
    /// # See also
    ///
    /// * C++ API: `world::term`
    #[doc(alias = "world::term")]
    pub fn term<T: IntoComponentId>(&self) -> Term {
        Term::new_type::<T>(Some(self))
    }
}

// Event mixin implementation
impl World {
    /// Create a new event builder (untyped) from entity id which represents an event
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
    /// * C++ API: `world::event`
    #[doc(alias = "world::event")]
    pub fn event_id(&self, event: impl IntoEntityId) -> EventBuilder {
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
    /// * C++ API: `world::event`
    #[doc(alias = "world::event")]
    pub fn event<T: EventData + ComponentId>(&self) -> EventBuilderTyped<T> {
        EventBuilderTyped::<T>::new(self, T::get_id(self.raw_world))
    }
}

// Observer mixin implementation
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
    ///
    /// # See also
    ///
    /// * C++ API: `world::observer`
    #[doc(alias = "world::observer")]
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
    ///
    /// # See also
    ///
    /// * C++ API: `world::observer`
    #[doc(alias = "world::observer")]
    pub fn observer_builder<'a, Components>(&self) -> ObserverBuilder<'a, Components>
    where
        Components: Iterable<'a>,
    {
        ObserverBuilder::<'a, Components>::new(self)
    }

    /// Create a new named observer.
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the observer.
    ///
    /// # Returns
    ///
    /// Observer builder.
    ///
    /// # See also
    ///
    /// * C++ API: `world::observer`
    #[doc(alias = "world::observer")]
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

// Filter mixin implementation
impl World {
    /// Create a new filter.
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Returns
    ///
    /// A new filter.
    ///
    /// # See also
    ///
    /// * C++ API: `world::filter`
    #[doc(alias = "world::filter")]
    pub fn filter<'a, Components>(&self) -> Filter<'a, Components>
    where
        Components: Iterable<'a>,
    {
        Filter::<'a, Components>::new(self)
    }

    /// Create a new named filter.
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the observer.
    ///
    /// # Returns
    ///
    /// A new filter.
    ///
    /// # See also
    ///
    /// * C++ API: `world::filter`
    #[doc(alias = "world::filter")]
    pub fn filter_named<'a, Components>(&self, name: &CStr) -> Filter<'a, Components>
    where
        Components: Iterable<'a>,
    {
        FilterBuilder::<'a, Components>::new_named(self, name).build()
    }

    /// Create a `filter_builder`
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Returns
    ///
    /// Filter builder.
    ///
    /// # See also
    ///
    /// * C++ API: `world::filter_builder`
    #[doc(alias = "world::filter_builder")]
    pub fn filter_builder<'a, Components>(&self) -> FilterBuilder<'a, Components>
    where
        Components: Iterable<'a>,
    {
        FilterBuilder::<'a, Components>::new(self)
    }

    /// Create a new named `filter_builder`.
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the observer.
    ///
    /// # Returns
    ///
    /// Filter builder.
    ///
    /// # See also
    ///
    /// * C++ API: `world::filter_builder`
    #[doc(alias = "world::filter_builder")]
    pub fn filter_builder_named<'a, Components>(&self, name: &CStr) -> FilterBuilder<'a, Components>
    where
        Components: Iterable<'a>,
    {
        FilterBuilder::<'a, Components>::new_named(self, name)
    }

    pub fn each<'a, Components>(
        &self,
        func: impl FnMut(Components::TupleType),
    ) -> Filter<'a, Components>
    where
        Components: Iterable<'a>,
    {
        let filter = Filter::<'a, Components>::new(self);
        filter.each(func);
        filter
    }

    pub fn each_entity<'a, Components>(
        &self,
        func: impl FnMut(&mut Entity, Components::TupleType),
    ) -> Filter<'a, Components>
    where
        Components: Iterable<'a>,
    {
        let filter = Filter::<'a, Components>::new(self);
        filter.each_entity(func);
        filter
    }
}

/// Query mixin implementation
impl World {
    /// Create a new query.
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Returns
    ///
    /// A new query.
    ///
    /// # See also
    ///
    /// * C++ API: `world::query`
    #[doc(alias = "world::query")]
    pub fn query<'a, Components>(&self) -> Query<'a, Components>
    where
        Components: Iterable<'a>,
    {
        Query::<'a, Components>::new(self)
    }

    /// Create a new named query.
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the query.
    ///
    /// # Returns
    ///
    /// A new query.
    ///
    /// # See also
    ///
    /// * C++ API: `world::query`
    #[doc(alias = "world::query")]
    pub fn query_named<'a, Components>(&self, name: &CStr) -> Query<'a, Components>
    where
        Components: Iterable<'a>,
    {
        QueryBuilder::<'a, Components>::new_named(self, name).build()
    }

    /// Create a new query builder.
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Returns
    ///
    /// A new query builder.
    ///
    /// # See also
    ///
    /// * C++ API: `world::query_builder`
    #[doc(alias = "world::query_builder")]
    pub fn query_builder<'a, Components>(&self) -> QueryBuilder<'a, Components>
    where
        Components: Iterable<'a>,
    {
        QueryBuilder::<'a, Components>::new(self)
    }

    /// Create a new named query builder.
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the query.
    ///
    /// # Returns
    ///
    /// A new query builder.
    ///
    /// # See also
    ///
    /// * C++ API: `world::query_builder`
    #[doc(alias = "world::query_builder")]
    pub fn query_builder_named<'a, Components>(&self, name: &CStr) -> QueryBuilder<'a, Components>
    where
        Components: Iterable<'a>,
    {
        QueryBuilder::<'a, Components>::new_named(self, name)
    }
}

/// Systems mixin implementation
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
    pub fn system_from_entity(&self, entity: Entity) -> System {
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

/// Pipeline mixin implementation
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

    /// Create a new named pipeline.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the pipeline.
    ///
    /// # See also
    ///
    /// * C++ API: `world::pipeline`
    #[doc(alias = "world::pipeline")]
    #[inline(always)]
    pub fn pipeline_named(&self, name: &CStr) -> PipelineBuilder<()> {
        PipelineBuilder::<()>::new_named(self, name)
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
        Pipeline: ComponentType<Struct> + ComponentId,
    {
        PipelineBuilder::<()>::new_entity(self, Pipeline::get_id(self.raw_world))
    }

    /// Set a custom pipeline. This operation sets the pipeline to run when `ecs_progress` is invoked.
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
    pub fn set_pipeline(&self, pipeline: impl IntoEntityId) {
        unsafe {
            sys::ecs_set_pipeline(self.raw_world, pipeline.get_id());
        }
    }

    /// Set a custom pipeline by type. This operation sets the pipeline to run when `ecs_progress` is invoked.
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
        Pipeline: ComponentType<Struct> + ComponentId,
    {
        unsafe {
            sys::ecs_set_pipeline(self.raw_world, Pipeline::get_id(self.raw_world));
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
            sys::ecs_get_pipeline(self.raw_world)
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
        unsafe { sys::ecs_progress(self.raw_world, delta_time) }
    }

    /// Run pipeline.
    /// Runs all systems in the specified pipeline. Can be invoked from multiple
    /// threads if staging is disabled, managing staging and, if needed, thread
    /// synchronization.
    ///
    /// Providing 0 for pipeline id runs the default pipeline (builtin or set via
    /// `set_pipeline()`). Using `progress()` auto-invokes this for the default pipeline.
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
    pub fn run_pipeline_id(&self, pipeline: impl IntoEntityId) {
        Self::run_pipeline_id_time(self, pipeline, 0.0);
    }

    /// Run pipeline.
    /// Runs all systems in the specified pipeline. Can be invoked from multiple
    /// threads if staging is disabled, managing staging and, if needed, thread
    /// synchronization.
    ///
    /// Providing 0 for pipeline id runs the default pipeline (builtin or set via
    /// `set_pipeline()`). Using `progress()` auto-invokes this for the default pipeline.
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
    pub fn run_pipeline_id_time(&self, pipeline: impl IntoEntityId, delta_time: super::FTime) {
        unsafe {
            sys::ecs_run_pipeline(self.raw_world, pipeline.get_id(), delta_time);
        }
    }

    /// Run pipeline.
    /// Runs all systems in the specified pipeline. Can be invoked from multiple
    /// threads if staging is disabled, managing staging and, if needed, thread
    /// synchronization.
    ///
    /// Providing 0 for pipeline id runs the default pipeline (builtin or set via
    /// `set_pipeline()`). Using `progress()` auto-invokes this for the default pipeline.
    /// Additional pipelines may be run explicitly.
    ///
    /// # Note
    ///
    /// Only supports single-threaded applications with a single stage when called from an application.
    ///
    /// # Type Parameters
    ///
    /// * `Component` - The associated type to use for the pipeline.
    ///
    /// # Arguments
    ///
    /// * `delta_time` - Time to advance the world.
    ///
    /// # See also
    ///
    /// * C++ API: `world::run_pipeline`
    #[doc(alias = "world::run_pipeline")]
    pub fn run_pipeline_time<Component>(&self, delta_time: super::FTime)
    where
        Component: ComponentType<Struct> + ComponentId,
    {
        unsafe {
            sys::ecs_run_pipeline(
                self.raw_world,
                Component::get_id(self.raw_world),
                delta_time,
            );
        }
    }

    /// Run pipeline.
    /// Runs all systems in the specified pipeline. Can be invoked from multiple
    /// threads if staging is disabled, managing staging and, if needed, thread
    /// synchronization.
    ///
    /// Providing 0 for pipeline id runs the default pipeline (builtin or set via
    /// `set_pipeline()`). Using `progress()` auto-invokes this for the default pipeline.
    /// Additional pipelines may be run explicitly.
    ///
    /// # Note
    ///
    /// Only supports single-threaded applications with a single stage when called from an application.
    ///
    /// # Type Parameters
    ///
    /// * `Component` - The associated type to use for the pipeline.
    ///
    /// # See also
    ///
    /// * C++ API: `world::run_pipeline`
    #[doc(alias = "world::run_pipeline")]
    pub fn run_pipeline<Component>(&self)
    where
        Component: ComponentType<Struct> + ComponentId,
    {
        Self::run_pipeline_time::<Component>(self, 0.0);
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
            sys::ecs_set_time_scale(self.raw_world, mul);
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
        self.get_world_info().time_scale
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
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_target_fps`
    #[doc(alias = "world::get_target_fps")]
    #[inline(always)]
    pub fn get_target_fps(&self) -> super::FTime {
        self.get_world_info().target_fps
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
            sys::ecs_set_target_fps(self.raw_world, target_fps);
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
            sys::ecs_reset_clock(self.raw_world);
        }
    }

    /// Set number of worker threads.
    ///
    /// Setting this value to a value higher than 1 will start as many threads and
    /// will cause systems to evenly distribute matched entities across threads.
    /// The operation may be called multiple times to reconfigure the number of threads used,
    /// but never while running a system / pipeline. Calling `ecs_set_threads` will also end the use
    /// of task threads setup with `ecs_set_task_threads` and vice-versa
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
            sys::ecs_set_threads(self.raw_world, threads);
        }
    }

    /// Get number of configured stages. Return number of stages set by `ecs_set_stage_count`.
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
        unsafe { sys::ecs_get_stage_count(self.raw_world) }
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
            sys::ecs_set_task_threads(self.raw_world, task_threads);
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
        unsafe { sys::ecs_using_task_threads(self.raw_world) }
    }
}

/// App mixin implementation
#[cfg(feature = "flecs_app")]
impl World {
    /// Create a new app.
    /// The app builder is a convenience wrapper around a loop that runs
    /// `world::progress`. An app allows for writing platform agnostic code,
    /// as it provides hooks to modules for overtaking the main loop which is
    /// required for frameworks like emscripten.
    ///
    /// # See also
    ///
    /// * C++ API: `world::app`
    #[doc(alias = "world::app")]
    #[inline(always)]
    pub fn app(&mut self) -> App {
        self.is_owned = false;
        App::new(self)
    }
}

/// Rules mixin implementation
#[cfg(feature = "flecs_rules")]
impl World {
    /// Create a new rule.
    ///
    /// # Returns
    ///
    /// A new rule.
    ///
    /// # See also
    ///
    /// * C++ API: `world::rule`
    #[doc(alias = "world::rule")]
    #[inline(always)]
    pub fn rule<'a, T>(&self) -> crate::addons::rules::Rule<'a, T>
    where
        T: Iterable<'a>,
    {
        crate::addons::rules::RuleBuilder::<'a, T>::new(self).build()
    }

    /// Create a new named rule.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the rule.
    ///
    /// # Returns
    ///
    /// A new rule.
    ///
    /// # See also
    ///
    /// * C++ API: `world::rule`
    #[doc(alias = "world::rule")]
    #[inline(always)]
    pub fn rule_named<'a, T>(&self, name: &CStr) -> crate::addons::rules::Rule<'a, T>
    where
        T: Iterable<'a>,
    {
        crate::addons::rules::RuleBuilder::<'a, T>::new_named(self, name).build()
    }

    /// Create a new rule builder.
    ///
    /// # Returns
    ///
    /// A new rule builder.
    ///
    /// # See also
    ///
    /// * C++ API: `world::rule_builder`
    #[doc(alias = "world::rule_builder")]
    #[inline(always)]
    pub fn rule_builder<'a, T>(&self) -> crate::addons::rules::RuleBuilder<'a, T>
    where
        T: Iterable<'a>,
    {
        crate::addons::rules::RuleBuilder::<'a, T>::new(self)
    }

    /// Create a new named rule builder.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the rule.
    ///
    /// # Returns
    ///
    /// A new rule builder.
    ///
    /// # See also
    ///
    /// * C++ API: `world::rule_builder`
    #[doc(alias = "world::rule_builder")]
    #[inline(always)]
    pub fn rule_builder_named<'a, T>(&self, name: &CStr) -> crate::addons::rules::RuleBuilder<'a, T>
    where
        T: Iterable<'a>,
    {
        crate::addons::rules::RuleBuilder::<'a, T>::new_named(self, name)
    }
}
