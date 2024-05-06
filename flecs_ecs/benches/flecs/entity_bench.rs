use crate::common_bench::*;

pub fn entity(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs_entity");

    group.bench_function("set_remove_name", |bencher| {
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
