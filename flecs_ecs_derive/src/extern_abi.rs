//! Attribute macro for conditional extern ABI specification.
//!
//! This module provides the `extern_abi` attribute macro that conditionally applies
//! the appropriate extern ABI based on the target platform.
//!
//! # Purpose
//!
//! WASM targets don't support unwinding, so they require `extern "C"` instead of
//! `extern "C-unwind"`. This macro automatically selects the correct ABI based on
//! the target platform.
//!
//! # Usage
//!
//! ```ignore
//! use flecs_ecs_derive::extern_abi;
//!
//! #[extern_abi]
//! fn my_callback() {
//!     // Function implementation
//! }
//! ```
//!
//! This expands to:
//! - `extern "C" fn my_callback() { ... }` on WASM targets
//! - `extern "C-unwind" fn my_callback() { ... }` on other targets

use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

/// Expansion function for the `extern_abi` attribute macro.
///
/// This function generates platform-specific extern declarations based on the target.
///
/// # Arguments
///
/// * `input_fn` - The parsed function item to apply the extern ABI to
///
/// # Returns
///
/// A `TokenStream` containing the platform-specific extern function declarations
///
/// # Errors
///
/// Returns a compile error if the function already has an extern specification.
pub(crate) fn expand_extern_abi(input_fn: ItemFn) -> TokenStream {
    let fn_name = &input_fn.sig.ident;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_block = &input_fn.block;
    let fn_generics = &input_fn.sig.generics;
    let fn_where_clause = &input_fn.sig.generics.where_clause;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;

    // Check if there's already an extern specification
    if input_fn.sig.abi.is_some() {
        return quote! {
            compile_error!("Function already has an extern ABI specification. Remove it to use #[extern_abi].");
        };
    }

    quote! {
        #(#fn_attrs)*
        #[cfg(target_family = "wasm")]
        #fn_vis extern "C" fn #fn_name #fn_generics(#fn_inputs) #fn_output #fn_where_clause #fn_block

        #(#fn_attrs)*
        #[cfg(not(target_family = "wasm"))]
        #fn_vis extern "C-unwind" fn #fn_name #fn_generics(#fn_inputs) #fn_output #fn_where_clause #fn_block
    }
}
