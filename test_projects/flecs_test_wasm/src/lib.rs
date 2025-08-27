use flecs_ecs::core::ecs_os_api;
use flecs_ecs::prelude::*;
use flecs_ecs::sys::{
    ecs_os_cond_t, ecs_os_mutex_t, ecs_os_thread_callback_t, ecs_os_thread_id_t, ecs_os_thread_t,
    ecs_size_t, ecs_time_t,
};
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

// Threading and synchronization stubs for WASM (single-threaded environment)
unsafe extern "C" fn wasm_thread_new(
    _callback: ecs_os_thread_callback_t,
    _param: *mut c_void,
) -> ecs_os_thread_t {
    0 as ecs_os_thread_t // Return null/zero thread handle
}

unsafe extern "C" fn wasm_thread_join(_thread: ecs_os_thread_t) -> *mut c_void {
    std::ptr::null_mut() // Return null result
}

unsafe extern "C" fn wasm_thread_self() -> ecs_os_thread_id_t {
    1 // Always return thread ID 1 in single-threaded WASM
}

unsafe extern "C" fn wasm_mutex_new() -> ecs_os_mutex_t {
    1 as ecs_os_mutex_t // Return a fake mutex handle
}

unsafe extern "C" fn wasm_mutex_free(_mutex: ecs_os_mutex_t) {
    // No-op in single-threaded WASM
}

unsafe extern "C" fn wasm_mutex_lock(_mutex: ecs_os_mutex_t) {
    // No-op in single-threaded WASM
}

unsafe extern "C" fn wasm_mutex_unlock(_mutex: ecs_os_mutex_t) {
    // No-op in single-threaded WASM
}

unsafe extern "C" fn wasm_cond_new() -> ecs_os_cond_t {
    1 as ecs_os_cond_t // Return a fake condition variable handle
}

unsafe extern "C" fn wasm_cond_free(_cond: ecs_os_cond_t) {
    // No-op in single-threaded WASM
}

unsafe extern "C" fn wasm_cond_signal(_cond: ecs_os_cond_t) {
    // No-op in single-threaded WASM
}

unsafe extern "C" fn wasm_cond_broadcast(_cond: ecs_os_cond_t) {
    // No-op in single-threaded WASM
}

unsafe extern "C" fn wasm_cond_wait(_cond: ecs_os_cond_t, _mutex: ecs_os_mutex_t) {
    // No-op in single-threaded WASM
}

unsafe extern "C" fn wasm_ainc(value: *mut i32) -> i32 {
    // Atomic increment - in single-threaded WASM, just regular increment
    if !value.is_null() {
        *value += 1;
        *value
    } else {
        0
    }
}

unsafe extern "C" fn wasm_adec(value: *mut i32) -> i32 {
    // Atomic decrement - in single-threaded WASM, just regular decrement
    if !value.is_null() {
        *value -= 1;
        *value
    } else {
        0
    }
}

unsafe extern "C" fn wasm_lainc(value: *mut i64) -> i64 {
    // Atomic increment (64-bit) - in single-threaded WASM, just regular increment
    if !value.is_null() {
        *value += 1;
        *value
    } else {
        0
    }
}

unsafe extern "C" fn wasm_ladec(value: *mut i64) -> i64 {
    // Atomic decrement (64-bit) - in single-threaded WASM, just regular decrement
    if !value.is_null() {
        *value -= 1;
        *value
    } else {
        0
    }
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

        // Set threading functions (no-ops for WASM)
        api.thread_new_ = Some(wasm_thread_new);
        api.thread_join_ = Some(wasm_thread_join);
        api.thread_self_ = Some(wasm_thread_self);
        api.task_new_ = Some(wasm_thread_new); // Same as thread_new
        api.task_join_ = Some(wasm_thread_join); // Same as thread_join

        // Set mutex functions (no-ops for WASM)
        api.mutex_new_ = Some(wasm_mutex_new);
        api.mutex_free_ = Some(wasm_mutex_free);
        api.mutex_lock_ = Some(wasm_mutex_lock);
        api.mutex_unlock_ = Some(wasm_mutex_unlock);

        // Set condition variable functions (no-ops for WASM)
        api.cond_new_ = Some(wasm_cond_new);
        api.cond_free_ = Some(wasm_cond_free);
        api.cond_signal_ = Some(wasm_cond_signal);
        api.cond_broadcast_ = Some(wasm_cond_broadcast);
        api.cond_wait_ = Some(wasm_cond_wait);

        // Set atomic functions
        api.ainc_ = Some(wasm_ainc);
        api.adec_ = Some(wasm_adec);
        api.lainc_ = Some(wasm_lainc);
        api.ladec_ = Some(wasm_ladec);

        // Set abort function
        api.abort_ = Some(wasm_abort);

        // Set log function to no-op
        api.log_ = Some(wasm_log);
    }));
}
#[no_mangle]
pub extern "C" fn example_pos_x() -> i32 {
    // Set up WASM-compatible OS API hooks before creating world
    setup_wasm_os_api();

    // Create world and basic entities
    let world = World::new();

    // Define component and create entity
    let entity = world.entity().set(Position { x: 10, y: 20 });

    // Create a simple system
    let system = world.system::<&mut Position>().each(|pos| {
        pos.x += 1;
    });

    // Use manual system.run() which we know works
    system.run();

    // Get the x value to verify the system ran
    entity.get::<&Position>(|pos| pos.x)
}