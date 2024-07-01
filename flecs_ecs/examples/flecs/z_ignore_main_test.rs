#![cfg_attr(feature = "flecs_nightly_tests", feature(internal_output_capture))]
#![allow(dead_code)]
#![allow(clippy::print_stderr, clippy::print_stdout)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused)]

// to initialize the OS api for flecs before tests run.
#[cfg(test)]
#[ctor::ctor]
fn init() {
    unsafe {
        flecs_ecs::sys::ecs_os_init();
    }
}

pub mod z_ignore_test_common;

mod entities;
mod hello_world;
mod observers;
mod prefabs;
mod queries;
mod relationships;
mod systems;

fn main() {}
