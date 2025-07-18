use crate::{common_bench::*, create_typed_query};
use core::ffi::c_void;
use core::hint::black_box;
use flecs_ecs::core::term::internals;
use flecs_ecs::sys;

fn bench_query_iter_tags(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    cache_kind: QueryCacheKind,
    id_count: usize,
    term_count: usize,
    sparse: bool,
    fragment: bool,
) {
    group.bench_function(format!("query_iter_{name}"), |bencher| {
        reset_srand();
        let world = World::new();

        let e = world.entity();

        let ids = get_component_ids!(&world, id_count, false, sparse, fragment);

        let mut query = world.query::<()>();

        query.set_cache_kind(cache_kind);

        for id in &ids[..term_count] {
            query.with(*id).self_().set_in();
        }

        let query_desc = internals::QueryConfig::query_desc_mut(&mut query);
        // Create 100 other queries to increase cache element fragmentation
        if cache_kind == QueryCacheKind::Auto {
            for i in 0..100 {
                unsafe { world.query_from_desc::<()>(query_desc).build() };
            }
        }

        let f = unsafe { world.query_from_desc::<()>(query_desc).build() };

        let mut result: u32 = 0;

        for _ in 0..QUERY_ENTITY_COUNT {
            let e = world.entity();
            for id in &ids {
                if flip_coin() {
                    unsafe { e.add_id_unchecked(*id) };
                }
            }
        }

        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                f.run(|mut it| {
                    while it.next() {
                        for i in it.iter() {
                            result += *it.entity_id(i) as u32;
                        }
                    }
                });
            }
            start.elapsed()
        });
        core::hint::black_box(result);
    });
}

fn bench_query_iter_components_setup(
    world: &World,
    cache_kind: QueryCacheKind,
    id_count: usize,
    term_count: usize,
    sparse: bool,
    fragment: bool,
) -> Query<()> {
    reset_srand();
    let e = world.entity();

    let ids = get_component_ids!(&world, id_count, true, sparse, fragment);

    let mut query = world.query::<()>();

    query.set_cache_kind(cache_kind);

    for id in &ids[..term_count] {
        query.with(*id).self_().set_in();
    }

    let f = query.build();

    // let query_desc = internals::QueryConfig::query_desc_mut(&mut query);
    // // Create 100 other queries to increase cache element fragmentation
    // if cache_kind == QueryCacheKind::Auto {
    //     for i in 0..100 {
    //         world.query_from_desc::<()>(query_desc).build();
    //     }
    // }

    //let mut f = world.query_from_desc::<()>(query_desc);

    for _ in 0..QUERY_ENTITY_COUNT {
        let e = world.entity();
        for (i, id) in ids.iter().enumerate() {
            if flip_coin() {
                unsafe {
                    sys::ecs_set_id(
                        world.ptr_mut(),
                        *e.id(),
                        *id,
                        4,
                        &((i + 1) as u32) as *const u32 as *const c_void,
                    );
                };
            }
        }
    }

    f
}

fn bench_query_iter_components_1_term(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    cache_kind: QueryCacheKind,
    id_count: usize,
    sparse: bool,
    fragment: bool,
) {
    group.bench_function(format!("query_iter_{name}"), |bencher| {
        let world = World::new();

        let mut f =
            bench_query_iter_components_setup(&world, cache_kind, id_count, 1, sparse, fragment);

        let f = unsafe { core::mem::transmute::<Query<()>, Query<(&C1,)>>(f) };

        let mut result: u32 = 0;

        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                f.each_entity(|e, (c1,)| {
                    result += *e.id() as u32 + c1.0;
                });
            }
            start.elapsed()
        });

        core::hint::black_box(result);
    });
}

fn bench_query_iter_components_4_term(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    cache_kind: QueryCacheKind,
    id_count: usize,
    sparse: bool,
    fragment: bool,
) {
    group.bench_function(format!("query_iter_{name}"), |bencher| {
        let world = World::new();

        let mut f =
            bench_query_iter_components_setup(&world, cache_kind, id_count, 4, sparse, fragment);

        let f = unsafe { core::mem::transmute::<Query<()>, Query<(&C1, &C2, &C3, &C4)>>(f) };

        let mut result: u32 = 0;

        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                result = 0;
                f.each_entity(|e, (c1, c2, c3, c4)| {
                    result += *e.id() as u32 + c1.0 + c2.0 + c3.0 + c4.0;
                });
            }
            start.elapsed()
        });

        core::hint::black_box(result);
    });
}

fn bench_query_iter_components_4_term_run(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    cache_kind: QueryCacheKind,
    id_count: usize,
    sparse: bool,
    fragment: bool,
) {
    group.bench_function(format!("query_iter_{name}"), |bencher| {
        let world = World::new();

        let mut f =
            bench_query_iter_components_setup(&world, cache_kind, id_count, 4, sparse, fragment);

        let f = unsafe { core::mem::transmute::<Query<()>, Query<(&C1, &C2, &C3, &C4)>>(f) };

        let mut result: u32 = 0;

        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                f.run(|mut it| {
                    while it.next() {
                        let c1 = it.field::<C1>(0).unwrap();
                        let c2 = it.field::<C2>(1).unwrap();
                        let c3 = it.field::<C3>(2).unwrap();
                        let c4 = it.field::<C4>(3).unwrap();
                        for i in it.iter() {
                            result +=
                                *it.entity_id(i) as u32 + c1[i].0 + c2[i].0 + c3[i].0 + c4[i].0;
                        }
                    }
                });
            }
            start.elapsed()
        });

        core::hint::black_box(result);
    });
}

fn bench_query_iter_components_8_term(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    cache_kind: QueryCacheKind,
    id_count: usize,
    sparse: bool,
    fragment: bool,
) {
    group.bench_function(format!("query_iter_{name}"), |bencher| {
        let world = World::new();

        let mut f =
            bench_query_iter_components_setup(&world, cache_kind, id_count, 8, sparse, fragment);

        let f = unsafe {
            core::mem::transmute::<Query<()>, Query<(&C1, &C2, &C3, &C4, &C5, &C6, &C7, &C8)>>(f)
        };

        let mut result: u32 = 0;

        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                f.each_entity(|e, (c1, c2, c3, c4, c5, c6, c7, c8)| {
                    result +=
                        *e.id() as u32 + c1.0 + c2.0 + c3.0 + c4.0 + c5.0 + c6.0 + c7.0 + c8.0;
                });
            }
            start.elapsed()
        });

        core::hint::black_box(result);
    });
}

struct BenchmarkConfig {
    name: &'static str,
    cache_kind: QueryCacheKind,
    id_count: usize,
    term_count: usize,
    sparse: bool,
    fragment: bool,
    component_benchmark: bool, // true for components, false for tags
}

pub fn query_iter(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("flecs");

    #[rustfmt::skip]
    let benchmarks = [

       // Uncached query iter
    //    BenchmarkConfig { name: "uncach_6_tags_1_term", cache_kind: QueryCacheKind::Default, id_count: 6, term_count: 1, sparse: false, fragment: true, component_benchmark: false },
    //    BenchmarkConfig { name: "uncach_6_tags_4_terms", cache_kind: QueryCacheKind::Default, id_count: 6, term_count: 4, sparse: false, fragment: true, component_benchmark: false },
    //    BenchmarkConfig { name: "uncach_10_tags_1_term", cache_kind: QueryCacheKind::Default, id_count: 10, term_count: 1, sparse: false, fragment: true, component_benchmark: false },
    //    BenchmarkConfig { name: "uncach_10_tags_4_terms", cache_kind: QueryCacheKind::Default, id_count: 10, term_count: 4, sparse: false, fragment: true, component_benchmark: false },
    //    BenchmarkConfig { name: "uncach_10_tags_8_terms", cache_kind: QueryCacheKind::Default, id_count: 10, term_count: 8, sparse: false, fragment: true, component_benchmark: false },
    //    BenchmarkConfig { name: "uncach_6_comps_1_term", cache_kind: QueryCacheKind::Default, id_count: 6, term_count: 1, sparse: false, fragment: true, component_benchmark: true },
    //    BenchmarkConfig { name: "uncach_6_comps_4_terms", cache_kind: QueryCacheKind::Default, id_count: 6, term_count: 4, sparse: false, fragment: true, component_benchmark: true },
    //    BenchmarkConfig { name: "uncach_10_comps_1_term", cache_kind: QueryCacheKind::Default, id_count: 10, term_count: 1, sparse: false, fragment: true, component_benchmark: true },
    //    BenchmarkConfig { name: "uncach_10_comps_4_terms", cache_kind: QueryCacheKind::Default, id_count: 10, term_count: 4, sparse: false, fragment: true, component_benchmark: true },
    //    BenchmarkConfig { name: "uncach_10_comps_8_terms", cache_kind: QueryCacheKind::Default, id_count: 10, term_count: 8, sparse: false, fragment: true, component_benchmark: true },

    //    BenchmarkConfig { name: "uncach_10_sparse_tags_4_terms", cache_kind: QueryCacheKind::Default, id_count: 10, term_count: 4, sparse: true, fragment: true, component_benchmark: false },
    //    BenchmarkConfig { name: "uncach_10_sparse_comps_4_terms", cache_kind: QueryCacheKind::Default, id_count: 10, term_count: 4, sparse: true, fragment: true, component_benchmark: true },
    //    BenchmarkConfig { name: "uncach_10_nofrag_tags_4_terms", cache_kind: QueryCacheKind::Default, id_count: 10, term_count: 4, sparse: true, fragment: false, component_benchmark: false },
    //    BenchmarkConfig { name: "uncach_10_nofrag_comps_4_terms", cache_kind: QueryCacheKind::Default, id_count: 10, term_count: 4, sparse: true, fragment: false, component_benchmark: true },

       // Cached query iter
    //    BenchmarkConfig { name: "cached_6_tags_1_term", cache_kind: QueryCacheKind::Auto, id_count: 6, term_count: 1, sparse: false, fragment: true, component_benchmark: false },
    //    BenchmarkConfig { name: "cached_6_tags_4_terms", cache_kind: QueryCacheKind::Auto, id_count: 6, term_count: 4, sparse: false, fragment: true, component_benchmark: false },
    //    BenchmarkConfig { name: "cached_8_tags_1_term", cache_kind: QueryCacheKind::Auto, id_count: 8, term_count: 1, sparse: false, fragment: true, component_benchmark: false },
    //    BenchmarkConfig { name: "cached_8_tags_4_terms", cache_kind: QueryCacheKind::Auto, id_count: 8, term_count: 4, sparse: false, fragment: true, component_benchmark: false },
    //    BenchmarkConfig { name: "cached_10_tags_1_term", cache_kind: QueryCacheKind::Auto, id_count: 10, term_count: 1, sparse: false, fragment: true, component_benchmark: false },
    //    BenchmarkConfig { name: "cached_10_tags_4_terms", cache_kind: QueryCacheKind::Auto, id_count: 10, term_count: 4, sparse: false, fragment: true, component_benchmark: false },
    //    BenchmarkConfig { name: "cached_10_tags_8_terms", cache_kind: QueryCacheKind::Auto, id_count: 10, term_count: 8, sparse: false, fragment: true, component_benchmark: false },
    //    BenchmarkConfig { name: "cached_16_tags_1_term", cache_kind: QueryCacheKind::Auto, id_count: 16, term_count: 1, sparse: false, fragment: true, component_benchmark: false },
    //    BenchmarkConfig { name: "cached_16_tags_4_terms", cache_kind: QueryCacheKind::Auto, id_count: 16, term_count: 4, sparse: false, fragment: true, component_benchmark: false },
    //    BenchmarkConfig { name: "cached_16_tags_8_terms", cache_kind: QueryCacheKind::Auto, id_count: 16, term_count: 8, sparse: false, fragment: true, component_benchmark: false },

    //    BenchmarkConfig { name: "cached_6_comps_1_term", cache_kind: QueryCacheKind::Auto, id_count: 6, term_count: 1, sparse: false, fragment: true, component_benchmark: true },
    //    BenchmarkConfig { name: "cached_6_comps_4_terms", cache_kind: QueryCacheKind::Auto, id_count: 6, term_count: 4, sparse: false, fragment: true, component_benchmark: true },
    //    BenchmarkConfig { name: "cached_8_comps_1_term", cache_kind: QueryCacheKind::Auto, id_count: 8, term_count: 1, sparse: false, fragment: true, component_benchmark: true },
    //    BenchmarkConfig { name: "cached_8_comps_4_terms", cache_kind: QueryCacheKind::Auto, id_count: 8, term_count: 4, sparse: false, fragment: true, component_benchmark: true },
    //    BenchmarkConfig { name: "cached_10_comps_1_term", cache_kind: QueryCacheKind::Auto, id_count: 10, term_count: 1, sparse: false, fragment: true, component_benchmark: true },
       BenchmarkConfig { name: "cached_10_comps_4_terms", cache_kind: QueryCacheKind::Auto, id_count: 10, term_count: 4, sparse: false, fragment: true, component_benchmark: true },
       BenchmarkConfig { name: "cached_10_comps_8_terms", cache_kind: QueryCacheKind::Auto, id_count: 10, term_count: 8, sparse: false, fragment: true, component_benchmark: true },
    //    BenchmarkConfig { name: "cached_16_comps_1_term", cache_kind: QueryCacheKind::Auto, id_count: 16, term_count: 1, sparse: false, fragment: true, component_benchmark: true },
    //    BenchmarkConfig { name: "cached_16_comps_4_terms", cache_kind: QueryCacheKind::Auto, id_count: 16, term_count: 4, sparse: false, fragment: true, component_benchmark: true },
    //    BenchmarkConfig { name: "cached_16_comps_8_terms", cache_kind: QueryCacheKind::Auto, id_count: 16, term_count: 8, sparse: false, fragment: true, component_benchmark: true },

    //    BenchmarkConfig { name: "cached_10_sparse_comps_4_terms", cache_kind: QueryCacheKind::Auto, id_count: 10, term_count: 4, sparse: true, fragment: true, component_benchmark: true },
    //    BenchmarkConfig { name: "cached_10_nofrag_comps_4_terms", cache_kind: QueryCacheKind::Auto, id_count: 10, term_count: 4, sparse: true, fragment: false, component_benchmark: true },
    ];

    for benchmark in benchmarks {
        if benchmark.component_benchmark {
            match benchmark.term_count {
                1 => bench_query_iter_components_1_term(
                    &mut group,
                    benchmark.name,
                    benchmark.cache_kind,
                    benchmark.id_count,
                    benchmark.sparse,
                    benchmark.fragment,
                ),
                4 => bench_query_iter_components_4_term(
                    &mut group,
                    benchmark.name,
                    benchmark.cache_kind,
                    benchmark.id_count,
                    benchmark.sparse,
                    benchmark.fragment,
                ),
                8 => bench_query_iter_components_8_term(
                    &mut group,
                    benchmark.name,
                    benchmark.cache_kind,
                    benchmark.id_count,
                    benchmark.sparse,
                    benchmark.fragment,
                ),
                _ => panic!("Unsupported components count"),
            }
        } else {
            bench_query_iter_tags(
                &mut group,
                benchmark.name,
                benchmark.cache_kind,
                benchmark.id_count,
                benchmark.term_count,
                benchmark.sparse,
                benchmark.fragment,
            );
        }
    }

    // bench_query_iter_components_4_term_run(
    //     &mut group,
    //     "query_iter_read_run_4",
    //     QueryCacheKind::Default,
    //     6,
    //     false,
    //     true,
    // );
    // c_query_iter_read_4(&mut group, "c_query_iter_read_4", QueryCacheKind::Auto, 6);

    group.finish();
}

fn c_query_iter_read_4(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    cache_kind: QueryCacheKind,
    id_count: usize,
) {
    group.bench_function(format!("c_query_iter_{name}"), |bencher| {
        reset_srand();
        unsafe {
            let world_ref = World::new();
            let world = world_ref.ptr_mut();
            let ids = create_ids(&world_ref, id_count, 4, true, false, true);

            let mut desc: sys::ecs_query_desc_t = sys::ecs_query_desc_t {
                cache_kind: cache_kind as u32,
                ..Default::default()
            };

            for (i, id) in ids.iter().enumerate().take(4) {
                desc.terms[i].id = ids[i];
                desc.terms[i].flags_ = sys::EcsSelf as u16;
                desc.terms[i].inout = sys::ecs_inout_kind_t_EcsIn as i16;
            }

            let f = sys::ecs_query_init(world, &desc);
            let mut result: u32 = 0;

            for i in 0..QUERY_ENTITY_COUNT {
                let e = sys::ecs_new(world);
                for id in &ids {
                    if (flip_coin()) {
                        sys::ecs_set_id(world, e, *id, 4, &(i + 1) as *const u32 as *const c_void);
                    }
                }
            }

            bencher.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    let mut it: sys::ecs_iter_t = sys::ecs_query_iter(world, f);
                    // optimized next call that's selected automatically by c++/system api
                    while (sys::ecs_query_next(&mut it as *mut sys::ecs_iter_t)) {
                        let count = it.count as usize;
                        let c1 = sys::ecs_field_w_size(&it, 4, 0) as *const u32;
                        let c2 = sys::ecs_field_w_size(&it, 4, 1) as *const u32;
                        let c3 = sys::ecs_field_w_size(&it, 4, 2) as *const u32;
                        let c4 = sys::ecs_field_w_size(&it, 4, 3) as *const u32;
                        for i in 0..count {
                            result += *it.entities.add(i) as u32;
                            result += *c1.add(i);
                            result += *c2.add(i);
                            result += *c3.add(i);
                            result += *c4.add(i);
                        }
                    }
                }
                start.elapsed()
            });

            core::hint::black_box(result);

            sys::ecs_query_fini(f);
        }
    });
}
