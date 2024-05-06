use crate::common_bench::*;

pub fn add_remove(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_add_remove");

    bench_loop_entities!(
        group,
        "tags/1",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; // Preparation
        ; (add_component_range, (T, 1, 1)), (remove_component_range, (T, 1, 1))
        ; (reset_component_range, (T, 1, 1))
    );

    bench_loop_entities!(
        group,
        "tags/2",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 2))
        ; // Preparation
        ; (add_component_range, (T, 1, 2)), (remove_component_range, (T, 1, 2))
        ; (reset_component_range, (T, 1, 2))
    );

    bench_loop_entities!(
        group,
        "tags/16",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 16))
        ; // Preparation
        ; (add_component_range, (T, 1, 16)), (remove_component_range, (T, 1, 16))
        ; (reset_component_range, (T, 1, 16))
    );

    bench_loop_entities!(
        group,
        "tags/64",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 64))
        ; // Preparation
        ; (add_component_range, (T, 1, 64)), (remove_component_range, (T, 1, 64))
        ; (reset_component_range, (T, 1, 64))
    );

    bench_loop_entities!(
        group,
        "components/1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; // Preparation
        ; (add_component_range, (C, 1, 1)), (remove_component_range, (C, 1, 1))
        ; (reset_component_range, (C, 1, 1))
    );

    bench_loop_entities!(
        group,
        "components/2",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 2))
        ; // Preparation
        ; (add_component_range, (C, 1, 2)), (remove_component_range, (C, 1, 2))
        ; (reset_component_range, (C, 1, 2))
    );

    bench_loop_entities!(
        group,
        "components/16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 16))
        ; // Preparation
        ; (add_component_range, (C, 1, 16)), (remove_component_range, (C, 1, 16))
        ; (reset_component_range, (C, 1, 16))
    );

    bench_loop_entities!(
        group,
        "components/64",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 64))
        ; // Preparation
        ; (add_component_range, (C, 1, 64)), (remove_component_range, (C, 1, 64))
        ; (reset_component_range, (C, 1, 64))
    );

    group.finish();
}

#[allow(unused)]
pub fn c_add_remove_tags(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_c_add_remove_tags");
    group.bench_function("32_c", |bencher| {
        unsafe {
            let id_count = 32;
            let world = ecs_mini();
            let entities = create_ids(world, ENTITY_COUNT as i32, 0, false);
            let ids = create_ids(world, id_count, 0, true);

            bencher.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    for entity in &entities {
                        for id in &ids {
                            ecs_add_id(world, *entity, *id);
                        }
                    }
                    for entity in &entities {
                        for id in &ids {
                            ecs_remove_id(world, *entity, *id);
                        }
                    }
                }
                start.elapsed() / (2 * ENTITY_COUNT * id_count as u32) // Return the total time per entity
            });
            ecs_fini(world);
        }
    });

    group.finish();
}

pub fn add_remove_cmd(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_add_remove_cmd");

    bench_loop_entities!(
        group,
        "tags/1",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; // Preparation
        ; (add_component_range_cmd, (T, 1, 1)), (remove_component_range_cmd, (T, 1, 1))
        ; (reset_component_range, (T, 1, 1))
    );

    bench_loop_entities!(
        group,
        "tags/2",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 2))
        ; // Preparation
        ; (add_component_range_cmd, (T, 1, 2)), (remove_component_range_cmd, (T, 1, 2))
        ; (reset_component_range, (T, 1, 2))
    );

    bench_loop_entities!(
        group,
        "tags/16",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 16))
        ; // Preparation
        ; (add_component_range_cmd, (T, 1, 16)), (remove_component_range_cmd, (T, 1, 16))
        ; (reset_component_range, (T, 1, 16))
    );

    bench_loop_entities!(
        group,
        "tags/64",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 64))
        ; // Preparation
        ; (add_component_range_cmd, (T, 1, 64)), (remove_component_range_cmd, (T, 1, 64))
        ; (reset_component_range, (T, 1, 64))
    );

    bench_loop_entities!(
        group,
        "components/1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; // Preparation
        ; (add_component_range_cmd, (C, 1, 1)), (remove_component_range_cmd, (C, 1, 1))
        ; (reset_component_range, (C, 1, 1))
    );

    bench_loop_entities!(
        group,
        "components/2",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 2))
        ; // Preparation
        ; (add_component_range_cmd, (C, 1, 2)), (remove_component_range_cmd, (C, 1, 2))
        ; (reset_component_range, (C, 1, 2))
    );

    bench_loop_entities!(
        group,
        "components/16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 16))
        ; // Preparation
        ; (add_component_range_cmd, (C, 1, 16)), (remove_component_range_cmd, (C, 1, 16))
        ; (reset_component_range, (C, 1, 16))
    );

    bench_loop_entities!(
        group,
        "components/64",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 64))
        ; // Preparation
        ; (add_component_range_cmd, (C, 1, 64)), (remove_component_range_cmd, (C, 1, 64))
        ; (reset_component_range, (C, 1, 64))
    );

    group.finish();
}

pub fn add_remove_1_tag_to_entity_with_n_components(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_add_remove_1_tag_to_entity_w_components");

    bench_loop_entities!(
        group,
        "1",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; (add_component_range, (C, 1, 1))
        ; (add_component_range, (T, 1, 1)), (remove_component_range, (T, 1, 1))
        ; (reset_component_range, (T, 1, 1)), (reset_component_range, (C, 1, 1))
    );

    bench_loop_entities!(
        group,
        "2",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; (add_component_range, (C, 1, 2))
        ; (add_component_range, (T, 1, 1)), (remove_component_range, (T, 1, 1))
        ; (reset_component_range, (T, 1, 1)), (reset_component_range, (C, 1, 2))
    );

    bench_loop_entities!(
        group,
        "16",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; (add_component_range, (C, 1, 16))
        ; (add_component_range, (T, 1, 1)), (remove_component_range, (T, 1, 1))
        ; (reset_component_range, (T, 1, 1)), (reset_component_range, (C, 1, 16))
    );

    bench_loop_entities!(
        group,
        "64",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; (add_component_range, (C, 1, 64))
        ; (add_component_range, (T, 1, 1)), (remove_component_range, (T, 1, 1))
        ; (reset_component_range, (T, 1, 1)), (reset_component_range, (C, 1, 64))
    );

    group.finish();
}
