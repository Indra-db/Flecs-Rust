//! addon for running the main application loop.

use core::ffi::c_void;

use crate::core::*;
use crate::sys;

/// Application interface.
///
/// These are typically constructed via [`World::app()`]
pub struct App<'a> {
    world: WorldRef<'a>,
    desc: sys::ecs_app_desc_t,
}

impl<'a> App<'a> {
    /// Create a new application.
    ///
    /// # Arguments
    ///
    /// * `world` - The world to run the application on.
    ///
    /// # See also
    ///
    /// * [`World::app()`]
    pub(crate) fn new(world: impl WorldProvider<'a>) -> Self {
        let mut obj = Self {
            world: world.world(),
            desc: sys::ecs_app_desc_t::default(),
        };

        let stats = unsafe { sys::ecs_get_world_info(obj.world.ptr_mut()) };
        obj.desc.target_fps = unsafe { (*stats).target_fps };
        let zero: FTime = 0.0;
        if obj.desc.target_fps.to_bits() == zero.to_bits() {
            obj.desc.target_fps = 60.0;
        }
        obj
    }

    /// Set the target frames per second.
    ///
    /// # Arguments
    ///
    /// * `fps` - The target frames per second.
    pub fn set_target_fps(&mut self, fps: FTime) -> &mut Self {
        self.desc.target_fps = fps;
        self
    }

    /// Set the time delta.
    ///
    /// # Arguments
    ///
    /// * `delta_time` - The time delta.
    pub fn set_delta_time(&mut self, delta_time: FTime) -> &mut Self {
        self.desc.delta_time = delta_time;
        self
    }

    /// Set the number of threads.
    ///
    /// # Arguments
    ///
    /// * `threads` - The number of threads.
    pub fn set_threads(&mut self, threads: i32) -> &mut Self {
        self.desc.threads = threads;
        self
    }

    /// Set the number of frames.
    ///
    /// # Arguments
    ///
    /// * `frames` - The number of frames.
    pub fn set_frames(&mut self, frames: i32) -> &mut Self {
        self.desc.frames = frames;
        self
    }

    /// Enable the REST API.
    ///
    /// # Arguments
    ///
    /// * `port` - The port to listen on.
    #[cfg(feature = "flecs_rest")]
    pub fn enable_rest(&mut self, port: u16) -> &mut Self {
        self.desc.enable_rest = true;
        self.desc.port = port;
        self
    }

    /// Enable the stats.
    ///
    /// # Arguments
    ///
    /// * `enable` - Whether to enable the stats.
    ///
    /// # See also
    ///
    /// * [`addons::stats`](crate::addons::stats)
    #[cfg(feature = "flecs_stats")]
    pub fn enable_stats(&mut self, enable: bool) -> &mut Self {
        self.desc.enable_stats = enable;
        self
    }

    // TODO change this to FnMut(&mut World) -> cint
    /// Set the application init action.
    ///
    /// # Arguments
    ///
    /// * `value` - The init action.
    pub fn init(&mut self, value: sys::ecs_app_init_action_t) -> &mut Self {
        self.desc.init = value;
        self
    }

    /// Set the application context.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context.
    pub fn context(&mut self, ctx: *mut c_void) -> &mut Self {
        self.desc.ctx = ctx;
        self
    }

    /// Run application. This will run the application with the parameters specified in desc.
    /// After the application quits ([`World::quit()`] is called) this will return.
    /// If a custom run action is set, it will be invoked by this operation.
    /// The default run action calls the frame action in a loop until it returns a non-zero value.
    ///
    /// # Returns
    ///
    /// The exit code of the application.
    pub fn run(&mut self) -> i32 {
        let world_ptr = self.world.ptr_mut();
        let result = unsafe { sys::ecs_app_run(world_ptr, &mut self.desc) };
        unsafe {
            if sys::ecs_should_quit(world_ptr) {
                // Only free world if quit flag is set. This ensures that we won't
                // try to cleanup the world if the app is used in an environment
                // that takes over the main loop, like with emscripten.
                if sys::flecs_poly_release_(world_ptr as *mut c_void) == 0 {
                    sys::ecs_fini(world_ptr);
                }
            }
        }
        result
    }
}

/// App mixin implementation
impl World {
    /// Create a new app.
    /// The app builder is a convenience wrapper around a loop that runs
    /// [`World::progress()`]. An app allows for writing platform agnostic code,
    /// as it provides hooks to modules for overtaking the main loop which is
    /// required for frameworks like emscripten.
    ///
    /// # See also
    ///
    /// * [`addons::app`](crate::addons::app)
    #[inline(always)]
    pub fn app(&self) -> App {
        App::new(self)
    }
}

impl<'a> WorldProvider<'a> for App<'a> {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}
