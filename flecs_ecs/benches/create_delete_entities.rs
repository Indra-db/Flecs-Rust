mod common;
use common::*;

pub fn flecs_create_delete_entities(criterion: &mut Criterion) {
    let mut group = create_group(criterion, "flecs_create_delete_entities");

    group.bench_function("empty", |bencher| {
        let world = World::new();

        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                for _ in 0..ENTITY_COUNT {
                    let entity = world.entity();
                    entity.destruct();
                }
            }
            let elapsed = start.elapsed();
            elapsed / 2 //time average per entity operation
        });
    });

    // tags
    bench_create_delete_entity!(group, "tag_1", ENTITY_COUNT, T, 1, 1);
    bench_create_delete_entity!(group, "tag_2", ENTITY_COUNT, T, 1, 2);
    bench_create_delete_entity!(group, "tag_16", ENTITY_COUNT, T, 1, 16);
    bench_create_delete_entity!(group, "tag_64", ENTITY_COUNT, T, 1, 64);
    // components
    bench_create_delete_entity!(group, "component_1", ENTITY_COUNT, C, 1, 1);
    bench_create_delete_entity!(group, "component_2", ENTITY_COUNT, C, 1, 2);
    bench_create_delete_entity!(group, "component_16", ENTITY_COUNT, C, 1, 16);
    bench_create_delete_entity!(group, "component_64", ENTITY_COUNT, C, 1, 64);

    group.finish();
}

pub fn flecs_create_delete_entities_cmd(criterion: &mut Criterion) {
    let mut group = create_group(criterion, "flecs_create_delete_entities_cmd");

    group.bench_function("empty", |bencher| {
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

    //tags
    bench_create_delete_entity_cmd!(group, "tag_1", ENTITY_COUNT, T, 1, 1);
    bench_create_delete_entity_cmd!(group, "tag_2", ENTITY_COUNT, T, 1, 2);
    bench_create_delete_entity_cmd!(group, "tag_16", ENTITY_COUNT, T, 1, 16);
    bench_create_delete_entity_cmd!(group, "tag_64", ENTITY_COUNT, T, 1, 64);
    // components
    bench_create_delete_entity_cmd!(group, "component_1", ENTITY_COUNT, C, 1, 1);
    bench_create_delete_entity_cmd!(group, "component_2", ENTITY_COUNT, C, 1, 2);
    bench_create_delete_entity_cmd!(group, "component_16", ENTITY_COUNT, C, 1, 16);
    bench_create_delete_entity_cmd!(group, "component_64", ENTITY_COUNT, C, 1, 64);

    group.finish();
}

criterion_group!(
    benches,
    flecs_create_delete_entities,
    flecs_create_delete_entities_cmd
);

criterion_main!(benches);
