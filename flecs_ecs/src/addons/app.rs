use std::ffi::c_void;

use crate::core::{
    c_binding::bindings::{
        ecs_app_desc_t, ecs_app_init_action_t, ecs_app_run, ecs_fini, ecs_get_world_info,
        ecs_should_quit,
    },
    c_types::WorldT,
    world::World,
};

pub struct App {
    world: *mut WorldT,
    desc: ecs_app_desc_t,
}

impl App {
    pub fn new(world: &World) -> Self {
        let mut obj = Self {
            world: world.raw_world,
            desc: ecs_app_desc_t::default(),
        };

        let stats = unsafe { ecs_get_world_info(world.raw_world) };
        obj.desc.target_fps = unsafe { (*stats).target_fps };
        if obj.desc.target_fps == 0.0 {
            obj.desc.target_fps = 60.0;
        }
        obj
    }

    pub fn target_fps(&mut self, fps: f32) -> &mut Self {
        self.desc.target_fps = fps;
        self
    }

    pub fn delta_time(&mut self, delta_time: f32) -> &mut Self {
        self.desc.delta_time = delta_time;
        self
    }

    pub fn threads(&mut self, threads: i32) -> &mut Self {
        self.desc.threads = threads;
        self
    }

    pub fn frames(&mut self, frames: i32) -> &mut Self {
        self.desc.frames = frames;
        self
    }

    pub fn enable_rest(&mut self, port: u16) -> &mut Self {
        self.desc.enable_rest = true;
        self.desc.port = port;
        self
    }

    pub fn enable_monitor(&mut self, enable: bool) -> &mut Self {
        self.desc.enable_monitor = enable;
        self
    }

    // TODO change this to FnMut(&mut World) -> cint
    pub fn init(&mut self, value: ecs_app_init_action_t) -> &mut Self {
        self.desc.init = value;
        self
    }

    pub fn context(&mut self, ctx: *mut c_void) -> &mut Self {
        self.desc.ctx = ctx;
        self
    }

    pub fn run(&mut self) -> i32 {
        unsafe {
            let result = ecs_app_run(self.world, &mut self.desc);
            if ecs_should_quit(self.world) {
                // Only free world if quit flag is set. This ensures that we won't
                // try to cleanup the world if the app is used in an environment
                // that takes over the main loop, like with emscripten.
                ecs_fini(self.world);
            }
            result
        }
    }
}
