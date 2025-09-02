//! WASI libc Rust FFI Bindings
//!
//! This crate provides Rust FFI bindings for wasi-libc C library functions.
//! It uses bindgen to generate safe Rust wrappers around C functions.
//!
//! # Usage
//!
//! ```rust
//! use wasi_libc_bindgen::*;
//!
//! // Use C library functions through safe Rust bindings
//! unsafe {
//!     let ptr = malloc(1024);
//!     if !ptr.is_null() {
//!         free(ptr);
//!     }
//! }
//! ```
//!
//! # Features
//!
//! - `link-libc`: Enable linking with libc.a for WASM targets

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(improper_ctypes)]

//skips whole file from cargo fmt --all --check for CI
#[rustfmt::skip]
mod bindings;
