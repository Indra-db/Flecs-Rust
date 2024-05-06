use crate::common_bench::*;
use std::ffi::CString;

pub fn get_relationship_targets(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("get_relationship_targets");

    bench_get_relationship_target!(group, "1", ENTITY_COUNT, 1);
    bench_get_relationship_target!(group, "2", ENTITY_COUNT, 2);
    bench_get_relationship_target!(group, "16", ENTITY_COUNT, 16);
    bench_get_relationship_target!(group, "64", ENTITY_COUNT, 64);

    group.finish();
}

pub fn override_components_add_remove(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("is_a_base_components_override_add_remove");

    bench_add_remove_override!(group, "1", 1);
    bench_add_remove_override!(group, "2", 2);
    bench_add_remove_override!(group, "4", 4);
    bench_add_remove_override!(group, "8", 8);
    bench_add_remove_override!(group, "16", 16);
    bench_add_remove_override!(group, "32", 32);
    bench_add_remove_override!(group, "64", 64);

    group.finish();
}

pub fn get_inherited_w_depth(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("get_inherited_w_depth");

    let depth_vec = [1, 2, 16, 64];
    for depth in depth_vec.iter() {
        group.bench_function(depth.to_string(), |bencher| {
            let world = World::new();
            let entities = create_entities(&world, ENTITY_COUNT as usize);

            let mut base = world.entity().add::<C1>();
            for _ in 0..*depth {
                base = world.entity().add_first::<flecs::IsA>(base);
            }

            for entity in &entities {
                entity.is_a_id(base);
            }

            bencher.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    for entity in &entities {
                        let _c1 = entity.get::<C1>();
                    }
                }
                let elapsed = start.elapsed();
                elapsed / ENTITY_COUNT
            });

            reset_component_range!(C, 1, 1);
        });
    }

    group.finish();
}

pub fn change_parent(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("change_parent");

    group.bench_function("", |bencher| {
        let world = World::new();

        let p1 = world.entity();
        let p2 = world.entity();

        let mut entities = Vec::with_capacity(ENTITY_COUNT as usize);

        for _ in 0..ENTITY_COUNT {
            let entity = world.entity();
            entity.child_of_id(p2);
            entities.push(entity);
        }

        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                for entity in &entities {
                    entity.child_of_id(p1);
                }
                for entity in &entities {
                    entity.child_of_id(p2);
                }
            }
            let elapsed = start.elapsed();
            elapsed / (2 * ENTITY_COUNT) //time average per entity operation
        });
    });

    group.bench_function("w_name", |bencher| {
        let world = World::new();

        let p1 = world.entity();
        let p2 = world.entity();

        let mut entities = Vec::with_capacity(ENTITY_COUNT as usize);

        for i in 0..ENTITY_COUNT {
            let name = format!("child_{}", i);
            let name_c = std::ffi::CString::new(name).unwrap();
            let name_cstr = name_c.as_c_str();
            let entity = world.entity_named(name_cstr);
            entity.child_of_id(p2);
            entities.push(entity);
        }
        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                for entity in &entities {
                    entity.child_of_id(p1);
                }
                for entity in &entities {
                    entity.child_of_id(p2);
                }
            }
            let elapsed = start.elapsed();
            elapsed / (2 * ENTITY_COUNT) //time average per entity operation
        });
    });

    group.bench_function("from_root", |bencher| {
        let world = World::new();

        let p = world.entity();

        let mut entities = Vec::with_capacity(ENTITY_COUNT as usize);

        for _ in 0..ENTITY_COUNT {
            let entity = world.entity();
            entities.push(entity);
        }

        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                for entity in &entities {
                    entity.child_of_id(p);
                }
                for entity in &entities {
                    entity.remove_id((*flecs::ChildOf, p));
                }
            }
            let elapsed = start.elapsed();
            elapsed / (2 * ENTITY_COUNT) //time average per entity operation
        });
    });
}

pub fn lookup_depth(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("lookup_depth");

    for depth in [0, 1, 10, 100] {
        group.bench_function(depth.to_string(), |bencher| {
            let world = World::new();

            let mut lookup_str = String::from("foo");
            for _ in 0..depth {
                lookup_str = format!("{}.foo", lookup_str);
            }

            let mut e = world.entity_named(c"foo");
            for _ in 0..depth {
                let child = world.entity_named(c"foo").child_of_id(e);
                e = child;
            }

            let lookup_cstring = CString::new(lookup_str).unwrap();
            let lookup_cstr = lookup_cstring.as_c_str();

            bencher.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    world.try_lookup_recursive(lookup_cstr);
                }
                let elapsed = start.elapsed();
                elapsed / 1 //time average per entity operation
            });
        });
    }

    group.finish();
}
