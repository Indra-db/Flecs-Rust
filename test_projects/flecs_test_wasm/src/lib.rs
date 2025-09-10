use flecs_ecs::core::ecs_os_api;
use flecs_ecs::prelude::*;
use flecs_ecs::sys::*;
use std::ffi::c_void;
use std::os::raw::{c_char, c_int};

use wasm_bindgen::prelude::*;

// Function that calls C's strlen
extern "C" fn calls_strlen(s: *const c_char) -> usize {
    unsafe { strlen(s) as usize }
}

// Proper Rust function that calls calls_strlen - exported directly for WASM
#[unsafe(no_mangle)]
pub extern "C" fn get_string_length(s: *const c_char) -> usize {
    calls_strlen(s)
}

// Test function that creates a test string and calls get_string_length
#[wasm_bindgen]
pub fn test_string_length() -> usize {
    let test_str = b"Hello, WASM!\0";
    get_string_length(test_str.as_ptr() as *const c_char)
}

// Wasm-bindgen exports for Flecs functions
#[wasm_bindgen]
pub fn wasm_create_world() -> u32 {
    create_world() as u32
}

#[wasm_bindgen]
pub fn wasm_progress_world(world_ptr: u32) {
    progress_world_ptr(world_ptr as *mut WorldState);
}

#[wasm_bindgen]
pub fn wasm_get_pos_x(world_ptr: u32) -> i32 {
    get_pos_x(world_ptr as *mut WorldState)
}

#[wasm_bindgen]
pub fn wasm_destroy_world(world_ptr: u32) {
    destroy_world(world_ptr as *mut WorldState);
}

#[derive(Debug, Component, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

// External declarations for libc functions from libc.a
// extern "C" {
//     fn malloc(size: usize) -> *mut c_void;
//     fn free(ptr: *mut c_void);
//     fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
//     fn calloc(count: usize, size: usize) -> *mut c_void;
// }

// External declarations for JavaScript console functions (for debugging)
#[cfg(target_arch = "wasm32")]
extern "C" {
    fn console_log(ptr: *const u8, len: usize);
    fn console_error(ptr: *const u8, len: usize);
    fn debug_trace(value: i32);
}

// Helper functions for debugging
#[cfg(target_arch = "wasm32")]
fn js_log(msg: &str) {
    unsafe {
        console_log(msg.as_ptr(), msg.len());
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn js_log(msg: &str) {
    println!("[JS_LOG] {}", msg);
}

#[cfg(target_arch = "wasm32")]
fn js_error(msg: &str) {
    unsafe {
        console_error(msg.as_ptr(), msg.len());
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn js_error(msg: &str) {
    eprintln!("[JS_ERROR] {}", msg);
}

#[cfg(target_arch = "wasm32")]
fn js_trace(value: i32) {
    unsafe {
        debug_trace(value);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn js_trace(value: i32) {
    println!("[JS_TRACE] {}", value);
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
    // Log detailed abort information to JavaScript console
    js_error("WASM ABORT TRIGGERED!");
    js_error("This abort was called from within Flecs C code");
    js_error("Most likely cause: frame/timing system incompatibility with WASM");

    // Call the panic with a more specific message
    panic!("Flecs internal abort - check JavaScript console for details");
}

unsafe extern "C" fn wasm_log(
    _level: c_int,
    _file: *const c_char,
    _line: c_int,
    _msg: *const c_char,
) {
    // // Convert the C string to a Rust string and log it
    // if !_msg.is_null() {
    //     let c_str = std::ffi::CStr::from_ptr(_msg);
    //     if let Ok(rust_str) = c_str.to_str() {
    //         js_log(&format!("[FLECS] {}", rust_str));
    //     }
    // }
}

unsafe extern "C" fn wasm_sleep(sec: i32, nanosec: i32) {
    // No-op for WASM - we can't actually sleep in single-threaded WASM
    // But we need to provide this function for ecs_os_has_time() to return true
    let total_ms = (sec as f64) * 1000.0 + (nanosec as f64) / 1_000_000.0;
    js_log(&format!(
        "WASM sleep called with {}s, {}ns (total: {:.2}ms, no-op)",
        sec, nanosec, total_ms
    ));
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
    use std::sync::atomic::{AtomicBool, Ordering};
    static SETUP_DONE: AtomicBool = AtomicBool::new(false);

    // Only set up once to avoid multiple hook registrations
    if SETUP_DONE
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
        .is_ok()
    {
        ecs_os_api::add_init_hook(Box::new(|api| {
            // Set memory management functions
            api.malloc_ = Some(wasm_malloc);
            api.free_ = Some(wasm_free);
            api.realloc_ = Some(wasm_realloc);
            api.calloc_ = Some(wasm_calloc);

            // Set time functions
            api.now_ = Some(wasm_now);
            api.get_time_ = Some(wasm_get_time);
            api.sleep_ = Some(wasm_sleep); // Set threading functions (no-ops for WASM)
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
}

#[no_mangle]
pub extern "C" fn test_progress() -> i32 {
    setup_wasm_os_api();

    let world = World::new();

    let e = world.entity().set(Position { x: 10, y: 10 });

    world.system::<&mut Position>().each(|pos| {
        pos.x += 1;
        pos.y += 1;
    });

    world.progress();

    let pos = e.cloned::<&Position>();
    pos.x
}

// Create a new world and return its pointer along with the entity ID
#[no_mangle]
pub extern "C" fn create_world() -> *mut WorldState {
    setup_wasm_os_api();

    let world = World::new();
    let entity = world.entity().set(Position { x: 10, y: 10 });
    let entity_id = entity.id();

    // Set up the system
    world.system::<&mut Position>().each(|pos| {
        pos.x += 1;
        pos.y += 1;
    });

    let world_state = WorldState {
        world,
        entity: entity_id,
    };

    let boxed = Box::new(world_state);
    Box::into_raw(boxed)
}

// Progress the world and return the current x position
#[no_mangle]
pub extern "C" fn progress_world_ptr(world_ptr: *mut WorldState) {
    unsafe {
        let world_state = &mut *world_ptr;
        world_state.world.progress();
    }
}

// Get current position without progressing
#[no_mangle]
pub extern "C" fn get_pos_x(world_ptr: *mut WorldState) -> i32 {
    unsafe {
        let world_state = &*world_ptr;
        let entity = world_state.world.entity_from_id(world_state.entity);
        let pos = entity.cloned::<&Position>();
        pos.x
    }
}

// Destroy the world and free memory
#[no_mangle]
pub extern "C" fn destroy_world(world_ptr: *mut WorldState) {
    if !world_ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(world_ptr);
        }
    }
}

// Helper struct to hold world and entity together
#[repr(C)]
pub struct WorldState {
    world: World,
    entity: Entity,
}
