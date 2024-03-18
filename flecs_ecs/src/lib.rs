#![allow(dead_code)]
#![warn(clippy::doc_markdown, clippy::semicolon_if_nothing_returned)]

pub use flecs_ecs_derive as macros;
pub use flecs_ecs_sys as sys;

pub mod core;

pub mod addons;
