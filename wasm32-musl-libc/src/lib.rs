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
