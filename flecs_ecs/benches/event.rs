#![allow(unused)]
include!("common.rs");
use common::*;

pub fn flecs_event_emit(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_event_emit");

    for observer_count in [0, 1, 10, 100] {
        group.bench_function(format!("{}_observers", observer_count), |bencher| {
            let world = World::new();
            let e = world.entity().add::<T1>();
            let table = e.table().unwrap();
            let evt = world.entity();

            for _ in 0..observer_count {
                world
                    .observer::<()>()
                    .with::<&T1>()
                    .self_()
                    .add_event_id(evt.id())
                    .iter_only(|_| {});
            }

            bencher.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    EventBuilderUntyped::new(&world, evt)
                        .add::<T1>()
                        .set_table_to_emit(table, 0, 0)
                        .emit();
                }
                let elapsed = start.elapsed();
                elapsed / 1 //time average per entity operation
            });

            reset_component_range!(T, 1, 1);
        });
    }
}

pub fn flecs_event_emit_propagate(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_event_emit_propagate_depth");

    for depth in [0, 10, 100, 1000] {
        group.bench_function(depth.to_string(), |bencher| {
            let world = World::new();
            let root = world.entity().add::<T1>();
            let mut cur = root;
            let root_table = root.table().unwrap();
            let evt = world.entity();

            for _ in 0..depth {
                cur = world.entity().child_of_id(cur);
            }

            world
                .observer::<()>()
                .with::<&T1>()
                .parent()
                .add_event_id(evt.id())
                .iter_only(|_| {});

            bencher.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    EventBuilderUntyped::new(&world, evt)
                        .add::<T1>()
                        .set_table_to_emit(root_table, 0, 0)
                        .emit();
                }
                let elapsed = start.elapsed();
                elapsed / 1 //time average per entity operation
            });

            reset_component_range!(T, 1, 1);
        });
    }
}

pub fn flecs_event_emit_forward(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_event_emit_propagate");

    for depth in [1, 1000] {
        group.bench_function(format!("components_{}_depth_{}", 1, depth), |bencher| {
            let world = World::new();
            let root = world.entity();

            let mut cur = root;

            let ids = vec_of_ids!(world, T, 1, 1);

            for id in &ids {
                world
                    .observer::<()>()
                    .with_id(*id)
                    .add_event::<flecs::OnAdd>()
                    .add_event::<flecs::OnRemove>()
                    .iter_only(|_| {});
            }

            add_component_range!(world, root, T, 1, 1);

            for _ in 0..depth {
                cur = world.entity().child_of_id(cur);
            }

            let e = world.entity();
            bencher.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    e.child_of_id(cur);
                    e.remove_id((*flecs::ChildOf, cur));
                }
                let elapsed = start.elapsed();
                elapsed / 1 //time average per entity operation
            });

            reset_component_range!(T, 1, 1);
        });
    }

    for depth in [1, 1000] {
        group.bench_function(format!("components_{}_depth_{}", 16, depth), |bencher| {
            let world = World::new();
            let root = world.entity();

            let mut cur = root;

            let ids = vec_of_ids!(world, T, 1, 16);

            for id in &ids {
                world
                    .observer::<()>()
                    .with_id(*id)
                    .add_event::<flecs::OnAdd>()
                    .add_event::<flecs::OnRemove>()
                    .iter_only(|_| {});
            }

            add_component_range!(world, root, T, 1, 16);

            for _ in 0..depth {
                cur = world.entity().child_of_id(cur);
            }

            let e = world.entity();
            bencher.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    e.child_of_id(cur);
                    e.remove_id((*flecs::ChildOf, cur));
                }
                let elapsed = start.elapsed();
                elapsed / 1 //time average per entity operation
            });

            reset_component_range!(T, 1, 16);
        });
    }
}

pub fn flecs_event_modified(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_event_modified");

    for observer_count in [0, 1, 10, 100] {
        group.bench_function(format!("{}_observers", observer_count), |bencher| {
            let world = World::new();
            let e = world.entity().add::<C1>();
            let table = e.table().unwrap();
            let evt = world.entity();

            for _ in 0..observer_count {
                world
                    .observer::<()>()
                    .with::<&C1>()
                    .self_()
                    .add_event::<flecs::OnSet>()
                    .iter_only(|_| {});
            }

            let id = *world.entity_from::<C1>();
            bencher.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    e.modified_id(id);
                }
                let elapsed = start.elapsed();
                elapsed / 1 //time average per entity operation
            });

            reset_component_range!(T, 1, 1);
        });
    }
}
