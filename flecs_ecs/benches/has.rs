#![allow(unused)]
include!("common.rs");
use common::*;

pub fn has_component_not_found(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_has_component_not_found");

    bench_loop_entities!(
       group,
       "empty_entity",
       ENTITY_COUNT
       ; (register_component_range, (C, 1, 1)) // Registration so it doesn't affect the benchmark
       ;  // Preparation
       ; (has_component_range, (C, 1, 1)) // Benchmark
       ; (reset_component_range, (C, 1, 1)) // Cleanup
    );

    bench_loop_entities!(
        group,
        "not_empty_entity",
        ENTITY_COUNT
        ; (register_component_range, (C, 2, 2)) // Registration so it doesn't affect the benchmark
        ; (add_component_range, (C, 1, 1)) // Preparation
        ; (has_component_range, (C, 2, 2)) // Benchmark
        ; (reset_component_range, (C, 1, 2)) // Cleanup
    );

    group.finish();
}

pub fn has_components_found(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_has_components_found");

    bench_loop_entities!(
        group,
        "1",
        ENTITY_COUNT
        ; // Empty registration
        ; (add_component_range, (C, 1, 1)) // Preparation
        ; (has_component_range, (C, 1, 1)) // Benchmark
        ; (reset_component_range, (C, 1, 1)) // Cleanup
    );

    bench_loop_entities!(
        group,
        "2",
        ENTITY_COUNT
        ; // Empty registration
        ;(add_component_range, (C, 1, 2)) //preparation code
        ; (has_component_range, (C, 1, 2)) //benchmark code
        ; (reset_component_range, (C, 1, 2)) //reset code
    );

    bench_loop_entities!(
        group,
        "16",
        ENTITY_COUNT
        ; //registration code
        ;(add_component_range, (C, 1, 16)) //preparation code
        ; (has_component_range, (C, 1, 16)) //benchmark code
        ; (reset_component_range, (C, 1, 16)) //reset code
    );

    bench_loop_entities!(
        group,
        "64",
        ENTITY_COUNT
        ; //registration code
        ;(add_component_range, (C, 1, 64)) //preparation code
        ; (has_component_range, (C, 1, 64)) //benchmark code
        ; (reset_component_range, (C, 1, 64)) //reset code
    );

    group.finish();
}
