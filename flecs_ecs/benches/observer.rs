#![allow(unused)]
include!("common.rs");
use common::*;

pub fn observer_create_w_add(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("observer_on_add_event_add_id_for");

    for entity_count in [100, 1000, 10_000, 50_000] {
        group.bench_function(
            format!("{} entities", entity_count.to_string()),
            |bencher| {
                let world = World::new();

                let mut entities = Vec::with_capacity(entity_count as usize);

                for _ in 0..entity_count {
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
                        for entity in &entities {
                            entity.add_id(id);
                            entity.remove_id(id);
                        }
                    }
                    let elapsed = start.elapsed();
                    elapsed / entity_count //time average per entity operation
                });
            },
        );
    }
}
