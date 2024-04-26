#![allow(unused)]
include!("common.rs");
use common::*;

pub fn entity_set_name(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_entity_set_name");

    group.bench_function("", |bencher| {
        let world = World::new();

        let e = world.entity();

        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                e.set_name(c"foo");
                e.remove_name();
            }
            let elapsed = start.elapsed();
            elapsed / (2) //time average per entity operation
        });
    });
}
