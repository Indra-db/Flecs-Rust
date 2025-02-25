#![allow(dead_code)]
use flecs_ecs::core::*;
use flecs_ecs::macros::*;

#[derive(Component)]
struct Foo(u8);

#[derive(Component)]
struct Bar(u8);

#[derive(Component)]
struct A;

#[derive(Component)]
struct B;

#[test]
fn building_conflicting_queries_no_violations() {
    let world = World::new();
    // building queries that overlap without running them is ok
    let _read = query!(world, &Foo).build();
    let _write0 = query!(world, &mut Foo).build();
    let _write1 = query!(world, &mut Foo).build();
}

#[test]
fn running_conflicting_queries_no_violations() {
    let world = World::new();
    let read = query!(world, &Foo).build();
    let write0 = query!(world, &mut Foo).build();
    let write1 = query!(world, &mut Foo).build();

    // running queries that overlap individually is ok
    read.run(|iter| {
        iter.fini();
    });
    write0.run(|iter| {
        iter.fini();
    });
    write1.run(|iter| {
        iter.fini();
    });

    read.each(|_| {});
    write0.each(|_| {});
    write1.each(|_| {});

    read.each_entity(|_, _| {});
    write0.each_entity(|_, _| {});
    write1.each_entity(|_, _| {});

    read.each_iter(|_, _, _| {});
    write0.each_iter(|_, _, _| {});
    write1.each_iter(|_, _, _| {});
}

mod query_in_query {
    use super::*;

    mod read_write {
        use super::*;

        #[test]
        #[should_panic]
        fn run_violation() {
            let world = World::new();
            let query0 = query!(world, &Foo).build();
            let query1 = query!(world, &mut Foo).build();
            query0.run(|iter| {
                query1.run(|iter| {
                    iter.fini();
                });
                iter.fini();
            });
        }

        #[test]
        #[should_panic]
        fn each_violation() {
            let world = World::new();
            let query0 = query!(world, &Foo).build();
            let query1 = query!(world, &mut Foo).build();
            query0.each(|_| {
                query1.each(|_| {});
            });
        }

        #[test]
        #[should_panic]
        fn each_entity_violation() {
            let world = World::new();
            let query0 = query!(world, &Foo).build();
            let query1 = query!(world, &mut Foo).build();
            query0.each_entity(|_, _| {
                query1.each_entity(|_, _| {});
            });
        }

        #[test]
        #[should_panic]
        fn each_iter_query_violation() {
            let world = World::new();
            let query0 = query!(world, &Foo).build();
            let query1 = query!(world, &mut Foo).build();
            query0.each_iter(|_, _, _| {
                query1.each_iter(|_, _, _| {});
            });
        }
    }

    mod write_read {
        use super::*;

        #[test]
        #[should_panic]
        fn run_violation() {
            let world = World::new();
            let query0 = query!(world, &mut Foo).build();
            let query1 = query!(world, &Foo).build();
            query0.run(|iter| {
                query1.run(|iter| {
                    iter.fini();
                });
                iter.fini();
            });
        }

        #[test]
        #[should_panic]
        fn each_violation() {
            let world = World::new();
            let query0 = query!(world, &mut Foo).build();
            let query1 = query!(world, &Foo).build();
            query0.each(|_| {
                query1.each(|_| {});
            });
        }

        #[test]
        #[should_panic]
        fn each_entity_violation() {
            let world = World::new();
            let query0 = query!(world, &mut Foo).build();
            let query1 = query!(world, &Foo).build();
            query0.each_entity(|_, _| {
                query1.each_entity(|_, _| {});
            });
        }

        #[test]
        #[should_panic]
        fn each_iter_violation() {
            let world = World::new();
            let query0 = query!(world, &mut Foo).build();
            let query1 = query!(world, &Foo).build();
            query0.each_iter(|_, _, _| {
                query1.each_iter(|_, _, _| {});
            });
        }
    }

    mod write_write {
        use super::*;

        #[test]
        #[should_panic]
        fn run_violation() {
            let world = World::new();
            let query0 = query!(world, &mut Foo).build();
            let query1 = query!(world, &mut Foo).build();
            query0.run(|iter| {
                query1.run(|iter| {
                    iter.fini();
                });
                iter.fini();
            });
        }

        #[test]
        #[should_panic]
        fn each_violation() {
            let world = World::new();
            let query0 = query!(world, &mut Foo).build();
            let query1 = query!(world, &mut Foo).build();
            query0.each(|_| {
                query1.each(|_| {});
            });
        }

        #[test]
        #[should_panic]
        fn each_entity_violation() {
            let world = World::new();
            let query0 = query!(world, &mut Foo).build();
            let query1 = query!(world, &mut Foo).build();
            query0.each_entity(|_, _| {
                query1.each_entity(|_, _| {});
            });
        }

        #[test]
        #[should_panic]
        fn each_iter_violation() {
            let world = World::new();
            let query0 = query!(world, &mut Foo).build();
            let query1 = query!(world, &mut Foo).build();
            query0.each_iter(|_, _, _| {
                query1.each_iter(|_, _, _| {});
            });
        }
    }
}

mod observer_in_observer {
    use super::*;

    mod read_write {
        use super::*;

        #[test]
        #[should_panic]
        fn run_violation() {
            let world = World::new();
            let e = world.entity().set(Foo(0)).id();
            observer!(world, A, &Foo).run(|iter| {
                iter.fini();
            });
            observer!(world, B, &mut Foo).run(move |iter| {
                iter.world().event().add::<Foo>().entity(e).emit(&A);
                iter.fini();
            });
            world.event().add::<Foo>().entity(e).emit(&B);
        }

        #[test]
        #[should_panic]
        fn each_entity_violation() {
            let world = World::new();
            let e = world.entity().set(Foo(0)).id();
            observer!(world, A, &Foo).each_entity(|_, _| {});
            observer!(world, B, &mut Foo).each_entity(move |entity, _| {
                entity.world().event().add::<Foo>().entity(e).emit(&A);
            });
            world.event().add::<Foo>().entity(e).emit(&B);
        }

        #[test]
        #[should_panic]
        fn each_iter_violation() {
            let world = World::new();
            let e = world.entity().set(Foo(0)).id();
            observer!(world, A, &Foo).each_iter(|_, _, _| {});
            observer!(world, B, &mut Foo).each_iter(move |iter, _, _| {
                iter.world().event().add::<Foo>().entity(e).emit(&A);
            });
            world.event().add::<Foo>().entity(e).emit(&B);
        }
    }

    mod write_read {
        use super::*;

        #[test]
        #[should_panic]
        fn run_violation() {
            let world = World::new();
            let e = world.entity().set(Foo(0)).id();
            observer!(world, A, &mut Foo).run(|iter| {
                iter.fini();
            });
            observer!(world, B, &Foo).run(move |iter| {
                iter.world().event().add::<Foo>().entity(e).emit(&A);
                iter.fini();
            });
            world.event().add::<Foo>().entity(e).emit(&B);
        }

        #[test]
        #[should_panic]
        fn each_entity_violation() {
            let world = World::new();
            let e = world.entity().set(Foo(0)).id();
            observer!(world, A, &mut Foo).each_entity(|_, _| {});
            observer!(world, B, &Foo).each_entity(move |entity, _| {
                entity.world().event().add::<Foo>().entity(e).emit(&A);
            });
            world.event().add::<Foo>().entity(e).emit(&B);
        }

        #[test]
        #[should_panic]
        fn each_iter_violation() {
            let world = World::new();
            let e = world.entity().set(Foo(0)).id();
            observer!(world, A, &mut Foo).each_iter(|_, _, _| {});
            observer!(world, B, &Foo).each_iter(move |iter, _, _| {
                iter.world().event().add::<Foo>().entity(e).emit(&A);
            });
            world.event().add::<Foo>().entity(e).emit(&B);
        }
    }

    mod write_write {
        use super::*;

        #[test]
        #[should_panic]
        fn run_violation() {
            let world = World::new();
            let e = world.entity().set(Foo(0)).id();
            observer!(world, A, &mut Foo).run(|iter| {
                iter.fini();
            });
            observer!(world, B, &mut Foo).run(move |iter| {
                iter.world().event().add::<Foo>().entity(e).emit(&A);
                iter.fini();
            });
            world.event().add::<Foo>().entity(e).emit(&B);
        }

        #[test]
        #[should_panic]
        fn each_entity_violation() {
            let world = World::new();
            let e = world.entity().set(Foo(0)).id();
            observer!(world, A, &mut Foo).each_entity(|_, _| {});
            observer!(world, B, &mut Foo).each_entity(move |entity, _| {
                entity.world().event().add::<Foo>().entity(e).emit(&A);
            });
            world.event().add::<Foo>().entity(e).emit(&B);
        }

        #[test]
        #[should_panic]
        fn each_iter_violation() {
            let world = World::new();
            let e = world.entity().set(Foo(0)).id();
            observer!(world, A, &mut Foo).each_iter(|_, _, _| {});
            observer!(world, B, &mut Foo).each_iter(move |iter, _, _| {
                iter.world().event().add::<Foo>().entity(e).emit(&A);
            });
            world.event().add::<Foo>().entity(e).emit(&B);
        }
    }
}

mod query_in_observer {
    use super::*;

    mod read_write {
        use super::*;

        #[test]
        #[should_panic]
        fn run_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            observer!(world, flecs::OnSet, &Foo).run(move |iter| {
                query.run(|iter| {
                    iter.fini();
                });
                iter.fini();
            });
            world.entity().set(Foo(0));
        }

        #[test]
        #[should_panic]
        fn each_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            observer!(world, flecs::OnSet, &Foo).each(move |_| {
                query.each(|_| {});
            });
            world.entity().set(Foo(0));
        }

        #[test]
        #[should_panic]
        fn each_entity_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            observer!(world, flecs::OnSet, &Foo).each_entity(move |_, _| {
                query.each_entity(|_, _| {});
            });
            world.entity().set(Foo(0));
        }

        #[test]
        #[should_panic]
        fn each_iter_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            observer!(world, flecs::OnSet, &Foo).each_iter(move |_, _, _| {
                query.each_iter(|_, _, _| {});
            });
            world.entity().set(Foo(0));
        }
    }

    mod write_read {
        use super::*;

        #[test]
        #[should_panic]
        fn run_violation() {
            let world = World::new();
            let query = query!(world, &Foo).build();
            observer!(world, flecs::OnSet, &mut Foo).run(move |iter| {
                query.run(|iter| {
                    iter.fini();
                });
                iter.fini();
            });
            world.entity().set(Foo(0));
        }

        #[test]
        #[should_panic]
        fn each_violation() {
            let world = World::new();
            let query = query!(world, &Foo).build();
            observer!(world, flecs::OnSet, &mut Foo).each(move |_| {
                query.each(|_| {});
            });
            world.entity().set(Foo(0));
        }

        #[test]
        #[should_panic]
        fn each_entity_violation() {
            let world = World::new();
            let query = query!(world, &Foo).build();
            observer!(world, flecs::OnSet, &mut Foo).each_entity(move |_, _| {
                query.each_entity(|_, _| {});
            });
            world.entity().set(Foo(0));
        }

        #[test]
        #[should_panic]
        fn each_iter_violation() {
            let world = World::new();
            let query = query!(world, &Foo).build();
            observer!(world, flecs::OnSet, &mut Foo).each_iter(move |_, _, _| {
                query.each_iter(|_, _, _| {});
            });
            world.entity().set(Foo(0));
        }
    }

    mod write_write {
        use super::*;

        #[test]
        #[should_panic]
        fn run_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            observer!(world, flecs::OnSet, &mut Foo).run(move |iter| {
                query.run(|iter| {
                    iter.fini();
                });
                iter.fini();
            });
            world.entity().set(Foo(0));
        }

        #[test]
        #[should_panic]
        fn each_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            observer!(world, flecs::OnSet, &mut Foo).each(move |_| {
                query.each(|_| {});
            });
            world.entity().set(Foo(0));
        }

        #[test]
        #[should_panic]
        fn each_entity_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            observer!(world, flecs::OnSet, &mut Foo).each_entity(move |_, _| {
                query.each_entity(|_, _| {});
            });
            world.entity().set(Foo(0));
        }

        #[test]
        #[should_panic]
        fn each_iter_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            observer!(world, flecs::OnSet, &mut Foo).each_iter(move |_, _, _| {
                query.each_iter(|_, _, _| {});
            });
            world.entity().set(Foo(0));
        }
    }
}

mod query_in_system {
    use super::*;

    mod read_write {
        use super::*;

        #[test]
        #[should_panic]
        fn run_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            system!(world, &Foo).run(move |iter| {
                query.run(|iter| {
                    iter.fini();
                });
                iter.fini();
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            system!(world, &Foo).each(move |_| {
                query.each(|_| {});
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_entity_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            system!(world, &Foo).each_entity(move |_, _| {
                query.each_entity(|_, _| {});
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_iter_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            system!(world, &Foo).each_iter(move |_, _, _| {
                query.each_iter(|_, _, _| {});
            });
            world.progress();
        }
    }

    mod write_read {
        use super::*;

        #[test]
        #[should_panic]
        fn run_violation() {
            let world = World::new();
            let query = query!(world, &Foo).build();
            system!(world, &mut Foo).run(move |iter| {
                query.run(|iter| {
                    iter.fini();
                });
                iter.fini();
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_violation() {
            let world = World::new();
            let query = query!(world, &Foo).build();
            system!(world, &mut Foo).each(move |_| {
                query.each(|_| {});
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_entity_violation() {
            let world = World::new();
            let query = query!(world, &Foo).build();
            system!(world, &mut Foo).each_entity(move |_, _| {
                query.each_entity(|_, _| {});
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_iter_violation() {
            let world = World::new();
            let query = query!(world, &Foo).build();
            system!(world, &mut Foo).each_iter(move |_, _, _| {
                query.each_iter(|_, _, _| {});
            });
            world.progress();
        }
    }

    mod write_write {
        use super::*;

        #[test]
        #[should_panic]
        fn run_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            system!(world, &mut Foo).run(move |iter| {
                query.run(|iter| {
                    iter.fini();
                });
                iter.fini();
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            system!(world, &mut Foo).each(move |_| {
                query.each(|_| {});
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_entity_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            system!(world, &mut Foo).each_entity(move |_, _| {
                query.each_entity(|_, _| {});
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_iter_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            system!(world, &mut Foo).each_iter(move |_, _, _| {
                query.each_iter(|_, _, _| {});
            });
            world.progress();
        }
    }
}

mod observer_in_system {
    use super::*;

    mod read_write {
        use super::*;

        #[test]
        #[should_panic]
        fn run_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            system!(world, &Foo).run(move |iter| {
                query.run(|iter| {
                    iter.fini();
                });
                iter.fini();
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            system!(world, &Foo).each(move |_| {
                query.each(|_| {});
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_entity_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            system!(world, &Foo).each_entity(move |_, _| {
                query.each_entity(|_, _| {});
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_iter_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            system!(world, &Foo).each_iter(move |_, _, _| {
                query.each_iter(|_, _, _| {});
            });
            world.progress();
        }
    }

    mod write_read {
        use super::*;

        #[test]
        #[should_panic]
        fn run_violation() {
            let world = World::new();
            let query = query!(world, &Foo).build();
            system!(world, &mut Foo).run(move |iter| {
                query.run(|iter| {
                    iter.fini();
                });
                iter.fini();
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_violation() {
            let world = World::new();
            let query = query!(world, &Foo).build();
            system!(world, &mut Foo).each(move |_| {
                query.each(|_| {});
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_entity_violation() {
            let world = World::new();
            let query = query!(world, &Foo).build();
            system!(world, &mut Foo).each_entity(move |_, _| {
                query.each_entity(|_, _| {});
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_iter_violation() {
            let world = World::new();
            let query = query!(world, &Foo).build();
            system!(world, &mut Foo).each_iter(move |_, _, _| {
                query.each_iter(|_, _, _| {});
            });
            world.progress();
        }
    }

    mod write_write {
        use super::*;

        #[test]
        #[should_panic]
        fn run_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            system!(world, &mut Foo).run(move |iter| {
                query.run(|iter| {
                    iter.fini();
                });
                iter.fini();
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            system!(world, &mut Foo).each(move |_| {
                query.each(|_| {});
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_entity_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            system!(world, &mut Foo).each_entity(move |_, _| {
                query.each_entity(|_, _| {});
            });
            world.progress();
        }

        #[test]
        #[should_panic]
        fn each_iter_violation() {
            let world = World::new();
            let query = query!(world, &mut Foo).build();
            system!(world, &mut Foo).each_iter(move |_, _, _| {
                query.each_iter(|_, _, _| {});
            });
            world.progress();
        }
    }
}
