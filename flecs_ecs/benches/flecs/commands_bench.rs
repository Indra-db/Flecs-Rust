use crate::common_bench::*;

pub fn add_remove_cmd(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs");

    // tags

    bench_loop_entities!(
        group,
        "add_remove_cmd_tags_1",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; // Preparation
        ; (add_component_range_cmd, (T, 1, 1)), (remove_component_range_cmd, (T, 1, 1))
    );

    bench_loop_entities!(
        group,
        "add_remove_cmd_tags_2",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 2))
        ; // Preparation
        ; (add_component_range_cmd, (T, 1, 2)), (remove_component_range_cmd, (T, 1, 2))
    );

    bench_loop_entities!(
        group,
        "add_remove_cmd_tags_16",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 16))
        ; // Preparation
        ; (add_component_range_cmd, (T, 1, 16)), (remove_component_range_cmd, (T, 1, 16))
    );

    bench_loop_entities!(
        group,
        "add_remove_cmd_tags_32",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 32))
        ; // Preparation
        ; (add_component_range_cmd, (T, 1, 32)), (remove_component_range_cmd, (T, 1, 32))
    );

    // components

    bench_loop_entities!(
        group,
        "add_remove_cmd_components_1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; // Preparation
        ; (set_component_range_cmd, (C, 1, 1)), (remove_component_range_cmd, (C, 1, 1))
    );

    bench_loop_entities!(
        group,
        "add_remove_cmd_components_2",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 2))
        ; // Preparation
        ; (set_component_range_cmd, (C, 1, 2)), (remove_component_range_cmd, (C, 1, 2))
    );

    bench_loop_entities!(
        group,
        "add_remove_cmd_components_16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 16))
        ; // Preparation
        ; (set_component_range_cmd, (C, 1, 16)), (remove_component_range_cmd, (C, 1, 16))
    );

    bench_loop_entities!(
        group,
        "add_remove_cmd_components_32",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 32))
        ; // Preparation
        ; (set_component_range_cmd, (C, 1, 32)), (remove_component_range_cmd, (C, 1, 32))
    );

    group.finish();
}

pub fn add_existing_cmd(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs");

    // Add tags

    bench_loop_entities!(
        group,
        "add_existing_cmd_tags_1",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; (add_component_range_cmd, (T, 1, 1))
        ; (add_component_range_cmd, (T, 1, 1))
    );

    bench_loop_entities!(
        group,
        "add_existing_cmd_tags_2",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; (add_component_range_cmd, (T, 1, 2))
        ; (add_component_range_cmd, (T, 1, 2))
    );

    bench_loop_entities!(
        group,
        "add_existing_cmd_tags_16",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; (add_component_range_cmd, (T, 1, 16))
        ; (add_component_range_cmd, (T, 1, 16))
    );

    bench_loop_entities!(
        group,
        "add_existing_cmd_tags_32",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; (add_component_range_cmd, (T, 1, 32))
        ; (add_component_range_cmd, (T, 1, 32))
    );

    // Add components

    bench_loop_entities!(
        group,
        "add_existing_cmd_components_1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range_cmd, (C, 1, 1))
        ; (add_component_range_cmd, (C, 1, 1))
    );

    bench_loop_entities!(
        group,
        "add_existing_cmd_components_2",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range_cmd, (C, 1, 2))
        ; (add_component_range_cmd, (C, 1, 2))
    );

    bench_loop_entities!(
        group,
        "add_existing_cmd_components_16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range_cmd, (C, 1, 16))
        ; (add_component_range_cmd, (C, 1, 16))
    );

    bench_loop_entities!(
        group,
        "add_existing_cmd_components_32",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range_cmd, (C, 1, 32))
        ; (add_component_range_cmd, (C, 1, 32))
    );

    group.finish();
}

pub fn create_delete_entities_cmd(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs");

    group.bench_function("create_delete_entities_cmd_empty", |bencher| {
        let world = World::new();

        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                world.defer_begin();
                for _ in 0..ENTITY_COUNT {
                    let entity = world.entity();
                    entity.destruct();
                }
                world.defer_end();
            }
            let elapsed = start.elapsed();
            elapsed / 2 //time average per entity operation
        });
    });

    group.bench_function("create_delete_entities_cmd_empty_named", |bencher| {
        let world = World::new();

        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                world.defer_begin();
                for _ in 0..ENTITY_COUNT {
                    let entity = world.entity_named("hello");
                    entity.destruct();
                }
                world.defer_end();
            }
            let elapsed = start.elapsed();
            elapsed / (ENTITY_COUNT * 2) //time average per entity operation
        });
    });

    //tags
    bench_create_delete_entity_cmd!(group, "tag_1", ENTITY_COUNT, T, 1, 1, add_component_range);
    bench_create_delete_entity_cmd!(group, "tag_2", ENTITY_COUNT, T, 1, 2, add_component_range);
    bench_create_delete_entity_cmd!(group, "tag_16", ENTITY_COUNT, T, 1, 16, add_component_range);
    bench_create_delete_entity_cmd!(group, "tag_64", ENTITY_COUNT, T, 1, 64, add_component_range);
    // components
    bench_create_delete_entity_cmd!(
        group,
        "component_1",
        ENTITY_COUNT,
        C,
        1,
        1,
        set_component_range
    );
    bench_create_delete_entity_cmd!(
        group,
        "component_2",
        ENTITY_COUNT,
        C,
        1,
        2,
        set_component_range
    );
    bench_create_delete_entity_cmd!(
        group,
        "component_16",
        ENTITY_COUNT,
        C,
        1,
        16,
        set_component_range
    );
    bench_create_delete_entity_cmd!(
        group,
        "component_64",
        ENTITY_COUNT,
        C,
        1,
        64,
        set_component_range
    );

    group.finish();
}

pub fn set_remove_cmd(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs");

    bench_loop_entities!(
        group,
        "set_remove_cmd_1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; // preparation
        ; (set_component_range_cmd, (C, 1, 1)), (remove_component_range, (C, 1, 1))
    );

    bench_loop_entities!(
        group,
        "set_remove_cmd_2",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 2))
        ; // preparation
        ; (set_component_range_cmd, (C, 1, 2)), (remove_component_range, (C, 1, 2))
    );

    bench_loop_entities!(
        group,
        "set_remove_cmd_16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 16))
        ; // preparation
        ; (set_component_range_cmd, (C, 1, 16)), (remove_component_range, (C, 1, 16))
    );

    bench_loop_entities!(
        group,
        "set_remove_cmd_32",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 32))
        ; // preparation
        ; (set_component_range_cmd, (C, 1, 32)), (remove_component_range, (C, 1, 32))
    );

    group.finish();
}

pub fn set_cmd(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs");

    bench_loop_entities!(
        group,
        "set_cmd_1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ;
        ; (set_component_range_cmd, (C, 1, 1))
    );

    bench_loop_entities!(
        group,
        "set_cmd_2",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 2))
        ;
        ; (set_component_range_cmd, (C, 1, 2))
    );

    bench_loop_entities!(
        group,
        "set_cmd_16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 16))
        ;
        ; (set_component_range_cmd, (C, 1, 16))
    );

    bench_loop_entities!(
        group,
        "set_cmd_32",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 32))
        ;
        ; (set_component_range_cmd, (C, 1, 32))
    );

    group.finish();
}
