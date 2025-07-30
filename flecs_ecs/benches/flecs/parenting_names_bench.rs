use crate::common_bench::*;
use crate::common_bench::*;

pub fn change_parent(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs");

    group.bench_function("change_parent", |bencher| {
        let world = World::new();

        let p1 = world.entity();
        let p2 = world.entity();

        let mut entities = Vec::with_capacity(ENTITY_COUNT as usize);

        for _ in 0..ENTITY_COUNT {
            entities.push(p2.child());
        }

        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                for entity in &entities {
                    entity.child_of(p1);
                }
                for entity in &entities {
                    entity.child_of(p2);
                }
            }
            let elapsed = start.elapsed();
            elapsed / (2 * ENTITY_COUNT) //time average per entity operation
        });
    });

    group.bench_function("change_parent_w_name", |bencher| {
        let world = World::new();

        let p1 = world.entity_named("parent_1");
        let p2 = world.entity_named("parent_2");

        let mut entities = Vec::with_capacity(ENTITY_COUNT as usize);

        for i in 0..ENTITY_COUNT {
            let name = format!("child_{i}");
            let name_str = name.as_str();
            let entity = world.entity_named(name_str);
            entity.child_of(p2);
            entities.push(entity);
        }
        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                for entity in &entities {
                    entity.child_of(p1);
                }
                for entity in &entities {
                    entity.child_of(p2);
                }
            }
            let elapsed = start.elapsed();
            elapsed / (2 * ENTITY_COUNT) //time average per entity operation
        });
    });

    group.bench_function("change_parent_from_root", |bencher| {
        let world = World::new();

        let p = world.entity();

        let mut entities = create_entities(&world, ENTITY_COUNT as usize);

        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                for entity in &entities {
                    entity.child_of(p);
                }
                for entity in &entities {
                    entity.remove((*flecs::ChildOf, p));
                }
            }
            let elapsed = start.elapsed();
            elapsed / (2 * ENTITY_COUNT) //time average per entity operation
        });
    });
}

pub fn lookup_depth(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs");

    for depth in [0, 1, 10, 100] {
        group.bench_function(format!("lookup_depth_{depth}"), |bencher| {
            let world = World::new();

            let mut lookup_string = String::from("foo");
            for _ in 0..depth {
                lookup_string = format!("{lookup_string}.foo");
            }

            let mut e = world.entity_named("foo");
            for _ in 0..depth {
                let child = world.entity().child_of(e);
                child.set_name("foo");
                e = child;
            }

            let lookup_str = lookup_string.as_str();

            bencher.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    world.try_lookup_recursive(lookup_str);
                }
                let elapsed = start.elapsed();
                elapsed / 1 //time average per entity operation
            });
        });
    }

    group.finish();
}

pub fn set_name(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs");

    group.bench_function("set_name", |bencher| {
        let world = World::new();

        let e = world.entity();

        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                e.set_name("foo");
                e.remove_name();
            }
            let elapsed = start.elapsed();
            elapsed / (2) //time average per entity operation
        });
    });
}
