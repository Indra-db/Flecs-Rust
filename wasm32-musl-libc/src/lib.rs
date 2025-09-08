//! Musl libc Top Half FFI Bindings
//!
//! This crate provides Rust FFI bindings for the **top half** of musl libc. The "top half" refers to the musl libc
//! source code and headers that provide the C standard library interface, excluding the bottom
//! half which contains platform-specific system call implementations.
//!
//! ## Purpose
//!
//! This crate solves the compatibility issue between Rust programs using C libraries and the
//! wasm32-unknown-unknown target by providing:
//!
//! 1. **Rust FFI bindings** - Generated via bindgen from musl headers, this can be used as the "wasm-shim headers"
//! 2. **Compiled libc.a** - Top half musl implementation for WASM. This will link the actual implementation.
//!
//! ## Musl Top Half vs Bottom Half
//!
//! - **Top Half** (this crate): Standard C library functions (malloc, memcpy, string.h, etc.)
//! - **Bottom Half**: Platform-specific system calls.
//!
//! ## Features
//!
//! - `link-libc`: Link with pre-compiled libc.a (for end users)
//! - `bindgen`: Regenerate Rust bindings from C headers (for maintainers)
//! - `build-libc`: Compile libc.a from musl source code (for maintainers)

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(improper_ctypes)]

//skips whole file from cargo fmt --all --check for CI
#[rustfmt::skip]
mod bindings;

pub use bindings::*;

// Memory allocation implementation using Rust's allocator
// This provides malloc/free compatibility for WASM targets

use core::alloc::Layout;
use core::ptr;

// Get access to the global allocator
extern crate alloc;
use alloc::alloc::{alloc, dealloc, realloc as rust_realloc};

// Internal structure to store allocation metadata
#[repr(C)]
struct AllocHeader {
    size: usize,
    magic: u32, // Magic number for corruption detection
}

const ALLOC_MAGIC: u32 = 0xDEADBEEF;
const HEADER_SIZE: usize = core::mem::size_of::<AllocHeader>();
const ALIGNMENT: usize = 16; // on wasm32-unknown-unknown malloc uses 16-byte alignment

/// C-compatible malloc implementation using Rust's allocator
///
/// # Safety
/// This function is safe to call from C code, but the returned pointer
/// must be freed with the corresponding free() function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: usize) -> *mut core::ffi::c_void {
    if size == 0 {
        return ptr::null_mut();
    }

    // Calculate total size including header
    let total_size = HEADER_SIZE + size;

    // Create layout for allocation
    let layout = match Layout::from_size_align(total_size, ALIGNMENT) {
        Ok(layout) => layout,
        Err(_) => return ptr::null_mut(),
    };

    // Allocate memory using Rust's allocator
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        return ptr::null_mut();
    }

    // Write the header
    let header = ptr as *mut AllocHeader;
    unsafe {
        (*header).size = size;
        (*header).magic = ALLOC_MAGIC;
    }

    // Return pointer after the header
    unsafe { (ptr as *mut u8).add(HEADER_SIZE) as *mut core::ffi::c_void }
}

/// C-compatible free implementation using Rust's allocator
///
/// # Safety
/// The pointer must have been allocated with malloc() and not already freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(ptr: *mut core::ffi::c_void) {
    if ptr.is_null() {
        return;
    }

    // Get the header
    let header_ptr = unsafe { (ptr as *mut u8).sub(HEADER_SIZE) as *mut AllocHeader };
    let header = unsafe { &*header_ptr };

    // Verify magic number
    if header.magic != ALLOC_MAGIC {
        // Memory corruption detected - this would be undefined behavior in C
        // In a production system, you might want to panic or log this
        return;
    }

    // Calculate total size and create layout for deallocation
    let total_size = HEADER_SIZE + header.size;
    let layout = match Layout::from_size_align(total_size, ALIGNMENT) {
        Ok(layout) => layout,
        Err(_) => return, // Can't deallocate if we can't create the layout
    };

    unsafe { dealloc(header_ptr as *mut u8, layout) };
}

/// C-compatible calloc implementation
///
/// # Safety
/// This function is safe to call from C code, but the returned pointer
/// must be freed with the corresponding free() function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn calloc(num: usize, size: usize) -> *mut core::ffi::c_void {
    // Check for overflow
    let total_size = match num.checked_mul(size) {
        Some(s) => s,
        None => return ptr::null_mut(),
    };

    let ptr = unsafe { malloc(total_size) };
    if !ptr.is_null() {
        // Zero the memory
        unsafe { ptr::write_bytes(ptr as *mut u8, 0, total_size) };
    }
    ptr
}

/// C-compatible realloc implementation
///
/// # Safety
/// The old pointer must have been allocated with malloc/calloc/realloc and not already freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn realloc(
    ptr: *mut core::ffi::c_void,
    new_size: usize,
) -> *mut core::ffi::c_void {
    if ptr.is_null() {
        return unsafe { malloc(new_size) };
    }

    if new_size == 0 {
        unsafe { free(ptr) };
        return ptr::null_mut();
    }

    // Get the old header
    let old_header_ptr = unsafe { (ptr as *mut u8).sub(HEADER_SIZE) as *mut AllocHeader };
    let old_header = unsafe { &*old_header_ptr };

    // Verify magic number
    if old_header.magic != ALLOC_MAGIC {
        // Memory corruption detected
        return ptr::null_mut();
    }

    let old_size = old_header.size;
    let old_total_size = HEADER_SIZE + old_size;
    let new_total_size = HEADER_SIZE + new_size;

    // Create layouts
    let old_layout = match Layout::from_size_align(old_total_size, ALIGNMENT) {
        Ok(layout) => layout,
        Err(_) => return ptr::null_mut(),
    };

    let new_layout = match Layout::from_size_align(new_total_size, ALIGNMENT) {
        Ok(layout) => layout,
        Err(_) => return ptr::null_mut(),
    };

    // Try to realloc using Rust's allocator
    let new_ptr = unsafe { rust_realloc(old_header_ptr as *mut u8, old_layout, new_layout.size()) };

    if new_ptr.is_null() {
        return ptr::null_mut();
    }

    // Update the header with new size
    let new_header = new_ptr as *mut AllocHeader;
    unsafe {
        (*new_header).size = new_size;
        (*new_header).magic = ALLOC_MAGIC;
    }

    // Return pointer after the header
    unsafe { (new_ptr as *mut u8).add(HEADER_SIZE) as *mut core::ffi::c_void }
}

/// Get the usable size of an allocated block
///
/// # Safety
/// The pointer must have been allocated with malloc/calloc/realloc and not already freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc_usable_size(ptr: *mut core::ffi::c_void) -> usize {
    if ptr.is_null() {
        return 0;
    }

    // Get the header
    let header_ptr = unsafe { (ptr as *mut u8).sub(HEADER_SIZE) as *mut AllocHeader };
    let header = unsafe { &*header_ptr };

    // Verify magic number
    if header.magic != ALLOC_MAGIC {
        return 0;
    }

    header.size
}

// Additional memory functions that are commonly needed

/// C-compatible aligned allocation
///
/// # Safety
/// This function is safe to call from C code, but the returned pointer
/// must be freed with the corresponding free() function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aligned_alloc(alignment: usize, size: usize) -> *mut core::ffi::c_void {
    if size == 0 {
        return ptr::null_mut();
    }

    // Calculate total size including header
    let total_size = HEADER_SIZE + size;

    // Use the requested alignment, but ensure it's at least our minimum
    let final_alignment = alignment.max(ALIGNMENT);

    // Create layout for allocation
    let layout = match Layout::from_size_align(total_size, final_alignment) {
        Ok(layout) => layout,
        Err(_) => return ptr::null_mut(),
    };

    // Allocate memory using Rust's allocator
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        return ptr::null_mut();
    }

    // Write the header
    let header = ptr as *mut AllocHeader;
    unsafe {
        (*header).size = size;
        (*header).magic = ALLOC_MAGIC;
    }

    // Return pointer after the header
    unsafe { (ptr as *mut u8).add(HEADER_SIZE) as *mut core::ffi::c_void }
}

/// POSIX-compatible posix_memalign
///
/// # Safety
/// This function is safe to call from C code, but the returned pointer
/// must be freed with the corresponding free() function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_memalign(
    memptr: *mut *mut core::ffi::c_void,
    alignment: usize,
    size: usize,
) -> core::ffi::c_int {
    if memptr.is_null() {
        return 22; // EINVAL
    }

    // Check that alignment is a power of 2 and at least sizeof(void*)
    if !alignment.is_power_of_two() || alignment < core::mem::size_of::<*mut core::ffi::c_void>() {
        return 22; // EINVAL
    }

    let ptr = unsafe { aligned_alloc(alignment, size) };
    if ptr.is_null() {
        return 12; // ENOMEM
    }

    unsafe { *memptr = ptr };
    0 // Success
}
