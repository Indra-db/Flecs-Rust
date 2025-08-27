use flecs_ecs::core::ecs_os_api;
use flecs_ecs::prelude::*;
use flecs_ecs::sys::{ecs_size_t, ecs_time_t};
use std::ffi::c_void;
use std::os::raw::{c_char, c_int};

#[derive(Debug, Component, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

// External declarations for libc functions from libc.a
extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    fn calloc(count: usize, size: usize) -> *mut c_void;
}

// WASM-compatible OS API implementations with correct signatures
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
    // Flecs calloc signature only takes size, not count
    calloc(1, size as usize)
}

unsafe extern "C" fn wasm_now() -> u64 {
    // Return a simple incrementing time for WASM (in microseconds)
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
    // Proper abort - this will help us identify the real issue
    panic!("Flecs abort called in WASM - there's a real problem to fix");
}

unsafe extern "C" fn wasm_log(
    _level: c_int,
    _file: *const c_char,
    _line: c_int,
    _msg: *const c_char,
) {
    // No-op for WASM - logging can be handled elsewhere
}

// Set up the WASM-compatible OS API using the hook system
fn setup_wasm_os_api() {
    ecs_os_api::add_init_hook(Box::new(|api| {
        // Set memory management functions
        api.malloc_ = Some(wasm_malloc);
        api.free_ = Some(wasm_free);
        api.realloc_ = Some(wasm_realloc);
        api.calloc_ = Some(wasm_calloc);

        // Set time functions
        api.now_ = Some(wasm_now);
        api.get_time_ = Some(wasm_get_time);

        // Set abort function
        api.abort_ = Some(wasm_abort);

        // Set log function to no-op
        api.log_ = Some(wasm_log);
    }));
}

#[no_mangle]
pub extern "C" fn example_pos_x() -> i32 {
    // Set up WASM OS API using the proper hook system
    setup_wasm_os_api();

    // Now create the world - this will trigger the OS API initialization
    let world = World::new();

    world.component::<Position>();

    let _e = world.entity().set(Position { x: 10, y: 20 });

    // Create a system that modifies position
    world.system::<&mut Position>().each(|pos| {
        pos.x += 1;
    });

    // The issue is here - progress_time() is calling abort
    world.progress_time(0.016666);

    // Return the updated x coordinate - should be 11 if progress worked
    let pos = _e.cloned::<&Position>();
    pos.x
}
