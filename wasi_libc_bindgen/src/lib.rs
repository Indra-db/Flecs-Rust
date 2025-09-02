//! WASI libc Top Half - Musl Rust FFI Bindings
//!
//! This crate provides Rust FFI bindings for the **top half** of musl libc, specifically targeting
//! the WASI (WebAssembly System Interface) environment. The "top half" refers to the musl libc
//! source code and headers that provide the C standard library interface, excluding the bottom
//! half which contains platform-specific system call implementations.
//!
//! ## Purpose
//!
//! This crate solves the compatibility issue between Rust programs using C libraries and the
//! wasm32-unknown-unknown target by providing:
//!
//! 1. **Rust FFI bindings** - Generated via bindgen from musl headers
//! 2. **Compiled libc.a** - Top half musl implementation for WASM
//! 3. **Build system integration** - Automatic header generation and library compilation
//!
//! ## Musl Top Half vs Bottom Half
//!
//! - **Top Half** (this crate): Standard C library functions (malloc, memcpy, string.h, etc.)
//! - **Bottom Half**: Platform-specific system calls (provided by WASI runtime)
//!
//! ## Features
//!
//! - `bindgen`: Regenerate Rust bindings from C headers
//! - `build-libc`: Compile libc.a from musl source code  
//! - `link-libc`: Link with pre-compiled libc.a (for end users)
//!
//! ## Usage
//!
//! ```rust
//! use wasi_musl_libc_top_half::*;
//!
//! unsafe {
//!     let ptr = malloc(1024);
//!     memset(ptr, 0, 1024);
//!     free(ptr);
//! }
//! ```

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(improper_ctypes)]

//skips whole file from cargo fmt --all --check for CI
#[rustfmt::skip]
mod bindings;
