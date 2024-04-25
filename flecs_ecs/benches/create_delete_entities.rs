#![allow(unused)]
include!("common.rs");
use common::*;

pub fn flecs_create_delete_entities(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_create_delete_entities");

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
            elapsed / (ENTITY_COUNT * 2) //time average per entity operation
        });
    });

    group.bench_function("empty_named", |bencher| {
        let world = World::new();

        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                for _ in 0..ENTITY_COUNT {
                    let entity = world.entity_named(c"hello");
                    entity.destruct();
                }
            }
            let elapsed = start.elapsed();
            elapsed / (ENTITY_COUNT * 2) //time average per entity operation
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
    let mut group = criterion.benchmark_group("flecs_create_delete_entities_cmd");

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

    group.bench_function("empty_named", |bencher| {
        let world = World::new();

        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                world.defer_begin();
                for _ in 0..ENTITY_COUNT {
                    let entity = world.entity_named(c"hello");
                    entity.destruct();
                }
                world.defer_end();
            }
            let elapsed = start.elapsed();
            elapsed / (ENTITY_COUNT * 2) //time average per entity operation
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

pub fn flecs_create_delete_entities_w_tree(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_create_delete_entities_w_tree");

    for width in [1, 10, 100, 1000] {
        for depth in [1, 10, 100, 1000] {
            group.bench_function(
                format!("w{}_d{}", width.to_string(), depth.to_string()),
                |bencher| {
                    let world = World::new();

                    let mut entities = Vec::with_capacity(depth as usize);

                    for _ in 0..depth {
                        let entity = world.entity();
                        entities.push(entity);
                    }

                    let id = world.entity();
                    let add_id = world.entity();

                    world
                        .observer::<()>()
                        .with_id(id)
                        .add_event::<flecs::OnAdd>()
                        .each_entity(|entity, ()| {
                            entity.add_id(add_id);
                        });

                    bencher.iter_custom(|iters| {
                        let start = Instant::now();
                        for _ in 0..iters {
                            let root = world.entity();
                            let mut cur = root;

                            for _ in 0..depth {
                                for _ in 0..width - 1 {
                                    let child = world.entity();
                                    child.child_of_id(cur);
                                }
                                let child = world.entity();
                                child.child_of_id(cur);
                                cur = child;
                            }

                            root.destruct();
                        }
                        let elapsed = start.elapsed();
                        elapsed / width // calculate overhead per child
                    });
                },
            );
        }
    }
    group.finish();
}
