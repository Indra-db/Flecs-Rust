//! Tests from flecsremoteapi.md

#![allow(unused_imports, unused_variables, dead_code, non_snake_case, path_statements, unreachable_code, unused_mut)]
#![cfg_attr(rustfmt, rustfmt_skip)]

use crate::common_test::*;

#[test]
fn flecs_remote_api_explorer_quickstart_01() {
    return; //compile-only
    let world = World::new();
    // Optional, gather statistics for explorer
    world.import::<stats::Stats>();

    // Creates REST server on default port (27750)
    world.set(flecs::rest::Rest::default());

    // Runs the system serving up REST requests
    while world.progress() {}
}

#[test]
fn flecs_remote_api_explorer_quickstart_02() {
    return; //compile-only
    let world = World::new();
    world
        .app()
        // Optional, gather statistics for explorer
        .enable_stats(true)
        .enable_rest(0)
        .run();
}

#[test]
fn flecs_remote_api_usage_rest_api_03() {
    return; //compile-only
    let world = World::new();
    // Optional, gather statistics for explorer
    world.import::<stats::Stats>();

    // Creates REST server on default port (27750)
    world.set(flecs::rest::Rest::default());

    // Runs the system serving up REST requests
    while world.progress() {}
}