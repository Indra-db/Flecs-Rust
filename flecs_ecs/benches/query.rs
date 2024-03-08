#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(warnings)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use flecs_ecs::{
    core::{
        c_types::{EntityT, IdT, WorldT},
        component_registration::*,
        query::Query,
        World,
    },
    macros::Component,
};
use seq_macro::seq;
use std::{ffi::CStr, sync::OnceLock};

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
    let world = World::new();

    for _ in 0..1000_000 {
        let mut e = world.new_entity();
        e.set(Pos { x: 10.0, y: 20.0 });
        e.set(Vel { x: 5.0, y: 5.0 });
        if flip_coin() {
            e.add::<X2>();
        }
        if flip_coin() {
            e.add::<X3>();
        }
        if flip_coin() {
            e.add::<X4>();
        }
        if flip_coin() {
            e.add::<X5>();
        }
        if flip_coin() {
            e.add::<X6>();
        }
        if flip_coin() {
            e.add::<X7>();
        }
        if flip_coin() {
            e.add::<X8>();
        }
        if flip_coin() {
            e.add::<X9>();
        }
        if flip_coin() {
            e.add::<X10>();
        }
        if flip_coin() {
            e.add::<X11>();
        }
    }

    let mut query = Query::<(&mut Pos, &Vel)>::new(&world);

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
