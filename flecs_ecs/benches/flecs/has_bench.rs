use crate::common_bench::*;

pub fn has(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs");

    bench_loop_entities!(
       group,
       "has_not_found",
       ENTITY_COUNT
       ; (register_component_range, (C, 1, 2)) // Registration so it doesn't affect the benchmark
       ;  (set_component_range, (C, 1, 1)) // Preparation
       ; (has_component_range, (C, 2, 2)) // Benchmark
    );

    bench_loop_entities!(
        group,
        "has",
        ENTITY_COUNT
        ; // Empty registration
        ; (set_component_range, (C, 1, 1)) // Preparation
        ; (has_component_range, (C, 1, 1)) // Benchmark
    );
    group.finish();
}

pub fn owns(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs");

    bench_loop_entities!(
       group,
       "owns_not_found",
       ENTITY_COUNT
       ; (register_component_range, (C, 1, 2)) // Registration so it doesn't affect the benchmark
       ; (set_component_range, (C, 1, 1)) // Preparation
       ; (owns_component_range, (C, 2, 2)) // Benchmark
    );

    bench_loop_entities!(
        group,
        "owns",
        ENTITY_COUNT
        ; // Empty registration
        ; (set_component_range, (C, 1, 1)) // Preparation
        ; (owns_component_range, (C, 1, 1)) // Benchmark
    );

    group.finish();
}
