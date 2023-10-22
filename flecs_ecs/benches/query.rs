#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(warnings)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use flecs_ecs_bridge::core::c_types::{EntityT, IdT, WorldT};
use flecs_ecs_bridge::core::component_registration::*;
use flecs_ecs_bridge::core::query::Query;
use flecs_ecs_bridge::core::{utility::functions::*, world::World};
use flecs_ecs_bridge_derive::Component;
use seq_macro::seq;
use std::sync::OnceLock;

#[derive(Debug, Default, Component, Clone)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Default, Component, Clone)]
pub struct Vel {
    pub x: f32,
    pub y: f32,
}

seq!(P in 0..=20 {
    // expands to structs named x0, x1, x2, ..., 20
    #[derive(Debug, Default, Clone, Component)]
    struct X~P
    {
        x: f32,
        y: f32,
    }
});

fn flip_coin() -> bool {
    rand::random::<bool>()
}

fn query_each_benchmark(c: &mut Criterion) {
    // Setup world and entities
    let world = World::default();

    for _ in 0..1000_000 {
        let mut e = world.new_entity();
        e.set_component(Pos { x: 10.0, y: 20.0 });
        e.set_component(Vel { x: 5.0, y: 5.0 });
        if flip_coin() {
            e.add_component::<X2>();
        }
        if flip_coin() {
            e.add_component::<X3>();
        }
        if flip_coin() {
            e.add_component::<X4>();
        }
        if flip_coin() {
            e.add_component::<X5>();
        }
        if flip_coin() {
            e.add_component::<X6>();
        }
        if flip_coin() {
            e.add_component::<X7>();
        }
        if flip_coin() {
            e.add_component::<X8>();
        }
        if flip_coin() {
            e.add_component::<X9>();
        }
        if flip_coin() {
            e.add_component::<X10>();
        }
        if flip_coin() {
            e.add_component::<X11>();
        }
    }

    let mut query = Query::<(Pos, Vel)>::new(&world);

    c.bench_function("query_each", |b| {
        b.iter(|| {
            let mut counter = 0;
            query.each(|(pos, vel)| {
                counter += 1;
                pos.x += vel.x;
                pos.y += vel.y;
            });

            // This will make sure the benchmarked code isn't optimized away
            black_box(counter);
        });
    });
}

criterion_group!(benches, query_each_benchmark);
criterion_main!(benches);