#![allow(unused)]
include!("common.rs");
use common::*;

pub fn add_existing_operations(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_add_existing");

    ////////////////////////
    // Add tags
    ////////////////////////

    bench_loop_entities!(
        group,
        "tags/1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range, (T, 1, 1))
        ; (add_component_range, (T, 1, 1))
        ; (reset_component_range, (T, 1, 1))
    );

    bench_loop_entities!(
        group,
        "tags/2",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range, (T, 1, 2))
        ; (add_component_range, (T, 1, 2))
        ; (reset_component_range, (T, 1, 2))
    );

    bench_loop_entities!(
        group,
        "tags/16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range, (T, 1, 16))
        ; (add_component_range, (T, 1, 16))
        ; (reset_component_range, (T, 1, 16))
    );

    bench_loop_entities!(
        group,
        "tags/64",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range, (T, 1, 64))
        ; (add_component_range, (T, 1, 64))
        ; (reset_component_range, (T, 1, 64))
    );

    ////////////////////////
    // Add components
    ////////////////////////

    bench_loop_entities!(
        group,
        "components/1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range, (C, 1, 1))
        ; (add_component_range, (C, 1, 1))
        ; (reset_component_range, (C, 1, 1))
    );

    bench_loop_entities!(
        group,
        "components/2",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 2))
        ; (add_component_range, (C, 1, 2))
        ; (add_component_range, (C, 1, 2))
        ; (reset_component_range, (C, 1, 2))
    );

    bench_loop_entities!(
        group,
        "components/16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range, (C, 1, 16))
        ; (add_component_range, (C, 1, 16))
        ; (reset_component_range, (C, 1, 16))
    );

    bench_loop_entities!(
        group,
        "components/64",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range, (C, 1, 64))
        ; (add_component_range, (C, 1, 64))
        ; (reset_component_range, (C, 1, 64))
    );

    group.finish();
}

pub fn add_existing_cmd(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_add_existing_cmd");

    ////////////////////////
    // Add tags
    ////////////////////////

    bench_loop_entities!(
        group,
        "tags/1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range_cmd, (T, 1, 1))
        ; (add_component_range_cmd, (T, 1, 1))
        ; (reset_component_range, (T, 1, 1))
    );

    bench_loop_entities!(
        group,
        "tags/2",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range_cmd, (T, 1, 2))
        ; (add_component_range_cmd, (T, 1, 2))
        ; (reset_component_range, (T, 1, 2))
    );

    bench_loop_entities!(
        group,
        "tags/16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range_cmd, (T, 1, 16))
        ; (add_component_range_cmd, (T, 1, 16))
        ; (reset_component_range, (T, 1, 16))
    );

    bench_loop_entities!(
        group,
        "tags/64",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range_cmd, (T, 1, 64))
        ; (add_component_range_cmd, (T, 1, 64))
        ; (reset_component_range, (T, 1, 64))
    );

    ////////////////////////
    // Add components
    ////////////////////////

    bench_loop_entities!(
        group,
        "components/1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range_cmd, (C, 1, 1))
        ; (add_component_range_cmd, (C, 1, 1))
        ; (reset_component_range, (C, 1, 1))
    );

    bench_loop_entities!(
        group,
        "components/2",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 2))
        ; (add_component_range_cmd, (C, 1, 2))
        ; (add_component_range_cmd, (C, 1, 2))
        ; (reset_component_range, (C, 1, 2))
    );

    bench_loop_entities!(
        group,
        "components/16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range_cmd, (C, 1, 16))
        ; (add_component_range_cmd, (C, 1, 16))
        ; (reset_component_range, (C, 1, 16))
    );

    bench_loop_entities!(
        group,
        "components/64",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range_cmd, (C, 1, 64))
        ; (add_component_range_cmd, (C, 1, 64))
        ; (reset_component_range, (C, 1, 64))
    );

    group.finish();
}
