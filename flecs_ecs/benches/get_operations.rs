mod common;
use common::*;

pub fn flecs_get_components_not_found(criterion: &mut Criterion) {
    let mut group = create_group(criterion, "flecs_get_components_not_found");

    bench_loop_entities!(
       group,
       "empty_entity",
       ENTITY_COUNT
       ; (register_component_range, (C, 1, 1)) // Registration so it doesn't affect the benchmark
       ;  // Preparation
       ; (get_component_range, (C, 1, 1)) // Benchmark
       ; (reset_component_range, (C, 1, 1)) // Cleanup
    );

    bench_loop_entities!(
        group,
        "entity",
        ENTITY_COUNT
        ; (register_component_range, (C, 2, 2)) // Registration so it doesn't affect the benchmark
        ; (add_component_range, (C, 1, 1)) // Preparation
        ; (get_component_range, (C, 2, 2)) // Benchmark
        ; (reset_component_range, (C, 1, 2)) // Cleanup
    );

    group.finish();
}

pub fn flecs_get_components_found(criterion: &mut Criterion) {
    let mut group = create_group(criterion, "flecs_get_components_found");

    bench_loop_entities!(
        group,
        "1",
        ENTITY_COUNT
        ; // registration
        ; (add_component_range, (C, 1, 1)) // Preparation
        ; (get_component_range, (C, 1, 1)) // Benchmark
        ; (reset_component_range, (C, 1, 1)) // Cleanup
    );

    bench_loop_entities!(
        group,
        "2",
        ENTITY_COUNT
        ; //registration
        ;(add_component_range, (C, 1, 2)) //preparation code
        ; (get_component_range, (C, 1, 2)) //benchmark code
        ; (reset_component_range, (C, 1, 2)) //reset code
    );

    bench_loop_entities!(
        group,
        "16",
        ENTITY_COUNT
        ; //registration
        ;(add_component_range, (C, 1, 16)) //preparation code
        ; (get_component_range, (C, 1, 16)) //benchmark code
        ; (reset_component_range, (C, 1, 16)) //reset code
    );

    bench_loop_entities!(
        group,
        "64",
        ENTITY_COUNT
        ; //registration
        ;(add_component_range, (C, 1, 64)) //preparation code
        ; (get_component_range, (C, 1, 64)) //benchmark code
        ; (reset_component_range, (C, 1, 64)) //reset code
    );

    group.finish();
}

pub fn flecs_get_mut_components_found(criterion: &mut Criterion) {
    let mut group = create_group(criterion, "flecs_get_mut_components_found");

    bench_loop_entities!(
        group,
        "1",
        ENTITY_COUNT
        ; // registration
        ; (add_component_range, (C, 1, 1)) // Preparation
        ; (get_mut_component_range, (C, 1, 1)) // Benchmark
        ; (reset_component_range, (C, 1, 1)) // Cleanup
    );

    bench_loop_entities!(
        group,
        "2",
        ENTITY_COUNT
        ; //registration
        ;(add_component_range, (C, 1, 2)) //preparation code
        ; (get_mut_component_range, (C, 1, 2)) //benchmark code
        ; (reset_component_range, (C, 1, 2)) //reset code
    );

    bench_loop_entities!(
        group,
        "16",
        ENTITY_COUNT
        ; //registration
        ;(add_component_range, (C, 1, 16)) //preparation code
        ; (get_mut_component_range, (C, 1, 16)) //benchmark code
        ; (reset_component_range, (C, 1, 16)) //reset code
    );

    bench_loop_entities!(
        group,
        "64",
        ENTITY_COUNT
        ; //registration
        ;(add_component_range, (C, 1, 64)) //preparation code
        ; (get_mut_component_range, (C, 1, 64)) //benchmark code
        ; (reset_component_range, (C, 1, 64)) //reset code
    );

    group.finish();
}

pub fn flecs_get_mut_components_not_found_and_create(criterion: &mut Criterion) {
    let mut group = create_group(criterion, "flecs_get_mut_components_not_found_and_create");

    bench_loop_entities!(
        group,
        "1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1)) // Registration so it doesn't affect the benchmark
        ;  // Preparation
        ; (ensure_mut_component_range, (C, 1, 1)) // Benchmark
        ; (reset_component_range, (C, 1, 1)) // Cleanup
    );

    bench_loop_entities!(
        group,
        "2",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 2)) // Registration so it doesn't affect the benchmark
        ; // Preparation
        ; (ensure_mut_component_range, (C, 1, 2)) // Benchmark
        ; (reset_component_range, (C, 1, 2)) // Cleanup
    );

    bench_loop_entities!(
        group,
        "16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 16)) // Registration so it doesn't affect the benchmark
        ; // Preparation
        ; (ensure_mut_component_range, (C, 1, 16)) // Benchmark
        ; (reset_component_range, (C, 1, 16)) // Cleanup
    );

    bench_loop_entities!(
        group,
        "64",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 64)) // Registration so it doesn't affect the benchmark
        ; // Preparation
        ; (ensure_mut_component_range, (C, 1, 64)) // Benchmark
        ; (reset_component_range, (C, 1, 64)) // Cleanup
    );

    group.finish();
}

pub fn flecs_get_mut_components_not_found_and_create_cmd(criterion: &mut Criterion) {
    let mut group = create_group(
        criterion,
        "flecs_get_mut_components_not_found_and_create_cmd",
    );
    bench_loop_entities!(
        group,
        "1",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 1)) // Registration so it doesn't affect the benchmark
        ;  // Preparation
        ; (ensure_mut_component_range_cmd, (C, 1, 1)) // Benchmark
        ; (reset_component_range, (C, 1, 1)) // Cleanup
    );

    bench_loop_entities!(
        group,
        "16",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 16)) // Registration so it doesn't affect the benchmark
        ; // Preparation
        ; (ensure_mut_component_range_cmd, (C, 1, 16)) // Benchmark
        ; (reset_component_range, (C, 1, 16)) // Cleanup
    );

    bench_loop_entities!(
        group,
        "64",
        ENTITY_COUNT
        ; (register_component_range, (C, 1, 64)) // Registration so it doesn't affect the benchmark
        ; // Preparation
        ; (ensure_mut_component_range_cmd, (C, 1, 64)) // Benchmark
        ; (reset_component_range, (C, 1, 64)) // Cleanup
    );

    group.finish();
}

pub fn flecs_c_get_components(_criterion: &mut Criterion) {
    //let mut group = create_group(criterion, "flecs_c_get_component");

    // group.bench_function("not_empty_entity", |bencher| {
    //     unsafe {
    //         let world = ecs_mini();
    //         let entities = create_ids(world, ENTITY_COUNT as i32, 0, false);
    //         let ids = create_ids(world, 1, 4, true);

    //         bencher.iter_custom(|iters| {
    //             let start = Instant::now();
    //             for _ in 0..iters {
    //                 for entity in entities.iter().take(ENTITY_COUNT as usize) {
    //                     ecs_get_id(world, *entity, ids[0]);
    //                 }
    //             }
    //             start.elapsed() / ENTITY_COUNT // Return the total time per entity
    //         });
    //         ecs_fini(world);
    //     }
    // });

    // group.bench_function("not_entity", |bencher| {
    //     unsafe {
    //         let world = ecs_mini();
    //         let entities = create_ids(world, ENTITY_COUNT as i32, 0, false);
    //         let ids = create_ids(world, 2, 4, true);

    //         for i in 0..ENTITY_COUNT as usize {
    //             ecs_add_id(world, entities[i], ids[0]);
    //         }

    //         bencher.iter_custom(|iters| {
    //             let start = Instant::now();
    //             for _ in 0..iters {
    //                 for entity in entities.iter().take(ENTITY_COUNT as usize) {
    //                     ecs_get_id(world, *entity, ids[1]);
    //                 }
    //             }
    //             start.elapsed() / ENTITY_COUNT // Return the total time per entity
    //         });
    //         ecs_fini(world);
    //     }
    // });

    // group.bench_function("1", |bencher| {
    //     unsafe {
    //         let world = ecs_mini();
    //         let entities = create_ids(world, ENTITY_COUNT as i32, 0, false);
    //         let ids = create_ids(world, 1, 4, true);

    //         for entity in entities.iter().take(ENTITY_COUNT as usize) {
    //             ecs_add_id(world, *entity, ids[0]);
    //         }

    //         bencher.iter_custom(|iters| {
    //             let start = Instant::now();
    //             for _ in 0..iters {
    //                 for entity in entities.iter().take(ENTITY_COUNT as usize) {
    //                     ecs_get_id(world, *entity, ids[0]);
    //                 }
    //             }
    //             start.elapsed() / ENTITY_COUNT // Return the total time per entity
    //         });
    //         ecs_fini(world);
    //     }
    // });

    // group.bench_function("16", |bencher| {
    //     unsafe {
    //         let id_count = 16;
    //         let world = ecs_mini();
    //         let entities = create_ids(world, ENTITY_COUNT as i32, 0, false);
    //         let ids = create_ids(world, id_count, 4, true);

    //         for entity in entities.iter().take(ENTITY_COUNT as usize) {
    //             for id in ids.iter().take(id_count as usize) {
    //                 ecs_add_id(world, *entity, *id);
    //             }
    //         }

    //         bencher.iter_custom(|iters| {
    //             let start = Instant::now();
    //             for _ in 0..iters {
    //                 for entity in &entities {
    //                     for id in &ids {
    //                         ecs_get_id(world, *entity, *id);
    //                     }
    //                 }
    //             }
    //             start.elapsed() / ENTITY_COUNT // Return the total time per entity
    //         });
    //         ecs_fini(world);
    //     }
    // });
}

criterion_group!(
    benches,
    flecs_get_components_not_found,
    flecs_get_components_found,
    flecs_get_mut_components_found,
    flecs_get_mut_components_not_found_and_create,
);

criterion_main!(benches);
