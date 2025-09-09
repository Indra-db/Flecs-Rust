// Simplified single-project approach for wasm-bindgen + wasi-libc
// This file demonstrates conditional compilation for the two-stage build

use wasm_bindgen::prelude::*;

// Import the `console.log` function from the browser's console API
//#[wasm_bindgen]
extern "C" {
    //#[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro for easier console logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Simple functions that work regardless of Flecs availability
//#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

//#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

// Conditional Flecs functionality
#[cfg(feature = "flecs_ecs")]
mod flecs_impl {
    use super::*;
    use flecs_ecs::prelude::*;

    #[derive(Component, Debug)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Component, Debug)]
    struct Velocity {
        x: f32,
        y: f32,
    }

    //#[wasm_bindgen]
    pub fn run_flecs_bindgen_test() {
        console_log!("Starting Flecs wasm-bindgen test with full runtime...");

        // Set up the WASM OS API (when available)
        #[cfg(not(feature = "bindgen_only"))]
        crate::setup_wasm_os_api();

        // Create a world
        let world = World::new();
        console_log!("World created");

        // Create an entity with Position and Velocity components
        let entity = world
            .entity()
            .set(Position { x: 0.0, y: 0.0 })
            .set(Velocity { x: 1.0, y: 0.5 });

        console_log!("Entity created with Position and Velocity");

        // Get the initial position
        entity.try_get::<&Position>(|pos| {
            console_log!("Initial position: ({}, {})", pos.x, pos.y);
        });

        // Create a simple movement system
        world
            .system::<(&mut Position, &Velocity)>()
            .each(|(pos, vel)| {
                pos.x += vel.x;
                pos.y += vel.y;
            });

        console_log!("Movement system created");

        // Run a few iterations
        for i in 0..5 {
            world.progress(0.016); // 60 FPS

            entity.try_get::<&Position>(|pos| {
                console_log!("Frame {}: position: ({:.2}, {:.2})", i + 1, pos.x, pos.y);
            });
        }

        console_log!("Flecs wasm-bindgen test completed successfully!");
    }
}

// Stub implementation for bindgen_only builds
#[cfg(not(feature = "flecs_ecs"))]
mod flecs_stub {
    use super::*;

    //#[wasm_bindgen]
    pub fn run_flecs_bindgen_test() {
        console_log!("Flecs test (stub mode - no ECS functionality)");
        console_log!("This is running in bindgen_only mode for binding generation");
        console_log!("The actual runtime will use the full Flecs implementation");
    }
}

// Re-export the appropriate implementation
#[cfg(feature = "flecs_ecs")]
pub use flecs_impl::*;

#[cfg(not(feature = "flecs_ecs"))]
pub use flecs_stub::*;

// Include the full WASM OS API setup only when Flecs is available
#[cfg(all(feature = "flecs_ecs", not(feature = "bindgen_only")))]
mod wasm_os_api {
    use flecs_ecs::core::ecs_os_api;
    use flecs_ecs::sys::{
        ecs_os_cond_t, ecs_os_mutex_t, ecs_os_thread_callback_t, ecs_os_thread_id_t,
        ecs_os_thread_t, ecs_size_t, ecs_time_t,
    };
    use std::ffi::c_void;
    use std::os::raw::{c_char, c_int};

    // External declarations for libc functions from wasi-libc
    extern "C" {
        fn malloc(size: usize) -> *mut c_void;
        fn free(ptr: *mut c_void);
        fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
        fn calloc(count: usize, size: usize) -> *mut c_void;
    }

    // WASM-compatible OS API implementations
    unsafe extern "C" fn wasm_malloc(size: ecs_size_t) -> *mut c_void {
        malloc(size as usize)
    }

    unsafe extern "C" fn wasm_free(ptr: *mut c_void) {
        free(ptr)
    }

    unsafe extern "C" fn wasm_realloc(ptr: *mut c_void, size: ecs_size_t) -> *mut c_void {
        realloc(ptr, size as usize)
    }

    unsafe extern "C" fn wasm_calloc(size: ecs_size_t) -> *mut c_void {
        calloc(1, size as usize)
    }

    unsafe extern "C" fn wasm_now() -> u64 {
        static mut TIME: u64 = 0;
        TIME += 16666; // ~60 FPS in microseconds
        TIME
    }

    unsafe extern "C" fn wasm_get_time(time_out: *mut ecs_time_t) {
        if !time_out.is_null() {
            let now_us = wasm_now();
            let sec = now_us / 1_000_000;
            let nanosec = (now_us % 1_000_000) * 1000;

            (*time_out).sec = sec as u32;
            (*time_out).nanosec = nanosec as u32;
        }
    }

    unsafe extern "C" fn wasm_abort() {
        panic!("Flecs internal abort");
    }

    unsafe extern "C" fn wasm_log(
        _level: c_int,
        _file: *const c_char,
        _line: c_int,
        _msg: *const c_char,
    ) {
        // No-op for simplicity
    }

    unsafe extern "C" fn wasm_sleep(_sec: i32, _nanosec: i32) {
        // No-op for WASM
    }

    // Threading stubs (WASM is single-threaded)
    unsafe extern "C" fn wasm_thread_new(
        _callback: ecs_os_thread_callback_t,
        _param: *mut c_void,
    ) -> ecs_os_thread_t {
        0 as ecs_os_thread_t
    }

    unsafe extern "C" fn wasm_thread_join(
        _thread: ecs_os_thread_t,
        _return_value: *mut *mut c_void,
    ) -> *mut c_void {
        std::ptr::null_mut()
    }

    unsafe extern "C" fn wasm_thread_self() -> ecs_os_thread_id_t {
        0
    }

    unsafe extern "C" fn wasm_task_new(
        _callback: ecs_os_thread_callback_t,
        _param: *mut c_void,
    ) -> ecs_os_thread_t {
        0 as ecs_os_thread_t
    }

    unsafe extern "C" fn wasm_task_join(_thread: ecs_os_thread_t) -> *mut c_void {
        std::ptr::null_mut()
    }

    unsafe extern "C" fn wasm_ainc(_value: *mut i32) -> i32 {
        0
    }

    unsafe extern "C" fn wasm_adec(_value: *mut i32) -> i32 {
        0
    }

    unsafe extern "C" fn wasm_lainc(_value: *mut i64) -> i64 {
        0
    }

    unsafe extern "C" fn wasm_ladec(_value: *mut i64) -> i64 {
        0
    }

    unsafe extern "C" fn wasm_mutex_new() -> ecs_os_mutex_t {
        0 as ecs_os_mutex_t
    }

    unsafe extern "C" fn wasm_mutex_free(_mutex: ecs_os_mutex_t) {}

    unsafe extern "C" fn wasm_mutex_lock(_mutex: ecs_os_mutex_t) {}

    unsafe extern "C" fn wasm_mutex_unlock(_mutex: ecs_os_mutex_t) {}

    unsafe extern "C" fn wasm_cond_new() -> ecs_os_cond_t {
        0 as ecs_os_cond_t
    }

    unsafe extern "C" fn wasm_cond_free(_cond: ecs_os_cond_t) {}

    unsafe extern "C" fn wasm_cond_signal(_cond: ecs_os_cond_t) {}

    unsafe extern "C" fn wasm_cond_broadcast(_cond: ecs_os_cond_t) {}

    unsafe extern "C" fn wasm_cond_wait(_cond: ecs_os_cond_t, _mutex: ecs_os_mutex_t) {}

    pub fn setup_wasm_os_api() {
        ecs_os_api::set_api_defaults();

        let api = ecs_os_api::get_api();

        api.malloc_ = Some(wasm_malloc);
        api.free_ = Some(wasm_free);
        api.realloc_ = Some(wasm_realloc);
        api.calloc_ = Some(wasm_calloc);

        api.get_time_ = Some(wasm_get_time);
        api.sleep_ = Some(wasm_sleep);
        api.now_ = Some(wasm_now);
        api.log_ = Some(wasm_log);
        api.abort_ = Some(wasm_abort);

        // Threading API
        api.thread_new_ = Some(wasm_thread_new);
        api.thread_join_ = Some(wasm_thread_join);
        api.thread_self_ = Some(wasm_thread_self);

        api.task_new_ = Some(wasm_task_new);
        api.task_join_ = Some(wasm_task_join);

        // Atomic operations
        api.ainc_ = Some(wasm_ainc);
        api.adec_ = Some(wasm_adec);
        api.lainc_ = Some(wasm_lainc);
        api.ladec_ = Some(wasm_ladec);

        // Synchronization primitives
        api.mutex_new_ = Some(wasm_mutex_new);
        api.mutex_free_ = Some(wasm_mutex_free);
        api.mutex_lock_ = Some(wasm_mutex_lock);
        api.mutex_unlock_ = Some(wasm_mutex_unlock);

        api.cond_new_ = Some(wasm_cond_new);
        api.cond_free_ = Some(wasm_cond_free);
        api.cond_signal_ = Some(wasm_cond_signal);
        api.cond_broadcast_ = Some(wasm_cond_broadcast);
        api.cond_wait_ = Some(wasm_cond_wait);
    }
}

#[cfg(all(feature = "flecs_ecs", not(feature = "bindgen_only")))]
pub use wasm_os_api::setup_wasm_os_api;

#[cfg(not(all(feature = "flecs_ecs", not(feature = "bindgen_only"))))]
pub fn setup_wasm_os_api() {
    // No-op when Flecs is not available
}
