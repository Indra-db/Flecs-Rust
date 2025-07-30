use crate::common_bench::*;
use flecs_ecs::sys;

pub fn add_remove(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs");

    // tags

    bench_loop_entities!(
        group,
        "add_remove_tags_1",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; // Preparation
        ; (add_component_range, (T, 1, 1)), (remove_component_range, (T, 1, 1))
    );

    bench_loop_entities!(
        group,
        "add_remove_tags_2",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 2))
        ; // Preparation
        ; (add_component_range, (T, 1, 2)), (remove_component_range, (T, 1, 2))
    );

    bench_loop_entities!(
        group,
        "add_remove_tags_16",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 16))
        ; // Preparation
        ; (add_component_range, (T, 1, 16)), (remove_component_range, (T, 1, 16))
    );

    bench_loop_entities!(
        group,
        "add_remove_tags_32",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 32))
        ; // Preparation
        ; (add_component_range, (T, 1, 32)), (remove_component_range, (T, 1, 32))
    );

    // components

    bench_loop_entities!(
        group,
        "add_remove_components_1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; // Preparation
        ; (set_component_range, (C, 1, 1)), (remove_component_range, (C, 1, 1))
    );

    bench_loop_entities!(
        group,
        "add_remove_components_2",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 2))
        ; // Preparation
        ; (set_component_range, (C, 1, 2)), (remove_component_range, (C, 1, 2))
    );

    bench_loop_entities!(
        group,
        "add_remove_components_16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 16))
        ; // Preparation
        ; (set_component_range, (C, 1, 16)), (remove_component_range, (C, 1, 16))
    );

    bench_loop_entities!(
        group,
        "add_remove_components_32",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 32))
        ; // Preparation
        ; (set_component_range, (C, 1, 32)), (remove_component_range, (C, 1, 32))
    );

    group.finish();
}

pub fn add_remove_hooks(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs");

    bench_add_remove_hooks!(group, "1", 1);
    bench_add_remove_hooks!(group, "2", 2);
    bench_add_remove_hooks!(group, "16", 16);
    bench_add_remove_hooks!(group, "32", 32);

    group.finish();
}

pub fn add_existing(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs");

    // Add tags

    bench_loop_entities!(
        group,
        "add_existing_tags_1",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; (add_component_range, (T, 1, 1))
        ; (add_component_range, (T, 1, 1))
    );

    bench_loop_entities!(
        group,
        "add_existing_tags_16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range, (T, 1, 16))
        ; (add_component_range, (T, 1, 16))
    );

    // Add components

    bench_loop_entities!(
        group,
        "add_existing_components_1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range, (C, 1, 1))
        ; (add_component_range, (C, 1, 1))
    );

    bench_loop_entities!(
        group,
        "add_existing_components_16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1))
        ; (add_component_range, (C, 1, 16))
        ; (add_component_range, (C, 1, 16))
    );

    group.finish();
}

pub fn add_remove_1_tag_to_entity_with_n_components(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs");

    bench_loop_entities!(
        group,
        "add_remove_tag_to_components_1",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; (set_component_range_cmd, (C, 1, 1))
        ; (add_component_range, (T, 1, 1)), (remove_component_range, (T, 1, 1))
    );

    bench_loop_entities!(
        group,
        "add_remove_tag_to_components_4",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; (set_component_range_cmd, (C, 1, 4))
        ; (add_component_range, (T, 1, 1)), (remove_component_range, (T, 1, 1))
    );

    bench_loop_entities!(
        group,
        "add_remove_tag_to_components_8",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; (set_component_range_cmd, (C, 1, 8))
        ; (add_component_range, (T, 1, 1)), (remove_component_range, (T, 1, 1))
    );

    bench_loop_entities!(
        group,
        "add_remove_tag_to_components_16",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; (set_component_range_cmd, (C, 1, 16))
        ; (add_component_range, (T, 1, 1)), (remove_component_range, (T, 1, 1))
    );

    bench_loop_entities!(
        group,
        "add_remove_tag_to_components_64",
        ENTITY_COUNT
        ; (register_component_range, (T, 1, 1))
        ; (set_component_range_cmd, (C, 1, 64))
        ; (add_component_range, (T, 1, 1)), (remove_component_range, (T, 1, 1))
    );

    group.finish();
}

pub fn add_remove_override(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs");

    bench_add_remove_override!(group, "1", 1);
    bench_add_remove_override!(group, "2", 2);
    bench_add_remove_override!(group, "4", 4);
    bench_add_remove_override!(group, "16", 16);

    group.finish();
}

pub fn toggle_component(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs");

    bench_loop_entities!(
        group,
        "toggle_tags_16",
        ENTITY_COUNT
        ; (set_components_togglable, (T,1,16))
        ; (add_component_range, (T, 1, 16))
        ; (enable_disable_component_range, (T, 1, 16))
    );

    bench_loop_entities!(
        group,
        "toggle_tags_32",
        ENTITY_COUNT
        ; (set_components_togglable, (T,1,32))
        ; (add_component_range, (T, 1, 32))
        ; (enable_disable_component_range, (T, 1, 32))
    );

    bench_loop_entities!(
        group,
        "toggle_component_16",
        ENTITY_COUNT
        ; (set_components_togglable, (C,1,16))
        ; (set_component_range, (C, 1, 16))
        ; (enable_disable_component_range, (C, 1, 16))
    );

    bench_loop_entities!(
        group,
        "toggle_component_32",
        ENTITY_COUNT
        ; (set_components_togglable, (C,1,32))
        ; (set_component_range, (C, 1, 32))
        ; (enable_disable_component_range, (C, 1, 32))
    );
    group.finish();
}

#[allow(unused)]
pub fn c_add_remove_tags(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_c_add_remove_tags");
    group.bench_function("32_c", |bencher| {
        unsafe {
            let id_count = 32;
            let world = sys::ecs_mini();
            let entities = create_ids(world, ENTITY_COUNT as usize, 0, false, false, true);
            let ids = create_ids(world, id_count, 0, true, false, true);

            bencher.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    for entity in &entities {
                        for id in &ids {
                            sys::ecs_add_id(world, *entity, *id);
                        }
                    }
                    for entity in &entities {
                        for id in &ids {
                            sys::ecs_remove_id(world, *entity, *id);
                        }
                    }
                }
                start.elapsed() / (2 * ENTITY_COUNT * id_count as u32) // Return the total time per entity
            });
            sys::ecs_fini(world);
        }
    });

    group.finish();
}
