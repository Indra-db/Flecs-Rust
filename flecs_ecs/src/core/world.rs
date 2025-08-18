//! World operations.

use core::ffi::CStr;
use core::{ffi::c_void, ptr::NonNull};

#[cfg(feature = "flecs_system")]
use crate::addons::system::{System, SystemBuilder};

#[cfg(feature = "flecs_pipeline")]
use crate::addons::pipeline::PipelineBuilder;

use crate::core::*;
use crate::sys;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

pub(crate) type FlecsArray = Vec<u64>;

/// The `World` is the container for all ECS data. It stores the entities and
/// their components, does queries and runs systems.
///
/// Typically there is only a single world, but there is no limit on the number
/// of worlds an application can create.
///
/// If the world is deleted, all data in the world will be deleted as well.
///
/// # Examples
///
/// ```
/// # use flecs_ecs::core::World;
/// let world = World::new();
/// ```
///
/// # See also
///
/// * [`addons::app`](crate::addons::app)
#[derive(Debug, Eq, PartialEq)]
pub struct World {
    pub(crate) raw_world: NonNull<sys::ecs_world_t>,
    pub(crate) components: NonNull<FlecsIdMap>,
    pub(crate) components_array: NonNull<FlecsArray>,
}

impl Clone for World {
    fn clone(&self) -> Self {
        unsafe { sys::flecs_poly_claim_(self.raw_world.as_ptr() as *mut c_void) };
        Self {
            raw_world: self.raw_world,
            components: self.components,
            components_array: self.components_array,
        }
    }
}

unsafe impl Send for World {}

impl Default for World {
    fn default() -> Self {
        ecs_os_api::ensure_initialized();

        let raw_world = NonNull::new(unsafe { sys::ecs_init() }).unwrap();
        let ctx = Box::leak(Box::new(WorldCtx::new()));
        let components = unsafe { NonNull::new_unchecked(&mut ctx.components) };
        let components_array = unsafe { NonNull::new_unchecked(&mut ctx.components_array) };
        let world = Self {
            raw_world,
            components,
            components_array,
        };
        unsafe {
            sys::ecs_set_binding_ctx(
                world.raw_world.as_ptr(),
                ctx as *mut WorldCtx as *mut c_void,
                None, //we manually destroy it in world drop for ref count check
            );
        }

        world.init_builtin_components();
        world
    }
}

impl Drop for World {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }

        let world_ptr = self.raw_world.as_ptr();
        if unsafe { sys::flecs_poly_release_(world_ptr as *mut c_void) } == 0 {
            if unsafe { sys::ecs_stage_get_id(world_ptr) } == -1 {
                unsafe { sys::ecs_stage_free(world_ptr) };
            } else {
                let ctx = self.world_ctx_mut();

                unsafe {
                    // before we call ecs_fini(), we increment the reference count back to 1
                    // otherwise, copies of this object created during ecs_fini (e.g. a component on_remove hook)
                    // would call again this destructor and ecs_fini().
                    sys::flecs_poly_claim_(world_ptr as *mut c_void);
                    sys::ecs_fini(self.raw_world.as_ptr())
                };
                let is_ref_count_not_zero = !ctx.is_ref_count_zero();
                if is_ref_count_not_zero && !ctx.is_panicking() {
                    ctx.set_is_panicking_true();
                    panic!("The code base still has lingering references to `Query` objects. This is a bug in the user code.
                        Please ensure that all `Query` objects are out of scope before the world is destroyed.");
                }

                let ctx = unsafe { Box::from_raw(ctx as *mut WorldCtx) };
                drop(ctx);
            }
        }
    }
}

impl World {
    /// Creates a new world, same as `default()`
    pub fn new() -> Self {
        Self::default()
    }

    fn init_builtin_components(&self) {
        // used for event handling with no data
        self.component_named::<()>("flecs::rust::() - None");

        #[cfg(feature = "flecs_meta")]
        {
            self.component_named::<crate::prelude::meta::EcsTypeKind>("flecs::meta::type_kind");
            self.component_named::<crate::prelude::meta::EcsPrimitiveKind>(
                "flecs::meta::primitive_kind",
            );

            //self.component_named::<crate::prelude::meta::EcsMember>("flecs::meta::member_t");
            //self.component_named::<crate::prelude::meta::EcsEnumConstant>(
            //    "flecs::meta::enum_constant",
            //);
            //self.component_named::<crate::prelude::meta::EcsBitmaskConstant>(
            //    "flecs::meta::bitmask_constant",
            //);

            unsafe {
                self.entity_named("::flecs::rust")
                    .add_id_unchecked(flecs::Module::ID)
            };
            crate::addons::meta::meta_init_builtin(self);
            // entity.scope(|world| {
            //     let comp = world.component::<Entity>();
            //     comp.opaque_func(crate::prelude::meta::flecs_entity_support);
            // });
        }
    }

    /// deletes and recreates the world
    ///
    /// # Panics
    /// This function panics if lingering references to `Query` and `World` objects are still present.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let e = world.entity().id();
    ///
    /// assert!(world.exists(e));
    /// let new_world = world.reset();
    /// assert!(!new_world.exists(e));
    /// ```
    pub fn reset(self) -> Self {
        if unsafe { sys::flecs_poly_refcount(self.raw_world.as_ptr() as *mut c_void) } > 1 {
            panic!("Reset would invalidate other world handles that are still lingering in the user's code base. 
            This is a bug in the user code. Please ensure that all world handles are out of scope before calling `reset`.");
        }
        drop(self);
        World::new()
    }

    /// obtain pointer to C world object.
    ///
    /// # Returns
    ///
    /// Returns a pointer to the C world object.
    #[inline(always)]
    #[doc(hidden)]
    pub fn ptr_mut(&self) -> *mut sys::ecs_world_t {
        self.raw_world.as_ptr()
    }

    /// Get the world's info. See [`sys::WorldInfo`] for what information you can retrieve.
    ///
    /// # Example
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// world.progress();
    ///
    /// let world_info = world.info();
    ///
    /// assert!(world_info.delta_time > 0.0);
    /// //assert!(world_info.world_time_total_raw > 0.0); //BUG TODO
    /// //assert!(world_info.systems_ran_frame == 0);
    /// ```
    pub fn info(&self) -> sys::WorldInfo {
        // SAFETY: The pointer is valid for the lifetime of the world.
        unsafe { *sys::ecs_get_world_info(self.raw_world.as_ptr()) }
    }

    /// Signals the application to quit.
    ///
    /// After calling this function, the next call to [`World::progress()`] returns false.
    ///
    /// # Example
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let mut count = 0;
    /// while world.progress() {
    ///     count += 1;
    ///     if count == 5 {
    ///         world.quit();
    ///     }
    /// }
    /// assert!(count == 5);
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::should_quit()`]
    pub fn quit(&self) {
        unsafe {
            sys::ecs_quit(self.raw_world.as_ptr());
        }
    }

    /// Tests if [`World::quit()`] has been called.
    ///
    /// # Returns
    ///
    /// True if quit has been called, false otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// assert!(!world.should_quit());
    /// world.quit();
    /// assert!(world.should_quit());
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::quit()`]
    pub fn should_quit(&self) -> bool {
        unsafe { sys::ecs_should_quit(self.raw_world.as_ptr()) }
    }

    /// Registers an action to be executed when the world is destroyed.
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // this doesn't actually deref the pointer
    pub fn on_destroyed(&self, action: sys::ecs_fini_action_t, ctx: *mut c_void) {
        unsafe {
            sys::ecs_atfini(self.raw_world.as_ptr(), action, ctx);
        }
    }

    /// Begins a frame.
    ///
    /// When an application does not use [`World::progress()`] to control the main loop, it
    /// can still use Flecs features such as FPS limiting and time measurements processed.
    ///
    /// Calls to [`World::frame_begin`] must always be followed by [`World::frame_end`].
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
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Position {
    ///     x: f32,
    ///     y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let entity = world.entity().set(Position { x: 5.0, y: 0.0 });
    ///
    /// let sys = world.system::<&Position>().each(|pos| {});
    ///
    /// let world_info = world.info();
    ///
    /// assert_eq!(world_info.systems_ran_frame, 0);
    ///
    /// let dt = world.frame_begin(0.0);
    ///
    /// world.frame_end();
    ///
    /// let world_info = world.info();
    ///
    /// //TODO
    /// //assert_eq!(world_info.systems_ran_frame, 1);
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::frame_end()`]
    pub fn frame_begin(&self, delta_time: f32) -> f32 {
        unsafe { sys::ecs_frame_begin(self.raw_world.as_ptr(), delta_time) }
    }

    /// Ends a frame.
    ///
    /// This operation must be called at the end of the frame, and always after
    /// [`World::frame_begin()`].
    ///
    /// # Safety
    /// The function should only be run from the main thread.
    ///
    /// # See also
    ///
    /// * [`World::frame_begin()`]
    pub fn frame_end(&self) {
        unsafe {
            sys::ecs_frame_end(self.raw_world.as_ptr());
        }
    }

    /// Begin readonly mode.
    ///
    /// When an application does not use [`World::progress()`] to control the main loop,
    /// it can still use Flecs features such as the defer queue. To stage changes, this function
    /// must be called after [`World::frame_begin()`].
    ///
    /// A call to [`World::readonly_begin()`] must be followed by a call to
    /// [`World::readonly_end()`].
    ///
    /// When staging is enabled, modifications to entities are stored to a stage.
    /// This ensures that arrays are not modified while iterating. Modifications are
    /// merged back to the "main stage" when [`World::readonly_end()`] is invoked.
    ///
    /// While the world is in staging mode, no structural changes (add/remove/...) can
    /// be made to the world itself. Operations must be executed on a stage instead
    /// (see [`World::stage()`]).
    ///
    /// Readonly mode is a stronger version of deferred mode. In deferred mode,
    /// ECS operations such as add/remove/set/delete etc. are added to a command
    /// queue to be executed later. In readonly mode, operations that could break
    /// scheduler logic (such as creating systems, queries) are also disallowed.
    ///
    /// Readonly mode itself has a single-threaded and a multi-threaded mode. In
    /// single-threaded mode certain mutations on the world are still allowed, for example:
    /// - Entity liveliness operations (such as new, `make_alive`), so that systems are
    ///   able to create new entities.
    /// - Implicit component registration, so that this works from systems.
    /// - Mutations to supporting data structures for the evaluation of uncached
    ///   queries, so that these can be created on the fly.
    ///
    /// These mutations are safe in single-threaded applications, but for
    /// multi-threaded applications, the world needs to be entirely immutable. For this
    /// purpose, multi-threaded readonly mode exists, which disallows all mutations on
    /// the world.
    ///
    /// While in readonly mode, applications can still enqueue ECS operations on a
    /// stage. Stages are managed automatically when using the pipeline addon and
    /// [`World::progress()`], but they can also be configured manually.
    ///
    /// Number of stages typically corresponds with number of threads
    ///
    /// When an attempt is made to perform an operation on a world in readonly mode,
    /// the code will throw an assert saying that the world is in readonly mode.
    ///
    /// A call to `readonly_begin` must be followed up with `readonly_end()`.
    /// When `readonly_end()` is called, all enqueued commands from configured
    /// stages are merged back into the world. Calls to `readonly_begin()` and
    /// `readonly_end()` should always happen from a context where the code has
    /// exclusive access to the world. The functions themselves are not thread safe.
    ///
    /// # Safety
    /// This function should only be run from the main thread.
    ///
    /// # Returns
    /// Whether the world is currently staged and whether it is in readonly mode.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Position {
    ///     x: i32,
    ///     y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let stage = world.stage(0);
    ///
    /// world.readonly_begin(false);
    ///
    /// assert_eq!(stage.count(Position::id()), 0);
    ///
    /// world.readonly_end();
    ///
    /// world.readonly_begin(false);
    ///
    /// stage.entity().set(Position { x: 10, y: 20 });
    /// stage.entity().set(Position { x: 10, y: 20 });
    ///
    /// assert_eq!(stage.count(Position::id()), 0);
    ///
    /// world.readonly_end();
    ///
    /// assert_eq!(stage.count(Position::id()), 2);
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::is_readonly()`]
    /// * [`World::readonly_end()`]
    pub fn readonly_begin(&self, multi_threaded: bool) -> bool {
        unsafe { sys::ecs_readonly_begin(self.raw_world.as_ptr(), multi_threaded) }
    }

    /// End readonly mode.
    ///
    /// Leaves staging mode. After this operation, the world may be directly mutated again.
    /// By default, this operation also merges data back into the world, unless auto-merging
    /// was disabled explicitly.
    ///
    /// # Safety
    /// This function should only be run from the main thread.
    ///
    /// # Returns
    ///
    /// Whether the world is currently staged.
    ///
    /// # Example
    ///
    /// see [`World::readonly_begin()`].
    ///
    /// # See also
    ///
    /// * [`World::is_readonly()`]
    /// * [`World::readonly_begin()`]
    pub fn readonly_end(&self) {
        unsafe {
            sys::ecs_readonly_end(self.raw_world.as_ptr());
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
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// assert!(!world.is_readonly());
    ///
    /// world.readonly_begin(false);
    ///
    /// assert!(world.is_readonly());
    ///
    /// world.readonly_end();
    ///
    /// assert!(!world.is_readonly());
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::readonly_begin()`]
    /// * [`World::readonly_end()`]
    pub fn is_readonly(&self) -> bool {
        unsafe { sys::ecs_stage_is_readonly(self.raw_world.as_ptr()) }
    }

    /// Defers operations until the end of the frame.
    ///
    /// When this operation is invoked while iterating, the operations between
    /// [`World::defer_begin()`] and [`World::defer_end()`] are executed at the
    /// end of the frame.
    ///
    /// # Safety
    /// This operation is thread safe.
    ///
    /// # Returns
    /// Whether the operation was successful.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Position {
    ///     x: i32,
    ///     y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world.defer_begin();
    ///
    /// let e = world.entity().set(Position { x: 10, y: 20 });
    ///
    /// assert!(!e.has(Position::id()));
    ///
    /// world.defer_end();
    ///
    /// assert!(e.has(Position::id()));
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::defer()`]
    /// * [`World::defer_end()`]
    /// * [`World::defer_suspend()`]
    /// * [`World::defer_resume()`]
    /// * [`World::is_deferred()`]
    pub fn defer_begin(&self) -> bool {
        unsafe { sys::ecs_defer_begin(self.raw_world.as_ptr()) }
    }

    /// Ends a block of operations to defer.
    ///
    /// This should follow a [`World::defer_begin()`] call.
    ///
    /// # Safety
    /// This operation is thread safe.
    ///
    /// # Returns
    /// Whether the operation was successful.
    ///
    /// # Example
    ///
    /// see [`World::defer_begin`]
    ///
    /// # See also
    ///
    /// * [`World::defer()`]
    /// * [`World::defer_begin()`]
    /// * [`World::defer_suspend()`]
    /// * [`World::defer_resume()`]
    /// * [`World::is_deferred()`]
    pub fn defer_end(&self) -> bool {
        unsafe { sys::ecs_defer_end(self.raw_world.as_ptr()) }
    }

    /// Test whether deferring is enabled.
    ///
    /// # Returns
    ///
    /// Whether deferring is enabled.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// assert!(!world.is_deferred());
    ///
    /// world.defer_begin();
    ///
    /// assert!(world.is_deferred());
    ///
    /// world.defer_end();
    ///
    /// assert!(!world.is_deferred());
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::defer()`]
    /// * [`World::defer_begin()`]
    /// * [`World::defer_end()`]
    /// * [`World::defer_suspend()`]
    /// * [`World::defer_resume()`]
    pub fn is_deferred(&self) -> bool {
        unsafe { sys::ecs_is_deferred(self.raw_world.as_ptr()) }
    }

    /// Defers all operations executed in the passed-in closure.
    ///
    /// # Arguments
    ///
    /// * `func` - The closure to execute.
    ///
    /// # Examples
    /// ```
    /// # use flecs_ecs::core::World;
    /// # let world = World::new();
    /// let return_something_if_wanted = world.defer(|| {
    ///     // deferred operations here
    /// });
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::defer_begin()`]
    /// * [`World::defer_end()`]
    /// * [`World::defer_suspend()`]
    /// * [`World::defer_resume()`]
    /// * [`World::is_deferred()`]
    pub fn defer<T>(&self, func: impl FnOnce() -> T) -> T {
        unsafe {
            sys::ecs_defer_begin(self.raw_world.as_ptr());
        }
        let result = func();
        unsafe {
            sys::ecs_defer_end(self.raw_world.as_ptr());
        }
        result
    }

    /// Suspends deferring of operations but do flush the queue.
    ///
    /// This operation can be used to do an undeferred operation
    /// while not flushing the operations in the queue.
    ///
    /// An application should invoke [`World::defer_resume()`] before
    /// [`World::defer_end()`] is called. The operation may only be called
    /// when deferring is enabled.
    ///
    /// # See also
    ///
    /// * [`World::defer()`]
    /// * [`World::defer_begin()`]
    /// * [`World::defer_end()`]
    /// * [`World::defer_resume()`]
    /// * [`World::is_deferred()`]
    pub fn defer_suspend(&self) {
        unsafe {
            sys::ecs_defer_suspend(self.raw_world.as_ptr());
        }
    }

    /// Resumes deferring of operations.
    ///
    /// # See also
    ///
    /// * [`World::defer()`]
    /// * [`World::defer_begin()`]
    /// * [`World::defer_end()`]
    /// * [`World::defer_suspend()`]
    /// * [`World::is_deferred()`]
    pub fn defer_resume(&self) {
        unsafe {
            sys::ecs_defer_resume(self.raw_world.as_ptr());
        }
    }

    /// Configure world to have N stages.
    ///
    /// This initializes N stages, which allows applications to defer operations to
    /// multiple isolated defer queues. This is typically used for applications with
    /// multiple threads, where each thread gets its own queue, and commands are
    /// merged when threads are synchronized.
    ///
    /// Note that [`World::set_threads()`] already creates the appropriate number of stages.
    /// The [`World::set_stage_count()`] operation is useful for applications that want to manage
    /// their own stages and/or threads.
    ///
    /// # Arguments
    ///
    /// * `stages`: The number of stages.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// world.set_stage_count(2);
    ///
    /// world.readonly_begin(false);
    ///
    /// let stage1 = world.stage(0);
    ///
    /// let e1 = stage1.entity_named("e1");
    ///
    /// world.readonly_end();
    ///
    /// assert!(e1.id() != 0);
    /// assert_eq!(e1.name(), "e1");
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::get_stage_count()`]
    /// * [`World::is_stage()`]
    /// * [`World::merge()`]
    /// * [`World::stage()`]
    /// * [`World::stage_id()`]
    pub fn set_stage_count(&self, stages: i32) {
        unsafe {
            sys::ecs_set_stage_count(self.raw_world.as_ptr(), stages);
        }
    }

    /// Get number of configured stages.
    ///
    /// Return number of stages set by [`World::set_stage_count()`].
    ///
    /// # Returns
    ///
    /// The number of stages used for threading.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// assert_eq!(world.get_stage_count(), 1);
    ///
    /// world.set_stage_count(4);
    ///
    /// assert_eq!(world.get_stage_count(), 4);
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::is_stage()`]
    /// * [`World::merge()`]
    /// * [`World::set_stage_count()`]
    /// * [`World::stage()`]
    /// * [`World::stage_id()`]
    pub fn get_stage_count(&self) -> i32 {
        unsafe { sys::ecs_get_stage_count(self.raw_world.as_ptr()) }
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
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// assert_eq!(world.stage_id(), 0);
    ///
    /// world.set_stage_count(4);
    ///
    /// assert_eq!(world.stage_id(), 0);
    ///
    /// let stage = world.stage(3);
    ///
    /// assert_eq!(stage.stage_id(), 3);
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::get_stage_count()`]
    /// * [`World::is_stage()`]
    /// * [`World::merge()`]
    /// * [`World::set_stage_count()`]
    /// * [`World::stage()`]
    pub fn stage_id(&self) -> i32 {
        unsafe { sys::ecs_stage_get_id(self.raw_world.as_ptr()) }
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
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// assert!(!world.is_stage());
    ///
    /// let stage = world.stage(0);
    ///
    /// assert!(stage.is_stage());
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::get_stage_count()`]
    /// * [`World::merge()`]
    /// * [`World::set_stage_count()`]
    /// * [`World::stage()`]
    /// * [`World::stage_id()`]
    pub fn is_stage(&self) -> bool {
        unsafe {
            ecs_assert!(
                sys::flecs_poly_is_(
                    self.raw_world.as_ptr() as *const c_void,
                    sys::ecs_world_t_magic as i32
                ) || sys::flecs_poly_is_(
                    self.raw_world.as_ptr() as *const c_void,
                    sys::ecs_stage_t_magic as i32
                ),
                FlecsErrorCode::InvalidParameter,
                "flecs::world instance contains invalid reference to world or stage"
            );
            sys::flecs_poly_is_(
                self.raw_world.as_ptr() as *const c_void,
                sys::ecs_stage_t_magic as i32,
            )
        }
    }

    /// Merge world or stage.
    ///
    /// When automatic merging is disabled, an application can call this
    /// operation on either an individual stage, or on the world which will merge
    /// all stages. This operation may only be called when staging is not enabled
    /// (either after [`World::progress()`] or after [`World::readonly_end()`]).
    ///
    /// This operation may be called on an already merged stage or world.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Position {
    ///     x: i32,
    ///     y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let e = world.entity();
    ///
    /// let stage = world.create_async_stage();
    ///
    /// e.mut_current_stage(stage).set(Position { x: 10, y: 20 });
    ///
    /// assert!(!e.has(Position::id()));
    ///
    /// stage.merge();
    ///
    /// assert!(e.has(Position::id()));
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::get_stage_count()`]
    /// * [`World::is_stage()`]
    /// * [`World::set_stage_count()`]
    /// * [`World::stage()`]
    /// * [`World::stage_id()`]
    pub fn merge(&self) {
        unsafe { sys::ecs_merge(self.raw_world.as_ptr()) };
    }

    /// Get stage-specific world pointer.
    ///
    /// Flecs threads can safely invoke the API as long as they have a private
    /// context to write to, also referred to as the stage. This function returns a
    /// pointer to a stage, disguised as a world pointer.
    ///
    /// Note that this function does not(!) create a new world. It simply wraps the
    /// existing world in a thread-specific context, which the API knows how to
    /// unwrap. The reason the stage is returned as an [`sys::ecs_world_t`] is so that
    /// it can be passed transparently to the existing API functions, vs. having to
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
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// assert_eq!(world.stage_id(), 0);
    ///
    /// world.set_stage_count(4);
    ///
    /// assert_eq!(world.stage_id(), 0);
    ///
    /// let stage = world.stage(3);
    ///
    /// assert_eq!(stage.stage_id(), 3);
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::get_stage_count()`]
    /// * [`World::is_stage()`]
    /// * [`World::merge()`]
    /// * [`World::set_stage_count()`]
    /// * [`World::stage_id()`]
    pub fn stage(&self, stage_id: i32) -> WorldRef<'_> {
        unsafe { WorldRef::from_ptr(sys::ecs_get_stage(self.raw_world.as_ptr(), stage_id)) }
    }

    /// Create asynchronous stage.
    ///
    /// An asynchronous stage can be used to asynchronously queue operations for
    /// later merging with the world. An asynchronous stage is similar to a regular
    /// stage, except that it does not allow reading from the world.
    ///
    /// Asynchronous stages are never merged automatically, and must therefore be
    /// manually merged with the [`World::merge()`] function. It is not necessary to call
    /// [`World::defer_begin()`] or [`World::defer_end()`] before and after enqueuing commands,
    /// as an asynchronous stage unconditionally defers operations.
    ///
    /// The application must ensure that no commands are added to the stage while the
    /// stage is being merged.
    ///
    /// An asynchronous stage will be cleaned up when it is dropped.
    ///
    /// # Returns
    ///
    /// The stage.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Position {
    ///     x: i32,
    ///     y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let e = world.entity();
    ///
    /// let stage = world.create_async_stage();
    ///
    /// e.mut_current_stage(stage).set(Position { x: 10, y: 20 });
    ///
    /// assert!(!e.has(Position::id()));
    ///
    /// stage.merge();
    ///
    /// assert!(e.has(Position::id()));
    /// ```
    pub fn create_async_stage(&self) -> WorldRef<'_> {
        unsafe { WorldRef::from_ptr(sys::ecs_stage_new(self.raw_world.as_ptr())) }
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
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let stage = world.stage(0);
    ///
    /// let world_ref = stage.real_world();
    ///
    /// assert!(!world_ref.is_stage());
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::is_stage()`]
    /// * [`World::stage()`]
    /// * [`WorldRef`]
    pub fn real_world(&self) -> WorldRef<'_> {
        self.world().real_world()
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
    /// # Example
    ///
    /// ```
    /// use core::ffi::c_void;
    /// use flecs_ecs::prelude::*;
    ///
    /// #[extern_abi]
    /// fn free_ctx(ctx: *mut c_void) {
    ///     unsafe {
    ///         Box::from_raw(ctx as *mut i32);
    ///     }
    /// }
    ///
    /// let world = World::new();
    ///
    /// let ctx = Box::leak(Box::new(42));
    ///
    /// world.set_context(ctx as *mut i32 as *mut c_void, Some(free_ctx));
    ///
    /// assert_eq!(world.context() as *const i32, ctx);
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::context()`]
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // this doesn't actually deref the pointer
    pub fn set_context(&self, ctx: *mut c_void, ctx_free: sys::ecs_ctx_free_t) {
        unsafe { sys::ecs_set_ctx(self.raw_world.as_ptr(), ctx, ctx_free) }
    }

    /// Get world context.
    ///
    /// # Returns
    ///
    /// The configured world context.
    ///
    /// # Example
    ///
    /// See [`World::set_context`].
    ///
    /// # See also
    ///
    /// * [`World::set_context()`]
    pub fn context(&self) -> *mut c_void {
        unsafe { sys::ecs_get_ctx(self.raw_world.as_ptr()) }
    }

    pub(crate) fn get_context(world: *mut sys::ecs_world_t) -> *mut WorldCtx {
        unsafe { sys::ecs_get_binding_ctx(world) as *mut WorldCtx }
    }

    pub(crate) fn get_components_map(world: *mut sys::ecs_world_t) -> &'static mut FlecsIdMap {
        unsafe { &mut (*(sys::ecs_get_binding_ctx(world) as *mut WorldCtx)).components }
    }

    pub(crate) fn get_components_map_ptr(world: *mut sys::ecs_world_t) -> *mut FlecsIdMap {
        unsafe { &mut (*(sys::ecs_get_binding_ctx(world) as *mut WorldCtx)).components }
    }

    #[doc(hidden)]
    pub fn components_map(&self) -> &'static mut FlecsIdMap {
        unsafe { &mut (*(self.components.as_ptr())) }
    }

    pub(crate) fn get_components_array(world: *mut sys::ecs_world_t) -> &'static mut FlecsArray {
        unsafe { &mut (*(sys::ecs_get_binding_ctx(world) as *mut WorldCtx)).components_array }
    }

    pub(crate) fn get_components_array_ptr(world: *mut sys::ecs_world_t) -> *mut FlecsArray {
        unsafe { &mut (*(sys::ecs_get_binding_ctx(world) as *mut WorldCtx)).components_array }
    }

    #[doc(hidden)]
    pub fn components_array(&self) -> &'static mut FlecsArray {
        unsafe { &mut (*(self.components_array.as_ptr())) }
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
    /// * [`World::get_binding_context()`]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub(crate) fn set_binding_context(&self, ctx: *mut c_void, ctx_free: sys::ecs_ctx_free_t) {
        unsafe { sys::ecs_set_binding_ctx(self.raw_world.as_ptr(), ctx, ctx_free) }
    }

    /// Get world binding context.
    ///
    /// # Returns
    ///
    /// The configured world context.
    ///
    /// # See also
    ///
    /// * [`World::set_binding_context()`]
    pub(crate) fn get_binding_context(&self) -> *mut c_void {
        unsafe { sys::ecs_get_binding_ctx(self.raw_world.as_ptr()) }
    }

    /// Preallocate memory for a number of entities.
    ///
    /// This function preallocates memory for the entity index.
    ///
    /// # Arguments
    ///
    /// * `entity_count` - Number of entities to preallocate memory for.
    pub fn preallocate_entity_count(&self, entity_count: i32) {
        unsafe { sys::ecs_dim(self.raw_world.as_ptr(), entity_count) };
    }

    /// Free unused memory.
    ///
    /// This operation frees allocated memory that is no longer in use by the world.
    /// Examples of allocations that get cleaned up are:
    /// - Unused pages in the entity index
    /// - Component columns
    /// - Empty tables
    ///
    /// Flecs uses allocators internally for speeding up allocations. Allocators are
    /// not evaluated by this function, which means that the memory reported by the
    /// OS may not go down. For this reason, this function is most effective when
    /// combined with `FLECS_USE_OS_ALLOC`, which disables internal allocators.
    pub fn shrink_memory(&self) {
        unsafe { sys::ecs_shrink(self.raw_world.as_ptr()) };
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
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// world.set_entity_range(5000, 0);
    ///
    /// let e = world.entity();
    ///
    /// assert_eq!(e.id(), 5000);
    ///
    /// let e = world.entity();
    ///
    /// assert_eq!(e.id(), 5001);
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::enable_range_check()`]
    /// * [`World::preallocate_entity_count()`]
    pub fn set_entity_range(&self, min: impl Into<Entity>, max: impl Into<Entity>) {
        unsafe { sys::ecs_set_entity_range(self.raw_world.as_ptr(), *min.into(), *max.into()) };
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
    /// # Example
    ///
    /// ```should_panic
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let e = world.entity();
    /// let e2 = world.entity();
    ///
    /// world.set_entity_range(5000, 0);
    /// world.enable_range_check(true);
    ///
    /// e.add(e2); // panics in debug mode! because e and e2 are outside the range
    /// panic!("in release mode, this does not panic, this is to prevent the test from failing")
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::set_entity_range()`]
    pub fn enable_range_check(&self, enabled: bool) {
        unsafe { sys::ecs_enable_range_check(self.raw_world.as_ptr(), enabled) };
    }

    /// Get the current scope. Get the scope set by `set_scope`.
    /// If no scope is set, this operation will return `None`.
    ///
    /// # Returns
    ///
    /// Returns an `EntityView` representing the current scope.
    /// If no scope is set, this operation will return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let e = world.entity_named("scope");
    ///
    /// world.set_scope(e);
    ///
    /// let s = world.get_scope();
    ///
    /// assert_eq!(s.unwrap(), e);
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::set_scope()`]
    #[inline(always)]
    pub fn get_scope(&self) -> Option<EntityView<'_>> {
        let scope = unsafe { sys::ecs_get_scope(self.raw_world.as_ptr()) };

        if scope == 0 {
            None
        } else {
            Some(EntityView::new_from(self, scope))
        }
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
    /// Returns an `EntityView` representing the previous set scope.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let e = world.entity_named("scope");
    ///
    /// // previous scope can be used to set the scope back to the original.
    /// let previous_scope = world.set_scope(e);
    ///
    /// let s = world.get_scope();
    ///
    /// assert_eq!(s.unwrap(), e);
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::get_scope()`]
    /// * [`World::set_scope()`]
    #[inline(always)]
    pub fn set_scope(&self, id: impl IntoId) -> EntityView<'_> {
        EntityView::new_from(self, unsafe {
            sys::ecs_set_scope(self.raw_world.as_ptr(), *id.into_id(self))
        })
    }

    /// Sets the search path for entity lookup operations.
    ///
    /// This function configures the search path used for looking up an entity.
    ///
    /// # Best Practices
    ///
    /// * It's advisable to restore the previous search path after making temporary changes.
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
    ///
    /// # Arguments
    ///
    /// * `search_path` - The entity to set as the search path.
    ///
    /// # Returns
    ///
    /// Returns the current search path after the operation.
    ///
    /// # See also
    ///
    /// * [`World::lookup()`]
    /// * [`World::lookup_recursive()`]
    /// * [`World::try_lookup()`]
    /// * [`World::try_lookup_recursive()`]
    /// * C API: `sys::ecs_set_lookup_path`
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // this doesn't actually deref the pointer
    // TODO we need to improve this function somehow, it's not very ergonomic
    pub fn set_lookup_path(&self, search_path: impl Into<Entity>) -> *mut sys::ecs_entity_t {
        unsafe { sys::ecs_set_lookup_path(self.raw_world.as_ptr(), &*search_path.into()) }
    }

    /// Lookup an entity by name.
    /// The entity is searched recursively recursively traversing
    /// up the tree until found.
    ///
    /// # Panics
    ///
    /// Ensure that the entity exists before using it.
    /// Use the [`World::try_lookup_recursive()`] variant otherwise.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    ///
    /// # Returns
    ///
    /// The entity
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Position {
    ///     x: i32,
    ///     y: i32,
    /// }
    ///
    /// let world = World::new();
    /// let a = world.entity().set(Position { x: 10, y: 20 }).with(|| {
    ///     world.entity_named("X");
    /// });
    ///
    /// let x = world.lookup_recursive("X");
    /// assert!(x.has(a));
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::lookup()`]
    /// * [`World::set_lookup_path()`]
    /// * [`World::try_lookup()`]
    /// * [`World::try_lookup_recursive()`]
    #[inline(always)]
    pub fn lookup_recursive(&self, name: &str) -> EntityView<'_> {
        self.try_lookup_recursive(name).unwrap_or_else(|| {
            panic!("Entity {name} not found, when unsure, use try_lookup_recursive")
        })
    }

    /// Lookup entity by name, only the current scope is searched
    ///
    /// # Panics
    ///
    /// Ensure that the entity exists before using it.
    /// Use the [`World::try_lookup()`] variant otherwise.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    ///
    /// # Returns
    ///
    /// The entity
    ///
    /// # See also
    ///
    /// * [`World::lookup_recursive()`]
    /// * [`World::set_lookup_path()`]
    /// * [`World::try_lookup()`]
    /// * [`World::try_lookup_recursive()`]
    #[inline(always)]
    pub fn lookup(&self, name: &str) -> EntityView<'_> {
        self.try_lookup(name)
            .unwrap_or_else(|| panic!("Entity {name} not found, when unsure, use try_lookup"))
    }

    /// Helper function for [`World::try_lookup()`] and [`World::try_lookup_recursive()`].
    fn try_lookup_impl(&self, name: &str, recursively: bool) -> Option<EntityView<'_>> {
        let name = compact_str::format_compact!("{}\0", name);

        let entity_id = unsafe {
            sys::ecs_lookup_path_w_sep(
                self.raw_world.as_ptr(),
                0,
                name.as_ptr() as *const _,
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
                recursively,
            )
        };
        if entity_id == 0 {
            None
        } else {
            Some(EntityView::new_from(self, entity_id))
        }
    }

    /// Lookup an entity by name.
    /// The entity is searched recursively recursively traversing
    /// up the tree until found.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    ///
    /// # Returns
    ///
    /// The entity if found, otherwise `None`.
    ///
    /// # See also
    ///
    /// * [`World::lookup()`]
    /// * [`World::lookup_recursive()`]
    /// * [`World::set_lookup_path()`]
    /// * [`World::try_lookup()`]
    #[inline(always)]
    pub fn try_lookup_recursive(&self, name: &str) -> Option<EntityView<'_>> {
        self.try_lookup_impl(name, true)
    }

    /// Lookup entity by name, only the current scope is searched
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    ///
    /// # Returns
    ///
    /// The entity if found, otherwise `None`.
    ///
    /// # See also
    ///
    /// * [`World::lookup()`]
    /// * [`World::lookup_recursive()`]
    /// * [`World::set_lookup_path()`]
    /// * [`World::try_lookup_recursive()`]
    #[inline(always)]
    pub fn try_lookup(&self, name: &str) -> Option<EntityView<'_>> {
        self.try_lookup_impl(name, false)
    }

    /// Sets a singleton component of type `T` on the world.
    ///
    /// # Arguments
    ///
    /// * `component` - The singleton component to set on the world.
    pub fn set<T: ComponentId + DataComponent + ComponentType<Struct>>(&self, component: T) {
        let id = T::entity_id(self);
        set_helper(self.raw_world.as_ptr(), id, component, id);
    }

    /// Set a singleton pair using the second element type and a first id.
    ///
    /// # Safety
    ///
    /// Caller must ensure that `First` and `second` pair id data type is the one provided.
    pub fn set_first<First>(&self, second: impl Into<Entity>, first: First)
    where
        First: ComponentId + ComponentType<Struct> + DataComponent,
    {
        let entity = EntityView::new_from(self, First::entity_id(self));
        entity.set_first::<First>(first, second);
    }

    /// Set a singleton pair using the second element type and a first id.
    ///
    /// # Safety
    ///
    /// Caller must ensure that `first` and `Second` pair id data type is the one provided.
    pub fn set_second<Second>(&self, first: impl Into<Entity>, second: Second)
    where
        Second: ComponentId + ComponentType<Struct> + DataComponent,
    {
        let entity = EntityView::new_from(self, Second::entity_id(self));
        entity.set_second::<Second>(first, second);
    }

    /// Set singleton pair.
    /// This operation sets the pair value, and uses the first non tag / ZST as type. If the
    /// entity did not yet have the pair, it will be added, otherwise overridden.
    pub fn set_pair<First, Second>(&self, data: <(First, Second) as ComponentOrPairId>::CastType)
    where
        First: ComponentId,
        Second: ComponentId,
        (First, Second): ComponentOrPairId,
    {
        const {
            assert!(
                !<(First, Second) as ComponentOrPairId>::IS_TAGS,
                "setting tag relationships is not possible with `set_pair`. use `add_pair` instead."
            );
        };

        let entity = EntityView::new_from(
            self,
            <<(First, Second) as ComponentOrPairId>::CastType as ComponentId>::entity_id(self),
        );
        entity.set_pair::<First, Second>(data);
    }

    /// assign a component for an entity.
    /// This operation sets the component value. If the entity did not yet have
    /// the component the operation will panic.
    pub fn assign<T: ComponentId + DataComponent>(&self, value: T) {
        let id = T::entity_id(self);
        assign_helper(self.ptr_mut(), id, value, id);
    }

    /// assign a component for an entity.
    /// This operation sets the component value. If the entity did not yet have
    /// the component the operation will panic.
    pub fn assign_id<T: ComponentId + DataComponent>(&self, value: T, id: impl IntoId) {
        let id = *id.into_id(self);
        assign_helper(self.ptr_mut(), id, value, id);
    }

    /// assign a component for an entity.
    /// This operation sets the component value. If the entity did not yet have
    /// the component the operation will panic.
    pub fn assign_pair<First, Second>(
        &self,
        value: <(First, Second) as ComponentOrPairId>::CastType,
    ) where
        First: ComponentId,
        Second: ComponentId,
        (First, Second): ComponentOrPairId,
    {
        let entity = EntityView::new_from(
            self,
            <<(First, Second) as ComponentOrPairId>::CastType as ComponentId>::entity_id(self),
        );

        entity.assign_pair::<First, Second>(value);
    }

    /// assign a component for an entity.
    /// This operation sets the component value. If the entity did not yet have
    /// the component the operation will panic.
    pub fn assign_first<First>(&self, first: First, second: impl Into<Entity>)
    where
        First: ComponentId + DataComponent,
    {
        let entity = EntityView::new_from(self, First::entity_id(self));
        entity.assign_first::<First>(first, second);
    }

    /// assign a component for an entity.
    /// This operation sets the component value. If the entity did not yet have
    /// the component the operation will panic.
    pub fn assign_second<Second>(&self, first: impl Into<Entity>, second: Second)
    where
        Second: ComponentId + DataComponent,
    {
        let entity = EntityView::new_from(self, Second::entity_id(self));
        entity.assign_second::<Second>(first, second);
    }

    /// signal that singleton component was modified.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the component that was modified.
    ///
    /// # See also
    ///
    /// * [`EntityView::modified()`]
    /// * [`World::modified()`]
    #[inline(always)]
    pub fn modified(&self, id: impl Into<Entity>) {
        let id = id.into();
        EntityView::new_from(self, id).modified(id);
    }

    /// set the version of the provided entity.
    pub fn set_version(&self, entity: impl Into<Entity>) {
        unsafe { sys::ecs_set_version(self.raw_world.as_ptr(), *entity.into()) };
    }
}

pub trait WorldGet<Return> {
    /// gets a mutable or immutable singleton component and/or relationship(s) from the world and return a value.
    /// Only one singleton component at a time is retrievable, but you can call this function multiple times within the callback.
    /// each component type must be marked `&` or `&mut` to indicate if it is mutable or not.
    /// use `Option` wrapper to indicate if the component is optional.
    ///
    /// - `try_get` assumes when not using `Option` wrapper, that the entity has the component.
    ///   If it does not, it will not run the callback.
    ///   If unsure and you still want to have the callback be ran, use `Option` wrapper instead.
    ///
    /// # Note
    ///
    /// - You cannot get single component tags with this function, use `has` functionality instead.
    /// - You can only get relationships with a payload, so where one is not a tag / not a zst.
    ///   tag relationships, use `has` functionality instead.
    /// - This causes the table to lock where the entity belongs to to prevent invalided references, see #Panics.
    ///   The lock is dropped at the end of the callback.
    ///
    /// # Panics
    ///
    /// - This will panic if within the callback you do any operation that could invalidate the reference.
    ///   This happens when the entity is moved to a different table in memory. Such as adding, removing components or
    ///   creating/deleting entities where the entity belongs to the same table (which could cause a table grow operation).
    ///   In case you need to do such operations, you can either do it after the get operation or defer the world with `world.defer_begin()`.
    ///
    /// # Returns
    ///
    /// - If the callback was run, the return value of the callback wrapped in [`Some`]
    /// - Otherwise, returns [`None`]
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Tag;
    ///
    /// #[derive(Component)]
    /// pub struct Velocity {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// #[derive(Component)]
    /// pub struct Position {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world.set(Position { x: 10.0, y: 20.0 });
    /// world.set_pair::<Tag, Position>(Position { x: 30.0, y: 40.0 });
    ///
    /// let val = world.try_get::<&Position>(|pos| pos.x);
    /// assert_eq!(val, Some(10.0));
    ///
    /// let val = world.try_get::<&Velocity>(|vel| vel.x);
    /// assert_eq!(val, None);
    ///
    /// let has_run = world
    ///     .try_get::<&mut (Tag, Position)>(|pos| {
    ///         assert_eq!(pos.x, 30.0);
    ///     })
    ///     .is_some();
    /// assert!(has_run);
    /// ```
    fn try_get<T: GetTupleTypeOperation>(
        &self,
        callback: impl for<'e> FnOnce(T::ActualType<'e>) -> Return,
    ) -> Option<Return>
    where
        T::OnlyType: ComponentOrPairId;

    /// gets a mutable or immutable singleton component and/or relationship(s) from the world and return a value.
    /// Only one singleton component at a time is retrievable, but you can call this function multiple times within the callback.
    /// each component type must be marked `&` or `&mut` to indicate if it is mutable or not.
    /// use `Option` wrapper to indicate if the component is optional.
    ///
    /// # Note
    ///
    /// - You cannot get single component tags with this function, use `has` functionality instead.
    /// - You can only get relationships with a payload, so where one is not a tag / not a zst.
    ///   tag relationships, use `has` functionality instead.
    /// - This causes the table to lock where the entity belongs to to prevent invalided references, see #Panics.
    ///   The lock is dropped at the end of the callback.
    ///
    /// # Panics
    ///
    /// - This will panic if within the callback you do any operation that could invalidate the reference.
    ///   This happens when the entity is moved to a different table in memory. Such as adding, removing components or
    ///   creating/deleting entities where the entity belongs to the same table (which could cause a table grow operation).
    ///   In case you need to do such operations, you can either do it after the get operation or defer the world with `world.defer_begin()`.
    ///
    /// - `get` assumes when not using `Option` wrapper, that the entity has the component.
    ///   This will panic if the entity does not have the component. If unsure, use `Option` wrapper or `try_get` function instead.
    ///   `try_get` does not run the callback if the entity does not have the component that isn't marked `Option`.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Tag;
    ///
    /// #[derive(Component)]
    /// pub struct Position {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world.set(Position { x: 10.0, y: 20.0 });
    /// world.set_pair::<Tag, Position>(Position { x: 30.0, y: 40.0 });
    ///
    /// let val = world.get::<&Position>(|pos| pos.x);
    /// assert_eq!(val, 10.0);
    ///
    /// world.get::<&mut (Tag, Position)>(|pos| {
    ///     assert_eq!(pos.x, 30.0);
    /// });
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::cloned()`]
    fn get<T: GetTupleTypeOperation>(
        &self,
        callback: impl for<'e> FnOnce(T::ActualType<'e>) -> Return,
    ) -> Return
    where
        T::OnlyType: ComponentOrPairId;
}

impl<Return> WorldGet<Return> for World {
    fn try_get<T: GetTupleTypeOperation>(
        &self,
        callback: impl for<'e> FnOnce(T::ActualType<'e>) -> Return,
    ) -> Option<Return>
    where
        T::OnlyType: ComponentOrPairId,
    {
        let entity = EntityView::new_from(
            self,
            <<T::OnlyType as ComponentOrPairId>::CastType>::entity_id(self),
        );
        entity.try_get::<T>(callback)
    }

    fn get<T: GetTupleTypeOperation>(
        &self,
        callback: impl for<'e> FnOnce(T::ActualType<'e>) -> Return,
    ) -> Return
    where
        T::OnlyType: ComponentOrPairId,
    {
        let entity = EntityView::new_from(
            self,
            <<T::OnlyType as ComponentOrPairId>::CastType>::entity_id(self),
        );
        entity.get::<T>(callback)
    }
}

impl World {
    /// Clones a singleton component and/or relationship from the world and returns it.
    /// each component type must be marked `&`. This helps Rust type checker to determine if it's a relationship.
    /// use `Option` wrapper to indicate if the component is optional.
    /// use `()` tuple format when getting multiple components.
    ///
    /// # Note
    ///
    /// - You cannot clone component tags with this function.
    /// - You can only clone relationships with a payload, so where one is not a tag / not a zst.
    ///
    /// # Panics
    ///
    /// - This will panic if the world does not have the singleton component that isn't marked `Option`.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Tag;
    ///
    /// #[derive(Component, Clone)]
    /// pub struct Position {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// #[derive(Component, Clone)]
    /// pub struct Velocity {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world.set(Position { x: 10.0, y: 20.0 });
    /// world.set_pair::<Tag, Position>(Position { x: 30.0, y: 40.0 });
    ///
    /// let pos = world.cloned::<&Position>();
    /// assert_eq!(pos.x, 10.0);
    ///
    /// let tag_pos = world.cloned::<&(Tag, Position)>();
    /// assert_eq!(tag_pos.x, 30.0);
    ///
    /// let vel = world.cloned::<Option<&Velocity>>();
    /// assert!(vel.is_none());
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::get()`]
    #[must_use]
    pub fn cloned<T: ClonedTupleTypeOperation>(&self) -> T::ActualType
    where
        T::OnlyType: ComponentOrPairId,
    {
        let entity = EntityView::new_from(
            self,
            <<T::OnlyType as ComponentOrPairId>::CastType>::entity_id(self),
        );
        entity.cloned::<T>()
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
    // #[doc(alias = "world::get_ref")]
    // #[inline(always)]
    pub fn get_ref<T>(&self) -> CachedRef<'_, T::CastType>
    where
        T: ComponentOrPairId,
        T::CastType: DataComponent,
    {
        EntityView::new_from(self, T::get_id(self)).get_ref::<T>()
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
    #[inline(always)]
    pub fn singleton<T: ComponentId>(&self) -> EntityView<'_> {
        EntityView::new_from(self, T::entity_id(self))
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
    /// * [`World::target()`]
    pub fn target(&self, relationship: impl IntoEntity, index: Option<usize>) -> EntityView<'_> {
        let relationship = *relationship.into_entity(self);
        EntityView::new_from(self, unsafe {
            sys::ecs_get_target(
                self.raw_world.as_ptr(),
                relationship,
                relationship,
                index.unwrap_or(0) as i32,
            )
        })
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
    /// * [`World::has()`]
    /// * [`World::has_enum()`]
    #[inline(always)]
    pub fn has<T: IntoId>(&self, id: T) -> bool {
        let id = *id.into_id(self);
        if T::IS_PAIR {
            let first_id = id.get_id_first(self);
            EntityView::new_from(self, first_id).has(id)
        } else {
            EntityView::new_from(self, id).has(id)
        }
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
    /// * [`World::has()`]
    #[inline(always)]
    pub fn has_enum<T>(&self, constant: T) -> bool
    where
        T: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        EntityView::new_from(self, T::entity_id(self)).has_enum(constant)
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
    /// `EntityView` handle to the singleton component.
    #[inline(always)]
    pub fn add<T: IntoId>(&self, id: T) -> EntityView<'_> {
        let id = *id.into_id(self);
        // this branch will compile out in release mode
        if T::IS_PAIR {
            let first_id = id.get_id_first(self);
            EntityView::new_from(self, first_id).add(id)
        } else {
            EntityView::new_from(self, id).add(id)
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
    /// `EntityView` handle to the singleton enum component.
    #[inline(always)]
    pub fn add_enum<T: ComponentId + ComponentType<Enum> + EnumComponentInfo>(
        &self,
        enum_value: T,
    ) -> EntityView<'_> {
        EntityView::new_from(self, T::entity_id(self)).add_enum::<T>(enum_value)
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
    /// `EntityView` handle to the singleton pair.
    #[inline(always)]
    pub fn add_pair_enum<First, Second>(&self, enum_value: Second) -> EntityView<'_>
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        EntityView::new_from(self, First::entity_id(self))
            .add_pair_enum::<First, Second>(enum_value)
    }

    /// Remove singleton component by id.
    /// id can be a component, entity or pair id.
    ///
    /// # Arguments
    ///
    /// * `id`: The id of the component to remove.
    pub fn remove<T: IntoId>(&self, id: T) -> EntityView<'_> {
        let id = *id.into_id(self);
        if T::IS_PAIR {
            let first_id = id.get_id_first(self);
            EntityView::new_from(self, first_id).remove(id)
        } else {
            EntityView::new_from(self, id).remove(id)
        }
    }

    /// Iterate entities in root of world
    ///
    /// # Arguments
    ///
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(EntityView)`.
    #[inline(always)]
    pub fn each_child(&self, callback: impl FnMut(EntityView)) {
        EntityView::new(self).each_child(callback);
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
    #[inline(always)]
    pub fn set_alias_component<T: ComponentId>(&self, alias: &str) -> EntityView<'_> {
        let alias = compact_str::format_compact!("{}\0", alias);

        let id = T::entity_id(self);
        if alias.is_empty() {
            unsafe {
                sys::ecs_set_alias(
                    self.raw_world.as_ptr(),
                    id,
                    sys::ecs_get_name(self.raw_world.as_ptr(), id),
                );
            };
        } else {
            unsafe { sys::ecs_set_alias(self.raw_world.as_ptr(), id, alias.as_ptr() as *const _) };
        }
        EntityView::new_from(self, id)
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
    #[inline(always)]
    pub fn set_alias_entity_by_name(&self, name: &str, alias: &str) -> EntityView<'_> {
        let name = compact_str::format_compact!("{}\0", name);
        let alias = compact_str::format_compact!("{}\0", alias);

        let id = unsafe {
            sys::ecs_lookup_path_w_sep(
                self.raw_world.as_ptr(),
                0,
                name.as_ptr() as *const _,
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
                true,
            )
        };
        ecs_assert!(id != 0, FlecsErrorCode::InvalidParameter);
        unsafe { sys::ecs_set_alias(self.raw_world.as_ptr(), id, alias.as_ptr() as *const _) };
        EntityView::new_from(self, id)
    }

    /// create alias for entity
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to create an alias for.
    /// * `alias` - The alias to create.
    #[inline(always)]
    pub fn set_alias_entity(&self, entity: impl Into<Entity>, alias: &str) {
        let alias = compact_str::format_compact!("{}\0", alias);

        let entity = *entity.into();
        if alias.is_empty() {
            unsafe {
                sys::ecs_set_alias(
                    self.raw_world.as_ptr(),
                    entity,
                    sys::ecs_get_name(self.raw_world.as_ptr(), entity),
                );
            };
        } else {
            unsafe {
                sys::ecs_set_alias(self.raw_world.as_ptr(), entity, alias.as_ptr() as *const _);
            };
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
    pub fn count(&self, id: impl IntoId) -> i32 {
        unsafe { sys::ecs_count_id(self.raw_world.as_ptr(), *id.into_id(self)) }
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
    pub fn count_enum<T: ComponentId + ComponentType<Enum> + EnumComponentInfo>(
        &self,
        enum_value: T,
    ) -> i32 {
        unsafe { sys::ecs_count_id(self.raw_world.as_ptr(), *(enum_value.id_variant(self).id)) }
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
    pub fn count_enum_tag_pair<First, Second>(&self, enum_value: Second) -> i32
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        unsafe {
            sys::ecs_count_id(
                self.raw_world.as_ptr(),
                ecs_pair(First::entity_id(self), *(enum_value.id_variant(self)).id),
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
    pub fn run_in_scope_with(&self, parent_id: impl IntoEntity, mut func: impl FnMut()) {
        let world = self.world();
        let prev: sys::ecs_id_t =
            unsafe { sys::ecs_set_scope(self.raw_world.as_ptr(), *parent_id.into_entity(world)) };
        func();
        unsafe {
            sys::ecs_set_scope(self.raw_world.as_ptr(), prev);
        }
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
    pub fn scope(&self, parent_id: impl IntoId, mut f: impl FnMut(&World)) {
        let previous_scope = self.set_scope(parent_id);
        f(self);
        self.set_scope(previous_scope);
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
    pub fn scope_name(&self, name: &str, f: impl FnMut(&World)) {
        self.scope(EntityView::new_named(self, name).id, f);
    }

    /// all entities created in function are created with id
    ///
    /// # Arguments
    ///
    /// * `id`: The id to create entities with.
    /// * `func`: The function to run.
    pub fn with(&self, id: impl IntoId, mut func: impl FnMut()) {
        let prev: sys::ecs_id_t =
            unsafe { sys::ecs_set_with(self.raw_world.as_ptr(), *id.into_id(self)) };
        func();
        unsafe {
            sys::ecs_set_with(self.raw_world.as_ptr(), prev);
        }
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
    pub fn with_enum<T>(&self, enum_value: T, func: impl FnMut())
    where
        T: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        self.with(enum_value.id_variant(self), func);
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
    pub fn with_enum_pair<First, Second>(&self, enum_value: Second, func: impl FnMut())
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        self.with(
            ecs_pair(First::entity_id(self), **(enum_value.id_variant(self))),
            func,
        );
    }

    /// Delete all entities with the given id
    ///
    /// # Arguments
    ///
    /// * `id`: The id to delete.
    pub fn delete_entities_with(&self, id: impl IntoId) {
        unsafe {
            sys::ecs_delete_with(self.raw_world.as_ptr(), *id.into_id(self));
        }
    }

    /// Delete all entities with the given enum constant
    ///
    /// # Type Parameters
    ///
    /// * `T`: The enum type.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to query against for deletion.
    pub fn delete_with_enum<T: ComponentId + ComponentType<Enum> + EnumComponentInfo>(
        &self,
        enum_value: T,
    ) {
        self.delete_entities_with(enum_value.id_variant(self));
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
    /// * `enum_value`: The enum value to query against for deletion.
    ///
    /// # See also
    ///
    /// * `world::delete_with`
    pub fn delete_with_enum_pair<First, Second>(&self, enum_value: Second)
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        self.delete_entities_with(ecs_pair(
            First::entity_id(self),
            **enum_value.id_variant(self),
        ));
    }

    /// Remove all instances of the given id from entities
    ///
    /// # Arguments
    ///
    /// * `id`: The id to remove.
    pub fn remove_all(&self, id: impl IntoId) {
        unsafe {
            sys::ecs_remove_all(self.raw_world.as_ptr(), *id.into_id(self));
        }
    }

    /// Remove all instances with the given enum constant from entities
    ///
    /// # Type Parameters
    ///
    /// * `T`: The enum type.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to query against for removal.
    pub fn remove_all_enum<T: ComponentId + ComponentType<Enum> + EnumComponentInfo>(
        &self,
        enum_value: T,
    ) {
        self.remove_all(enum_value.id_variant(self));
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
    /// * `enum_value`: The enum value to query against for removal.
    pub fn remove_all_enum_pair<First, Second>(&self, enum_value: Second)
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        self.remove_all((First::entity_id(self), enum_value.id_variant(self)));
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
    pub fn exists(&self, entity: impl Into<Entity>) -> bool {
        unsafe { sys::ecs_exists(self.raw_world.as_ptr(), *entity.into()) }
    }

    /// Checks if the given entity ID is alive in the world.
    pub fn is_alive(&self, entity: impl Into<Entity>) -> bool {
        unsafe { sys::ecs_is_alive(self.raw_world.as_ptr(), *entity.into()) }
    }

    /// Checks if the given entity ID is valid.
    /// Invalid entities cannot be used with API functions.
    pub fn is_valid(&self, entity: impl Into<Entity>) -> bool {
        unsafe { sys::ecs_is_valid(self.raw_world.as_ptr(), *entity.into()) }
    }

    /// Get alive entity for id.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check
    ///
    /// # Returns
    ///
    /// The entity with the current generation. If the entity is not alive, this
    /// function will return an Entity of 0. Use `try_get_alive` if you want to
    /// return an `Option<EntityView>`.
    pub fn get_alive(&self, entity: impl Into<Entity>) -> EntityView<'_> {
        let entity = unsafe { sys::ecs_get_alive(self.raw_world.as_ptr(), *entity.into()) };

        EntityView::new_from(self, entity)
    }

    /// Get alive entity for id.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check
    ///
    /// # Returns
    ///
    /// The entity with the current generation.
    /// If the entity is not alive, this function will return `None`.
    pub fn try_get_alive(&self, entity: impl Into<Entity>) -> Option<EntityView<'_>> {
        let entity = unsafe { sys::ecs_get_alive(self.raw_world.as_ptr(), *entity.into()) };
        if entity == 0 {
            None
        } else {
            Some(EntityView::new_from(self, entity))
        }
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
    pub fn make_alive(&self, entity: impl Into<Entity>) -> EntityView<'_> {
        let entity = *entity.into();
        unsafe { sys::ecs_make_alive(self.raw_world.as_ptr(), entity) };
        EntityView::new_from(self, entity)
    }

    /// Run callback after completing frame
    ///
    /// # Arguments
    ///
    /// * `action` - The action to run.
    /// * `ctx` - The context to pass to the action.
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // this doesn't actually deref the pointer
    pub fn run_post_frame(&self, action: sys::ecs_fini_action_t, ctx: *mut c_void) {
        unsafe {
            sys::ecs_run_post_frame(self.raw_world.as_ptr(), action, ctx);
        }
    }
}

/// `EntityView` mixin implementation
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
    /// `EntityView` wrapping the id of the enum constant.
    #[doc(alias = "world::id")] //enum mixin implementation
    pub fn entity_from_enum<T>(&self, enum_value: T) -> EntityView<'_>
    where
        T: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        EntityView::new_from(self, enum_value.id_variant(self))
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
    pub fn entity_from_named<'a, T: ComponentId>(&'a self, name: &str) -> EntityView<'a> {
        EntityView::new_from(self, T::__register_or_get_id_named::<true>(self, name))
    }

    /// Create an entity that's associated with a type
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to associate with the new entity.
    pub fn entity_from<T: ComponentId>(&self) -> EntityView<'_> {
        EntityView::new_from(self, T::entity_id(self))
    }

    /// Create an entity that's associated with a name.
    /// The name does an extra allocation if it's bigger than 24 bytes. To avoid this, use `entity_named_cstr`.
    /// length of 24 bytes: `"hi this is 24 bytes long"`
    ///
    /// Named entities can be looked up with the lookup functions. Entity names
    /// may be scoped, where each element in the name is separated by "::".
    /// For example: "`Foo::Bar`". If parts of the hierarchy in the scoped name do
    /// not yet exist, they will be automatically created.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let entity = world.entity_named("Foo");
    /// assert_eq!(entity.get_name(), Some("Foo".to_string()));
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::entity()`]
    /// * [`World::entity_named_cstr()`]
    pub fn entity_named(&self, name: &str) -> EntityView<'_> {
        EntityView::new_named(self, name)
    }

    /// Create an entity that's associated with a name within a scope, using a custom separator and root separator.
    /// The name does an extra allocation if it's bigger than 24 bytes. To avoid this, use `entity_named_cstr`.
    /// length of 24 bytes: `"hi this is 24 bytes long"`
    ///
    /// Named entities can be looked up with the lookup functions. Entity names
    /// may be scoped, where each element in the name is separated by the sep you use.
    /// For example: "`Foo-Bar`". If parts of the hierarchy in the scoped name do
    /// not yet exist, they will be automatically created.
    /// Note, this does still create the hierarchy as `Foo::Bar`.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let entity = world.entity_named_scoped("Foo-Bar", "-", "::");
    /// assert_eq!(entity.get_name(), Some("Bar".to_string()));
    /// assert_eq!(entity.path(), Some("::Foo::Bar".to_string()));
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::entity()`]
    /// * [`World::entity_named_cstr()`]
    pub fn entity_named_scoped(&self, name: &str, sep: &str, root_sep: &str) -> EntityView<'_> {
        EntityView::new_named_scoped(self, name, sep, root_sep)
    }

    /// Create an entity that's associated with a name.
    /// The name must be a valid C str. No extra allocation is done.
    ///
    /// Named entities can be looked up with the lookup functions. Entity names
    /// may be scoped, where each element in the name is separated by "::".
    /// For example: "`Foo::Bar`". If parts of the hierarchy in the scoped name do
    /// not yet exist, they will be automatically created.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let entity = world.entity_named("Foo");
    /// assert_eq!(entity.get_name(), Some("Foo".to_string()));
    /// ```
    ///
    /// # See also
    ///
    /// * [`World::entity()`]
    /// * [`World::entity_named()`]
    pub fn entity_named_cstr(&self, name: &CStr) -> EntityView<'_> {
        EntityView::new_named_cstr(self, name)
    }

    /// Create a new entity.
    ///
    /// # See also
    ///
    /// * [`World::entity_named()`]
    /// * [`World::entity_named_cstr()`]
    #[inline(always)]
    pub fn entity(&self) -> EntityView<'_> {
        EntityView::new(self)
    }

    /// Create entity with id 0.
    /// This function is useful when the API must provide an entity that
    /// belongs to a world, but the entity id is 0.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    /// let entity = world.entity_null();
    /// assert_eq!(entity.id(), 0);
    /// ```
    pub fn entity_null(&self) -> EntityView<'_> {
        EntityView::new_null(self)
    }

    /// wraps an [`EntityView`] with the provided id.
    ///
    /// # Arguments
    ///
    /// * `id` - The id to use for the new entity.
    pub fn entity_from_id(&self, id: impl Into<Entity>) -> EntityView<'_> {
        EntityView::new_from(self, id.into())
    }

    /// Creates a prefab
    ///
    /// # Returns
    ///
    /// The prefab entity.
    ///
    /// # See also
    ///
    /// * [`World::prefab_named()`]
    /// * [`World::prefab_type()`]
    /// * [`World::prefab_type_named()`]
    pub fn prefab(&self) -> EntityView<'_> {
        let result = EntityView::new(self);
        result.add(id::<flecs::Prefab>());
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
    /// * [`World::prefab()`]
    /// * [`World::prefab_type()`]
    /// * [`World::prefab_type_named()`]
    pub fn prefab_named<'a>(&'a self, name: &str) -> EntityView<'a> {
        let result = EntityView::new_named(self, name);
        unsafe { result.add_id_unchecked(ECS_PREFAB) };
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
    /// * [`World::prefab()`]
    /// * [`World::prefab_named()`]
    /// * [`World::prefab_type_named()`]
    pub fn prefab_type<T: ComponentId>(&self) -> EntityView<'_> {
        let result = self.entity_from::<T>();
        result.add(ECS_PREFAB);
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
    /// * [`World::prefab()`]
    /// * [`World::prefab_named()`]
    /// * [`World::prefab_type()`]
    pub fn prefab_type_named<'a, T: ComponentId>(&'a self, name: &str) -> EntityView<'a> {
        let result = self.entity_from_named::<T>(name);
        result.add(ECS_PREFAB);
        result
    }
}
/// Id mixin implementation
impl World {
    /// Get the id of the provided component type.
    /// This returns the id of a component type which has been registered with [`ComponentId`] trait.
    #[inline(always)]
    pub fn component_id<T: ComponentId>(&self) -> Entity {
        Entity(T::entity_id(self))
    }

    /// Get the id of the provided component type.
    /// This returns the id of a component type which has potentially not been registered with `ComponentId` trait.
    /// This holds ids for external components (not marked with the trait), such as in cases of meta fields.
    /// When meta is enabled, this will also hold ids for components that are registered with the `ComponentId` trait.
    pub(crate) fn component_id_map<T: 'static>(&self) -> u64 {
        *self.components_map()
            .get(&core::any::TypeId::of::<T>())
            .unwrap_or_else(|| panic!("Component with name: {} is not registered, pre-register components with `world.component::<T>() or world.component_ext::<T>(id)`", core::any::type_name::<T>()))
    }

    /// Get the id of the provided pair of components.
    pub fn id_from<T: IntoId>(&self, id: T) -> Id {
        Id(*id.into_id(self))
    }

    /// get `IdView` from an id or from a relationship pair
    ///
    /// # Arguments
    ///
    /// * `id` - The id to convert to an `IdView`.
    ///
    /// # Returns
    ///
    /// The `IdView` from the provided id.
    pub fn id_view_from<Id: IntoId>(&self, id: Id) -> IdView<'_> {
        let id = *id.into_id(self);
        if Id::IS_PAIR {
            ecs_assert!(
                {
                    let first = ecs_first(id, self);
                    let second = ecs_second(id, self);
                    !ecs_is_pair(first) && !ecs_is_pair(second)
                },
                FlecsErrorCode::InvalidParameter,
                "cannot create nested pairs"
            );
        }

        IdView::new_from_id(self, id)
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
    pub fn component<T: ComponentId>(&self) -> Component<'_, T::UnderlyingType> {
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
    pub fn component_named<'a, T: ComponentId>(
        &'a self,
        name: &str,
    ) -> Component<'a, T::UnderlyingType> {
        Component::<T::UnderlyingType>::new_named(self, name)
    }

    /// Create new untyped component.
    pub fn component_untyped(&self) -> UntypedComponent<'_> {
        UntypedComponent::new(self)
    }

    /// Create new named untyped component.
    pub fn component_untyped_named(&self, name: &str) -> UntypedComponent<'_> {
        UntypedComponent::new_named(self, name)
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
    pub fn component_untyped_from(&self, id: impl IntoEntity) -> UntypedComponent<'_> {
        UntypedComponent::new_from(self, id)
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
    pub fn to_entity<T: ComponentId + ComponentType<Enum> + EnumComponentInfo>(
        &self,
        enum_value: T,
    ) -> EntityView<'_> {
        EntityView::new_from(self, enum_value.id_variant(self))
    }
}

/// Term mixin implementation
impl World {
    // /// Creates a term for a (component) type.
    // ///
    // /// # Type Parameters
    // ///
    // /// * `T` - The component type.
    // ///
    // /// # Returns
    // ///
    // /// The term for the component type.
    // #[doc(alias = "world::term")]
    // pub fn term<T: ComponentOrPairId>(&self) -> Term {
    //     Term::new_type::<T>(self)
    // }
}

// Event mixin implementation
impl World {
    /// Create a new event builder (untyped) from entity id which represents an event
    ///
    /// # Safety
    /// Caller must ensure that `event` is a ZST or that a pointer to the associated type is set on the builder
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
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue()`]
    /// * [`World::event()`]
    pub unsafe fn event_id(&self, event: impl Into<Entity>) -> EventBuilder<'_, ()> {
        EventBuilder::<()>::new_untyped(self, event)
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
    /// * [`EntityView::emit()`]
    /// * [`EntityView::enqueue()`]
    /// * [`World::event_id()`]
    pub fn event<T: ComponentId>(&self) -> EventBuilder<'_, T> {
        EventBuilder::<T>::new(self)
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
    pub fn observer_from<'a>(&'a self, e: EntityView<'a>) -> Observer<'a> {
        Observer::new_from_existing(e)
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
    /// * [`World::observer_from()`]
    /// * [`World::observer_id()`]
    /// * [`World::observer_named()`]
    pub fn observer<Event: ComponentId, Components>(&self) -> ObserverBuilder<'_, Event, Components>
    where
        Components: QueryTuple,
    {
        ObserverBuilder::<Event, Components>::new(self)
    }

    pub fn observer_id<Components>(
        &self,
        event: impl Into<Entity>,
    ) -> ObserverBuilder<'_, (), Components>
    where
        Components: QueryTuple,
    {
        let mut builder = ObserverBuilder::<(), Components>::new_untyped(self);
        builder.add_event(event);
        builder
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
    /// * [`World::observer_from()`]
    /// * [`World::observer()`]
    /// * [`World::observer_id()`]
    pub fn observer_named<'a, Event: ComponentId, Components>(
        &'a self,
        name: &str,
    ) -> ObserverBuilder<'a, Event, Components>
    where
        Components: QueryTuple,
    {
        ObserverBuilder::<Event, Components>::new_named(self, name)
    }
}

/// Query mixin implementation
impl World {
    /// Create a new uncached [`Query`].
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # See also
    ///
    /// * [`World::new_query()`]
    /// * [`World::new_query_named()`]
    /// * [`World::query()`]
    /// * [`World::query_named()`]
    pub fn new_query<Components>(&self) -> Query<Components>
    where
        Components: QueryTuple,
    {
        QueryBuilder::<Components>::new(self).build()
    }

    /// Create a new named [`Query`].
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
    /// * [`World::new_query()`]
    /// * [`World::new_query_named()`]
    /// * [`World::query()`]
    /// * [`World::query_named()`]
    pub fn new_query_named<Components>(&self, name: &str) -> Query<Components>
    where
        Components: QueryTuple,
    {
        QueryBuilder::<Components>::new_named(self, name).build()
    }

    /// Create a new [`QueryBuilder`].
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
    /// * [`World::new_query()`]
    /// * [`World::new_query_named()`]
    /// * [`World::query_named()`]
    pub fn query<Components>(&self) -> QueryBuilder<'_, Components>
    where
        Components: QueryTuple,
    {
        QueryBuilder::<Components>::new(self)
    }

    /// Create a new named [`QueryBuilder`].
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
    /// * [`World::new_query()`]
    /// * [`World::new_query_named()`]
    /// * [`World::query()`]
    pub fn query_named<'a, Components>(&'a self, name: &str) -> QueryBuilder<'a, Components>
    where
        Components: QueryTuple,
    {
        QueryBuilder::<Components>::new_named(self, name)
    }

    /// Create a query from a query description.
    ///
    /// # Safety
    ///
    /// Caller needs to ensure the query type is correct of the provided `desc`.
    pub unsafe fn query_from_desc<Components>(
        &self,
        desc: &mut sys::ecs_query_desc_t,
    ) -> QueryBuilder<'_, Components>
    where
        Components: QueryTuple,
    {
        QueryBuilder::<Components>::new_from_desc(self, desc)
    }

    /// Attempts to convert an entity into a query.
    ///
    /// Returns the untyped query if the entity is alive and valid; otherwise, returns `None`.
    ///
    /// # Safety
    ///
    /// Proceed with caution. Use `.iter_only` instead.
    ///
    /// # See Also
    ///
    /// * [`World::query_from`]
    pub fn try_query_from(&self, query_entity: impl Into<Entity>) -> Option<Query<()>> {
        Query::<()>::new_from_entity(self, query_entity)
    }

    /// Converts an entity into a query, automatically unwrapping the result.
    ///
    /// This method panics if the entity is not alive or not a valid query.
    /// For safer usage, consider using [`World::try_query_from`].
    ///
    /// # Panics
    ///
    /// Panics if the entity is not alive or not a valid query.
    ///
    /// # Safety
    ///
    /// Proceed with caution. Use `.iter_only` instead.
    ///
    /// # See Also
    ///
    /// * [`World::try_query_from`]
    pub fn query_from(&self, query_entity: impl Into<Entity>) -> Query<()> {
        self.try_query_from(query_entity)
            .expect("entity / query is not alive or valid")
    }

    /// Create and iterate an uncached query.
    ///
    /// This function creates a query and immediately iterates it.
    ///
    /// # Returns
    ///
    /// The query.
    ///
    /// # Type Parameters
    ///
    /// * `Components`: The components to match on.
    ///
    /// # See also
    ///
    /// * [`QueryAPI::each()`]
    /// * [`World::each_entity()`]
    pub fn each<Components>(&self, func: impl FnMut(Components::TupleType<'_>)) -> Query<Components>
    where
        Components: QueryTuple,
    {
        let query = QueryBuilder::<Components>::new(self).build();
        query.each(func);
        query
    }

    /// Create and iterate an uncached query.
    ///
    /// This function creates a query and immediately iterates it.
    ///
    /// # Returns
    ///
    /// The query.
    ///
    /// # Type Parameters
    ///
    /// * `Components`: The components to match on.
    ///
    /// # See also
    ///
    /// * [`QueryAPI::each_entity()`]
    /// * [`World::each()`]
    pub fn each_entity<Components>(
        &self,
        func: impl FnMut(EntityView, Components::TupleType<'_>),
    ) -> Query<Components>
    where
        Components: QueryTuple,
    {
        let query = QueryBuilder::<Components>::new(self).build();
        query.each_entity(func);
        query
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
    /// * `entity` - An `EntityView` that represents a system within the world.
    pub fn system_from<'a>(&'a self, entity: EntityView<'a>) -> System<'a> {
        System::new_from_existing(entity)
    }

    /// Creates a new `SystemBuilder` instance for constructing systems.
    ///
    /// This function initializes a `SystemBuilder` which is used to create systems that match specific components.
    /// It is a generic method that works with any component types that implement the `QueryTuple` trait.
    ///
    /// # Type Parameters
    /// - `Components`: The components to match on. Must implement the `QueryTuple` trait.
    ///
    /// # See also
    ///
    /// * [`World::system_named()`]
    pub fn system<Components>(&self) -> SystemBuilder<'_, Components>
    where
        Components: QueryTuple,
    {
        SystemBuilder::<Components>::new(self)
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
    /// - `Components`: The components to match on. Must implement the `QueryTuple` trait.
    ///
    /// # See also
    ///
    /// * [`World::system()`]
    pub fn system_named<'a, Components>(&'a self, name: &str) -> SystemBuilder<'a, Components>
    where
        Components: QueryTuple,
    {
        SystemBuilder::<Components>::new_named(self, name)
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
    /// - `Components`: The components to match on. Must implement the `QueryTuple` trait.
    pub fn system_builder_from_desc<Components>(
        &self,
        desc: sys::ecs_system_desc_t,
    ) -> SystemBuilder<'_, Components>
    where
        Components: QueryTuple,
    {
        SystemBuilder::<Components>::new_from_desc(self, desc)
    }
}

/// Pipeline mixin implementation
#[cfg(feature = "flecs_pipeline")]
impl World {
    /// Create a new [`Pipeline`](crate::addons::pipeline::Pipeline).
    ///
    /// # See also
    ///
    /// * [`World::pipeline_named()`]
    /// * [`World::pipeline_type()`]
    #[inline(always)]
    pub fn pipeline(&self) -> PipelineBuilder<'_, ()> {
        PipelineBuilder::<()>::new(self)
    }

    /// Create a new named [`Pipeline`](crate::addons::pipeline::Pipeline).
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the pipeline.
    ///
    /// # See also
    ///
    /// * [`World::pipeline()`]
    /// * [`World::pipeline_type()`]
    #[inline(always)]
    pub fn pipeline_named<'a>(&'a self, name: &str) -> PipelineBuilder<'a, ()> {
        PipelineBuilder::<()>::new_named(self, name)
    }

    /// Create a new [`Pipeline`](crate::addons::pipeline::Pipeline) with the provided associated type.
    ///
    /// # Type Parameters
    ///
    /// * `Pipeline` - The associated type to use for the pipeline.
    ///
    /// # See also
    ///
    /// * [`World::pipeline()`]
    /// * [`World::pipeline_named()`]
    #[inline(always)]
    pub fn pipeline_type<Pipeline>(&self) -> PipelineBuilder<'_, ()>
    where
        Pipeline: ComponentType<Struct> + ComponentId,
    {
        PipelineBuilder::<()>::new_w_entity(self, Pipeline::entity_id(self))
    }

    /// Set a custom pipeline. This operation sets the pipeline to run when [`World::progress()`] is invoked.
    ///
    /// # Arguments
    ///
    /// * `pipeline` - The pipeline to set.
    ///
    /// # See also
    ///
    /// * [`World::get_pipeline()`]
    /// * [`World::set_pipeline()`]
    #[inline(always)]
    pub fn set_pipeline(&self, pipeline: impl IntoEntity) {
        let world = self.world();
        unsafe {
            sys::ecs_set_pipeline(self.raw_world.as_ptr(), *pipeline.into_entity(world));
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
    /// * [`World::set_pipeline()`]
    /// * [`World::set_pipeline()`]
    #[inline(always)]
    pub fn get_pipeline(&self) -> EntityView<'_> {
        EntityView::new_from(self, unsafe {
            sys::ecs_get_pipeline(self.raw_world.as_ptr())
        })
    }

    /// Progress world one tick.
    ///
    /// Progresses the world by running all enabled and periodic systems
    /// on their matching entities.
    ///
    /// This is a wrapper around [`World::progress_time()`]. It passes `0.0` as
    /// the `delta_time` to automatically measure the time passed since the last
    /// frame. This mode is useful for applications that do not manage time
    /// explicitly and want the system to measure the time automatically.
    ///
    /// # Returns
    ///
    /// True if the world has been progressed, false if [`World::quit()`] has been called.
    ///
    /// # See also
    ///
    /// * [`World::progress_time()`]
    /// * C API: `ecs_progress`
    #[inline(always)]
    pub fn progress(&self) -> bool {
        self.progress_time(0.0)
    }

    /// Progress world by delta time.
    ///
    /// Progresses the world by running all enabled and periodic systems
    /// on their matching entities for the specified time since the last frame.
    ///
    /// When `delta_time` is 0, `World::progress_time()` will automatically measure the time passed
    /// since the last frame. For applications not using time management, passing a
    /// non-zero `delta_time` (1.0 recommended) skips automatic time measurement to avoid overhead.
    ///
    /// # Arguments
    ///
    /// * `delta_time` - The time to progress the world by. Pass 0.0 for automatic time measurement.
    ///
    /// # Returns
    ///
    /// True if the world has been progressed, false if [`World::quit()`] has been called.
    ///
    /// # See also
    ///
    /// * [`World::progress()`]
    /// * C API: `ecs_progress`
    #[inline(always)]
    pub fn progress_time(&self, delta_time: f32) -> bool {
        unsafe { sys::ecs_progress(self.raw_world.as_ptr(), delta_time) }
    }

    /// Run pipeline.
    /// Runs all systems in the specified pipeline. Can be invoked from multiple
    /// threads if staging is disabled, managing staging and, if needed, thread
    /// synchronization.
    ///
    /// Providing 0 for pipeline id runs the default pipeline (builtin or set via
    /// `set_pipeline()`). Using [`World::progress()`] auto-invokes this for the
    /// default pipeline. Additional pipelines may be run explicitly.
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
    /// * [`World::run_pipeline()`]
    /// * [`World::run_pipeline_time()`]
    #[inline(always)]
    pub fn run_pipeline(&self, pipeline: impl IntoEntity) {
        Self::run_pipeline_time(self, pipeline, 0.0);
    }

    /// Run pipeline.
    /// Runs all systems in the specified pipeline. Can be invoked from multiple
    /// threads if staging is disabled, managing staging and, if needed, thread
    /// synchronization.
    ///
    /// Providing 0 for pipeline id runs the default pipeline (builtin or set via
    /// `set_pipeline()`). Using [`World::progress()`] auto-invokes this for the
    /// default pipeline. Additional pipelines may be run explicitly.
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
    /// * [`World::run_pipeline()`]
    /// * [`World::run_pipeline_time()`]
    #[inline(always)]
    pub fn run_pipeline_time(&self, pipeline: impl IntoEntity, delta_time: super::FTime) {
        let world = self.world();
        unsafe {
            sys::ecs_run_pipeline(
                self.raw_world.as_ptr(),
                *pipeline.into_entity(world),
                delta_time,
            );
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
    /// * [`World::get_time_scale()`]
    #[inline(always)]
    pub fn set_time_scale(&self, mul: super::FTime) {
        unsafe {
            sys::ecs_set_time_scale(self.raw_world.as_ptr(), mul);
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
    /// * [`World::set_time_scale()`]
    #[inline(always)]
    pub fn get_time_scale(&self) -> super::FTime {
        self.info().time_scale
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
    /// * [`World::set_target_fps()`]
    #[inline(always)]
    pub fn get_target_fps(&self) -> super::FTime {
        self.info().target_fps
    }

    /// Set target frames per second (FPS).
    ///
    /// Configures the world to run at the specified target FPS, ensuring that
    /// [`World::progress()`] is not called more frequently than this rate. This mechanism
    /// enables tracking the elapsed time since the last [`World::progress()`] call and
    /// sleeping for any remaining time in the frame, if applicable.
    ///
    /// Utilizing this feature promotes consistent system execution intervals and
    /// conserves CPU resources by avoiding more frequent system runs than necessary.
    ///
    /// It's important to note that [`World::progress()`] will only introduce sleep periods
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
    /// * [`World::get_target_fps()`]
    #[inline(always)]
    pub fn set_target_fps(&self, target_fps: super::FTime) {
        unsafe {
            sys::ecs_set_target_fps(self.raw_world.as_ptr(), target_fps);
        }
    }

    /// Reset world clock. Reset the clock that keeps track of the total time passed in the simulation.
    #[inline(always)]
    pub fn reset_clock(&self) {
        unsafe {
            sys::ecs_reset_clock(self.raw_world.as_ptr());
        }
    }

    /// Set number of worker threads.
    ///
    /// Setting this value to a value higher than 1 will start as many threads and
    /// will cause systems to evenly distribute matched entities across threads.
    /// The operation may be called multiple times to reconfigure the number of threads used,
    /// but never while running a system / pipeline. Calling [`World::set_threads()`] will also end the use
    /// of task threads setup with [`World::set_task_threads()`] and vice-versa
    ///
    /// # Arguments
    ///
    /// * `threads` - The number of threads to use.
    ///
    /// # See also
    ///
    /// * [`World::set_stage_count()`]
    /// * [`World::set_task_threads()`]
    #[inline(always)]
    pub fn set_threads(&self, threads: i32) {
        unsafe {
            sys::ecs_set_threads(self.raw_world.as_ptr(), threads);
        }
    }

    /// Get number of configured stages. Return number of stages set by [`World::set_stage_count()`].
    ///
    /// # Returns
    ///
    /// The number of stages as an integer.
    ///
    /// # See also
    ///
    /// * [`World::set_stage_count()`]
    /// * [`World::set_threads()`]
    #[inline(always)]
    pub fn get_threads(&self) -> i32 {
        unsafe { sys::ecs_get_stage_count(self.raw_world.as_ptr()) }
    }

    /// returns true if the world is currently multithreaded, such as when a system that is multithreaded is running.
    #[inline(always)]
    pub fn is_currently_multithreaded(&self) -> bool {
        unsafe {
            sys::ecs_world_get_flags(self.raw_world.as_ptr()) & sys::EcsWorldMultiThreaded != 0
        }
    }

    /// Set number of worker task threads.
    ///
    /// Configures the world to use a specified number of short-lived task threads,
    /// distinct from [`World::set_threads()`] where threads persist. Here, threads are
    /// created and joined for each world update, leveraging the `os_api_t` tasks
    /// APIs for task management instead of traditional thread APIs. This approach
    /// is advantageous for integrating with external asynchronous job systems,
    /// allowing for the dynamic creation and synchronization of tasks specific to
    /// each world update.
    ///
    /// This function can be invoked multiple times to adjust the count of task threads,
    /// but must not be called concurrently with system or pipeline execution. Switching
    /// to [`World::set_task_threads()`] from [`World::set_threads()`] (or vice versa) will
    /// terminate the use of the previously configured threading model.
    ///
    /// # Arguments
    ///
    /// * `task_threads` - The number of task threads to use.
    ///
    /// # See also
    ///
    /// * [`World::using_task_threads()`]
    #[inline(always)]
    pub fn set_task_threads(&self, task_threads: i32) {
        unsafe {
            sys::ecs_set_task_threads(self.raw_world.as_ptr(), task_threads);
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
    /// * [`World::set_task_threads()`]
    #[inline(always)]
    pub fn using_task_threads(&self) -> bool {
        unsafe { sys::ecs_using_task_threads(self.raw_world.as_ptr()) }
    }

    /// Delete empty tables within the world
    ///
    /// # See also
    ///
    /// * C API: `ecs_delete_empty_tables`
    #[inline(always)] //min_id_count: i32, time_budget_seconds: f64) -> i32
    pub fn delete_empty_tables(&self, desc: sys::ecs_delete_empty_tables_desc_t) -> i32 {
        unsafe { sys::ecs_delete_empty_tables(self.raw_world.as_ptr(), &desc) }
    }

    /// Begin exclusive thread access to the world.
    ///
    /// # Panics
    ///
    /// This operation ensures that only the thread from which this operation is
    /// called can access the world. Attempts to access the world from other threads
    /// will panic.
    ///
    /// `exclusive_access_begin()` must be called in pairs with
    /// `exclusive_access_end()`. Calling `exclusive_access_begin()` from another
    /// thread without first calling `exclusive_access_end()` will panic.
    ///
    /// This operation should only be called once per thread. Calling it multiple
    /// times for the same thread will cause a panic.
    ///
    /// # Note
    ///
    /// This feature only works in builds where asserts are enabled. The
    /// feature requires the OS API `thread_self_` callback to be set.
    ///
    /// # Arguments
    ///
    /// * `thread_name` - Name of the thread obtaining exclusive access. Use `c"thread_name"` to pass a C-style string.
    ///   Required to be a static string for safety reasons.
    pub fn exclusive_access_begin(&self, thread_name: Option<&'static CStr>) {
        let name_ptr = thread_name.map_or(core::ptr::null(), CStr::as_ptr);

        unsafe {
            sys::ecs_exclusive_access_begin(self.raw_world.as_ptr(), name_ptr);
        }
    }

    /// End exclusive thread access to the world.
    ///
    /// # Panics
    ///
    /// This operation must be called from the same thread that called
    /// `exclusive_access_begin()`. Calling it from a different thread will cause
    /// a panic.
    ///
    /// This operation should be called after `exclusive_access_begin()`. After
    /// calling this operation, other threads are no longer prevented from mutating
    /// the world.
    ///
    /// When `lock_world` is set to true, no thread will be able to mutate the world
    /// until `exclusive_access_begin()` is called again. While the world is locked,
    /// only read-only operations are allowed. For example, `get` without mutable access is allowed,
    /// but `get` with mutable access is not allowed.
    ///
    /// A locked world can be unlocked by calling `exclusive_access_end()` again with
    /// `lock_world` set to false. Note that this only works for locked worlds; if
    /// `exclusive_access_end()` is called on a world that has exclusive thread
    /// access from a different thread, a panic will occur.
    ///
    /// # Arguments
    ///
    /// * `lock_world` - When true, any mutations on the world will be blocked.
    ///
    /// # Note
    ///
    /// This feature only works in builds where asserts are enabled. The
    /// feature requires the OS API `thread_self_` callback to be set.
    pub fn exclusive_access_end(&self, lock_world: bool) {
        unsafe {
            sys::ecs_exclusive_access_end(self.raw_world.as_ptr(), lock_world);
        }
    }

    /// Return component id if it has been registered.
    ///
    /// This is similar to `component_id::<T>()` but will never register the
    /// component with the world. If `T` is not registered in this world, returns `None`.
    #[inline(always)]
    pub fn get_component_id<T: ComponentId>(&self) -> Option<Entity> {
        if <T as ComponentId>::is_registered_with_world(self) {
            Some(Entity(T::entity_id(self)))
        } else {
            None
        }
    }

    /// Return raw type info for an id (component, tag, or pair).
    ///
    /// Returns `None` when no type info is available for the provided id.
    #[inline(always)]
    pub fn type_info_from(&self, id: impl IntoId) -> Option<*const sys::ecs_type_info_t> {
        let ptr = unsafe { sys::ecs_get_type_info(self.raw_world.as_ptr(), *id.into_entity(self)) };
        if ptr.is_null() { None } else { Some(ptr) }
    }
}
