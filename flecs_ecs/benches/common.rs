#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(warnings)]

pub use std::time::{Duration, Instant};

pub use criterion::{black_box, criterion_group, criterion_main, Criterion};
pub use flecs_ecs::core::*;
pub use flecs_ecs_derive::Component;
pub use flecs_ecs_sys::*;
pub use seq_macro::seq;

pub const ENTITY_COUNT: u32 = 1000;

seq!(P in 0..=64 {
    #[derive(Debug, Default, Clone, Component)]
    pub struct C~P(pub f32);

    #[derive(Debug, Default, Clone, Component)]
    pub struct T~P;
});

#[derive(Debug, Default, Component, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Default, Component, Clone)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

pub fn create_group(
    criterion: &mut Criterion,
    name: impl Into<String>,
) -> criterion::BenchmarkGroup<'_, criterion::measurement::WallTime> {
    let mut group = criterion.benchmark_group(name);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));
    group.sample_size(50);
    group.noise_threshold(0.01);
    group.confidence_level(0.95);
    group.significance_level(0.05);
    group
}

pub fn flip_coin() -> bool {
    rand::random::<bool>()
}

fn criterion_config() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_secs_f32(0.5))
        .measurement_time(Duration::from_secs(1))
        .sample_size(50)
}

pub fn create_entities(world: &World, count: usize) -> Vec<EntityView> {
    let mut vec = Vec::<EntityView>::with_capacity(count);
    for _ in 0..count {
        let entity = world.entity();
        vec.push(entity);
    }
    vec
}

/// this function is for C benchmarks
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn create_ids(world: *mut ecs_world_t, count: i32, size: ecs_size_t, low: bool) -> Vec<u64> {
    if count > 0 {
        let mut vec = Vec::<ecs_entity_t>::with_capacity(count as usize);
        unsafe {
            for i in 0..count as usize {
                if low {
                    vec.push(ecs_new_low_id(world));
                } else {
                    vec.push(ecs_new(world));
                }
                if size > 0 {
                    ecs_set_id(
                        world,
                        vec[i],
                        FLECS_IDEcsComponentID_,
                        std::mem::size_of::<EcsComponent>(),
                        &EcsComponent {
                            size: size as i32,
                            alignment: 4,
                        } as *const EcsComponent as *const std::ffi::c_void,
                    );
                }
            }
        }
        return vec;
    } else {
        return Vec::new();
    }
}

/// this is a helper function that performs a reset on previously registered component data that is cached for the world.
/// this is useful for benchmarks because in each iteration the world is reset to a clean state, but the static
/// component data is not reset which would cause problems in single world mode.
/// This function is called at the end of each iteration to do this reset.
#[inline(always)]
pub fn reset_component<T: ComponentId>() {
    #[cfg(feature = "flecs_single_world_application")]
    T::__reset_one_lock_data();
}

#[cfg(feature = "flecs_single_world_application")]
macro_rules! reset_components {
    ($($type:ty),*) => {{
        $(
            {
                reset_component::<$type>();
            }
        )*
    }};
}

#[cfg(not(feature = "flecs_single_world_application"))]
macro_rules! reset_components {
    ($($type:ty),*) => {};
}

#[cfg(not(feature = "flecs_single_world_application"))]
macro_rules! reset_component_range {
    ($component:ty, $start:expr, $end:expr) => {};
}

macro_rules! reset_component_range {
    ($component:ty, $start:expr, $end:expr) => {{
        seq!(P in $start..=$end {
            reset_component::<$component~P>();
        });
    }}
}

macro_rules! add_component_range {
    ($world:expr, $entity:expr, $component:ty, $start:expr, $end:expr) => {{
        seq!(P in $start..=$end {
            $entity.add::<$component~P>();
    });
    }};
}

macro_rules! add_component_range_cmd {
    ($world:expr, $entity:expr, $component:ty, $start:expr, $end:expr) => {{
        $world.defer_begin();
        add_component_range!($world, $entity, $component, $start, $end);
        $world.defer_end();
    }};
}

macro_rules! add_relationship_targets {
    ($entity:expr, $amount:expr) => {{
        seq!(P in 1..=$amount {
            $entity.add::<(C1, T~P)>();
    });
    }};
}

macro_rules! add_component_on_add_hook {
    ($world:expr, $amount:expr) => {{
        seq!(P in 1..=$amount {
            $world.component::<C~P>().on_add(|_, _| {});
    });
    }};
}

macro_rules! add_component_on_remove_hook {
    ($world:expr, $amount:expr) => {{
        seq!(P in 1..=$amount {
            $world.component::<C~P>().on_remove(|_, _| {});
    });
    }};
}

macro_rules! set_component_range {
    ($world:expr, $entity:expr, $component:ty, $start:expr, $end:expr) => {{
        seq!(P in $start..=$end {
            $entity.set::<$component~P>($component~P(0.0));
    });
    }};
}

macro_rules! set_component_range_cmd {
    ($world:expr, $entity:expr, $component:ty, $start:expr, $end:expr) => {{
        $world.defer_begin();
        set_component_range!($world, $entity, $component, $start, $end);
        $world.defer_end();
    }};
}

macro_rules! remove_component_range {
    ($world:expr, $entity:expr, $component:ty, $start:expr, $end:expr) => {{
        seq!(P in $start..=$end {
            $entity.remove::<$component~P>();
    });
    }};
}

macro_rules! remove_component_range_cmd {
    ($world:expr, $entity:expr, $component:ty, $start:expr, $end:expr) => {{
        $world.defer_begin();
        remove_component_range!($world, $entity, $component, $start, $end);
        $world.defer_end();
    }};
}

macro_rules! has_component_range {
    ($world:expr, $entity:expr, $component:ty, $start:expr, $end:expr) => {{
        seq!(P in $start..=$end {
            let _ = $entity.has::<$component~P>();
    });
    }};
}

macro_rules! get_component_range {
    ($world:expr, $entity:expr, $component:ty, $start:expr, $end:expr) => {{
        seq!(P in $start..=$end {
            let _ = $entity.try_get::<$component~P>();
    });
    }};
}

macro_rules! get_mut_component_range {
    ($world:expr, $entity:expr, $component:ty, $start:expr, $end:expr) => {{
        seq!(P in $start..=$end {
            let _ = $entity.try_get_mut::<$component~P>();
    });
    }};
}

macro_rules! get_mut_component_range_cmd {
    ($world:expr, $entity:expr, $component:ty, $start:expr, $end:expr) => {{
        $world.defer_begin();
        get_mut_component_range!($world, $entity, $component, $start, $end);
        $world.defer_end();
    }};
}

macro_rules! ensure_mut_component_range {
    ($world:expr, $entity:expr, $component:ty, $start:expr, $end:expr) => {{
        seq!(P in $start..=$end {
            let _ = $entity.ensure_mut::<$component~P>();
    });
    }};
}

macro_rules! ensure_mut_component_range_cmd {
    ($world:expr, $entity:expr, $component:ty, $start:expr, $end:expr) => {{
        $world.defer_begin();
        ensure_mut_component_range!($world, $entity, $component, $start, $end);
        $world.defer_end();
    }};
}

macro_rules! register_component_range {
    ($world:expr, $component:ty, $start:expr, $end:expr) => {{
        seq!(P in $start..=$end {
            <$component~P as ComponentId>::get_id(&$world);
        });
    }};
}

macro_rules! bench_loop_entities {

    ($group:expr,$name:literal,$entity_count:expr
    ; $(($registration:ident,($r_type:ty, $r_start:expr, $r_end:expr))),*
    ; $(($preparation:ident,($p_type:ty, $p_start:expr, $p_end:expr))),*
    ; $(($benchmark:ident,($b_type:ty, $b_start:expr, $b_end:expr))),*
    ; $(($cleanup:ident,($c_type:ty, $c_start:expr, $c_end:expr))),*)
    => {{
        $group.bench_function($name, |bencher| {
            let world = World::new();
            let mut entities = Vec::with_capacity($entity_count as usize);
            $(
                $registration!(world, $r_type, $r_start, $r_end);
            )*

            for _ in 0..$entity_count {
                let entity = world.entity();
                $(
                    $preparation!(world, entity, $p_type, $p_start, $p_end);
                )*
                entities.push(entity);
            }

            bencher.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    for e in &mut entities {
                        $(
                            $benchmark!(&world, e, $b_type, $b_start, $b_end);
                        )*
                    }
                }
                let elapsed = start.elapsed();
                let mut operations : u32 = 0;
                let mut count : u32 = 0;
                 $(
                    {

                        count += 1;
                        operations += ($b_end - $b_start + 1) as u32;
                    };
                )*
                elapsed / (operations * $entity_count * count) //time average per entity operation
            });
            $(
                $cleanup!($c_type, $c_start, $c_end);
            )*
        });
    }};
}

macro_rules! bench_create_delete_entity {

    ($group:expr,$name:literal,$entity_count:expr
    ,$ttype:ty, $start:expr, $end:expr)
    => {{
        $group.bench_function($name, |bencher| {
            let world = World::new();

            register_component_range!(world, $ttype, $start, $end);

            bencher.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    for _ in 0..$entity_count {
                        let entity = world.entity();
                        add_component_range!(world, entity, $ttype, $start, $end);
                        entity.destruct();
                    }
                }
                let elapsed = start.elapsed();
                elapsed / (2 + (($end - $start + 1) as u32) * $entity_count) //time average per entity operation
            });
            reset_component_range!($ttype, $start, $end);
        });
    }};
}

macro_rules! bench_create_delete_entity_cmd {

    ($group:expr,$name:literal,$entity_count:expr
    ,$ttype:ty, $start:expr, $end:expr)
    => {{
        $group.bench_function($name, |bencher| {
            let world = World::new();

            register_component_range!(world, $ttype, $start, $end);

            bencher.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    world.defer_begin();
                    for _ in 0..$entity_count {
                        let entity = world.entity();
                        add_component_range!(world, entity, $ttype, $start, $end);
                        entity.destruct();
                    }
                    world.defer_end();
                }
                let elapsed = start.elapsed();
                elapsed / (2 + (($end - $start + 1) as u32) * $entity_count) //time average per entity operation
            });
            reset_component_range!($ttype, $start, $end);
        });
    }};
}

macro_rules! bench_get_relationship_target {
    ($group:expr,$name:literal,$entity_count:expr,$target_count:expr) => {{
        $group.bench_function($name, |bencher| {
            let world = World::new();
            let mut entities = Vec::<Entity>::with_capacity($entity_count as usize);

            for _ in 0..$entity_count {
                let entity = world.entity();
                add_relationship_targets!(entity, $target_count);
                entities.push(entity);
            }
            bencher.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    for entity in &entities {
                        for i in 0..$target_count {
                            let _ = entity.get_target::<C1>(i);
                        }
                    }
                }
                let elapsed = start.elapsed();
                elapsed / ($entity_count * $target_count) as u32
            });

            reset_component_range!(T, 1, $target_count);
            reset_component_range!(C, 1, 1);
        });
    }};
}

macro_rules! bench_add_remove_override {
    ($group:expr,$name:literal,$amount:expr) => {{
        $group.bench_function($name, |bencher| {
            let world = World::new();
            let entities = create_entities(&world, ENTITY_COUNT as usize);

            let base = world.entity();
            for _ in 0..$amount {
                add_component_range!(&world, base, C, 1, $amount);
            }

            for entity in &entities {
                entity.is_a_id(entity);
            }

            bencher.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    for entity in &entities {
                        add_component_range!(&world, entity, C, 1, $amount);
                        remove_component_range!(&world, entity, C, 1, $amount);
                    }
                }
                let elapsed = start.elapsed();
                elapsed / (2 * ENTITY_COUNT * $amount) as u32
            });

            reset_component_range!(C, 1, $amount);
        });
    }};
}

macro_rules! bench_add_remove_hooks {
    ($group:expr,$name:literal,$amount:expr) => {{
        $group.bench_function($name, |bencher| {
            let world = World::new();
            let entities = create_entities(&world, ENTITY_COUNT as usize);

            add_component_on_add_hook!(world, $amount);
            add_component_on_remove_hook!(world, $amount);

            bencher.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    for entity in &entities {
                        add_component_range!(&world, entity, C, 1, $amount);
                        remove_component_range!(&world, entity, C, 1, $amount);
                    }
                }
                let elapsed = start.elapsed();
                elapsed / (2 * ENTITY_COUNT * $amount) as u32
            });

            reset_component_range!(C, 1, $amount);
        });
    }};
}

pub(crate) use add_component_on_add_hook;
pub(crate) use add_component_on_remove_hook;
pub(crate) use add_component_range;
pub(crate) use add_component_range_cmd;
pub(crate) use add_relationship_targets;
pub(crate) use bench_add_remove_hooks;
pub(crate) use bench_add_remove_override;
pub(crate) use bench_create_delete_entity;
pub(crate) use bench_create_delete_entity_cmd;
pub(crate) use bench_get_relationship_target;
pub(crate) use bench_loop_entities;
pub(crate) use ensure_mut_component_range;
pub(crate) use ensure_mut_component_range_cmd;
pub(crate) use get_component_range;
pub(crate) use get_mut_component_range;
pub(crate) use get_mut_component_range_cmd;
pub(crate) use has_component_range;
pub(crate) use register_component_range;
pub(crate) use remove_component_range;
pub(crate) use remove_component_range_cmd;
pub(crate) use reset_component_range;
pub(crate) use reset_components;
pub(crate) use set_component_range;
pub(crate) use set_component_range_cmd;
