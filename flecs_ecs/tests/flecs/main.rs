#![allow(dead_code)]
//! integration tests for the public API of the `flecs_ecs` crate
//! split into a single module due to the benefits it provides.
//! see <`https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html/`> for more information.

pub mod common_test;

mod clone_default_impl_test;
mod component_lifecycle_test;
mod component_test;
mod entity_bulk_rust_test;
mod entity_rust_test;
mod entity_test;
mod enum_test;
mod eq_test;
//mod flecs_docs_test;
mod is_ref_test;
mod meta_macro_test;
mod meta_test;
mod meta_test_rust;
mod meta_trait_test;
mod module_test;
mod observer_rust_test;
mod observer_test;
mod query_builder_test;
mod query_rust_test;
mod query_test;
#[cfg(feature = "flecs_safety_locks")]
mod safety;
mod system_test;
mod world_test;
