use crate::common_bench::*;

pub fn set_remove(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_set");

    bench_loop_entities!(
        group,
        "components/1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; // preparation
        ; (set_component_range, (C, 1, 1)), (remove_component_range, (C, 1, 1))
        ; (reset_component_range, (C, 1, 1))
    );

    bench_loop_entities!(
        group,
        "components/2",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 2))
        ; // preparation
        ; (set_component_range, (C, 1, 2)), (remove_component_range, (C, 1, 2))
        ; (reset_component_range, (C, 1, 2))
    );

    bench_loop_entities!(
        group,
        "components/16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 16))
        ; // preparation
        ; (set_component_range, (C, 1, 16)), (remove_component_range, (C, 1, 16))
        ; (reset_component_range, (C, 1, 16))
    );

    bench_loop_entities!(
        group,
        "components/64",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 64))
        ; // preparation
        ; (set_component_range, (C, 1, 64)), (remove_component_range, (C, 1, 64))
        ; (reset_component_range, (C, 1, 64))
    );

    group.finish();
}

pub fn set_remove_cmd(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_set_remove_cmd");

    bench_loop_entities!(
        group,
        "components/1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; // preparation
        ; (set_component_range_cmd, (C, 1, 1)), (remove_component_range, (C, 1, 1))
        ; (reset_component_range, (C, 1, 1))
    );

    bench_loop_entities!(
        group,
        "components/2",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 2))
        ; // preparation
        ; (set_component_range_cmd, (C, 1, 2)), (remove_component_range, (C, 1, 2))
        ; (reset_component_range, (C, 1, 2))
    );

    bench_loop_entities!(
        group,
        "components/16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 16))
        ; // preparation
        ; (set_component_range_cmd, (C, 1, 16)), (remove_component_range, (C, 1, 16))
        ; (reset_component_range, (C, 1, 16))
    );

    bench_loop_entities!(
        group,
        "components/64",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 64))
        ; // preparation
        ; (set_component_range_cmd, (C, 1, 64)), (remove_component_range, (C, 1, 64))
        ; (reset_component_range, (C, 1, 64))
    );

    group.finish();
}

pub fn set_found(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_set_found");

    bench_loop_entities!(
        group,
        "components/1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range, (C, 1, 1))
        ; (set_component_range, (C, 1, 1))
        ; (reset_component_range, (C, 1, 1))
    );

    bench_loop_entities!(
        group,
        "components/2",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 2))
        ; (add_component_range, (C, 1, 2))
        ; (set_component_range, (C, 1, 2))
        ; (reset_component_range, (C, 1, 2))
    );

    bench_loop_entities!(
        group,
        "components/16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 16))
        ; (add_component_range, (C, 1, 16))
        ; (set_component_range, (C, 1, 16))
        ; (reset_component_range, (C, 1, 16))
    );

    bench_loop_entities!(
        group,
        "components/64",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 64))
        ; (add_component_range, (C, 1, 64))
        ; (set_component_range, (C, 1, 64))
        ; (reset_component_range, (C, 1, 64))
    );

    group.finish();
}

pub fn set_found_cmd(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_set_found_cmd");

    bench_loop_entities!(
        group,
        "components/1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range, (C, 1, 1))
        ; (set_component_range_cmd, (C, 1, 1))
        ; (reset_component_range, (C, 1, 1))
    );

    bench_loop_entities!(
        group,
        "components/2",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 2))
        ; (add_component_range, (C, 1, 2))
        ; (set_component_range_cmd, (C, 1, 2))
        ; (reset_component_range, (C, 1, 2))
    );

    bench_loop_entities!(
        group,
        "components/16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 16))
        ; (add_component_range, (C, 1, 16))
        ; (set_component_range_cmd, (C, 1, 16))
        ; (reset_component_range, (C, 1, 16))
    );

    bench_loop_entities!(
        group,
        "components/64",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 64))
        ; (add_component_range, (C, 1, 64))
        ; (set_component_range_cmd, (C, 1, 64))
        ; (reset_component_range, (C, 1, 64))
    );

    group.finish();
}
