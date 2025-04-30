#![allow(dead_code)]

use flecs_ecs::core::*;
use flecs_ecs::macros::*;

use crate::common_test::*;

#[derive(Debug, Component)]
struct LastEntity(Entity);

#[test]
fn observer_2_terms_on_add() {
    let world = World::new();

    world.set(Count(0));

    world
        .observer::<flecs::OnAdd, ()>()
        .with::<Position>()
        .with::<Velocity>()
        .each_entity(|e, _| {
            let world = e.world();
            world.get::<&mut Count>(|count| {
                count.0 += 1;
            });
        });

    let e = world.entity();
    world.get::<&mut Count>(|count| {
        assert_eq!(count.0, 0);
    });
    e.set(Position { x: 10, y: 20 });
    world.get::<&mut Count>(|count| {
        assert_eq!(count.0, 0);
    });
    e.set(Velocity { x: 1, y: 2 });
    world.get::<&mut Count>(|count| {
        assert_eq!(count.0, 1);
    });
}

#[test]
fn observer_2_terms_on_remove() {
    let world = World::new();

    world.set(Count(0));

    world
        .observer::<flecs::OnRemove, (&Position, &Velocity)>()
        .each_entity(|e, (pos, vel)| {
            let world = e.world();
            world.get::<&mut Count>(|count| {
                count.0 += 1;
            });
            assert_eq!(pos.x, 10);
            assert_eq!(pos.y, 20);
            assert_eq!(vel.x, 1);
            assert_eq!(vel.y, 2);
        });

    let e = world.entity();
    world.get::<&mut Count>(|count| {
        assert_eq!(count, 0);
    });
    e.set(Position { x: 10, y: 20 });
    world.get::<&mut Count>(|count| {
        assert_eq!(count, 0);
    });
    e.set(Velocity { x: 1, y: 2 });
    world.get::<&mut Count>(|count| {
        assert_eq!(count, 0);
    });
    e.remove::<Velocity>();
    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
    e.remove::<Position>();
    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_2_terms_on_set() {
    let world = World::new();

    world.set(Count(0));

    world
        .observer::<flecs::OnSet, (&Position, &Velocity)>()
        .each_entity(|e, (pos, vel)| {
            let world = e.world();
            world.get::<&mut Count>(|count| {
                count.0 += 1;
            });
            assert_eq!(pos.x, 10);
            assert_eq!(pos.y, 20);
            assert_eq!(vel.x, 1);
            assert_eq!(vel.y, 2);
        });

    let e = world.entity();
    world.get::<&mut Count>(|count| {
        assert_eq!(count, 0);
    });
    e.set(Position { x: 10, y: 20 });
    world.get::<&mut Count>(|count| {
        assert_eq!(count, 0);
    });
    e.set(Velocity { x: 1, y: 2 });
    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_10_terms() {
    let world = World::new();

    world.set(Count(0));

    let e = world.entity();
    let e_id = e.id();

    world
        .observer::<flecs::OnAdd, ()>()
        .with::<TagA>()
        .with::<TagB>()
        .with::<TagC>()
        .with::<TagD>()
        .with::<TagE>()
        .with::<TagF>()
        .with::<TagG>()
        .with::<TagH>()
        .with::<TagI>()
        .with::<TagJ>()
        .each_iter(move |it, _i, _| {
            let world = it.world();
            assert_eq!(it.count(), 1);
            assert!(it.entity(0) == e_id);
            assert_eq!(it.field_count(), 10);
            world.get::<&mut Count>(|count| {
                count.0 += 1;
            });
        });

    e.add::<TagA>()
        .add::<TagB>()
        .add::<TagC>()
        .add::<TagD>()
        .add::<TagE>()
        .add::<TagF>()
        .add::<TagG>()
        .add::<TagH>()
        .add::<TagI>()
        .add::<TagJ>();

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_16_terms() {
    let world = World::new();

    world.set(Count(0));

    let e = world.entity();
    let e_id = e.id();

    world
        .observer::<flecs::OnAdd, ()>()
        .with::<TagA>()
        .with::<TagB>()
        .with::<TagC>()
        .with::<TagD>()
        .with::<TagE>()
        .with::<TagF>()
        .with::<TagG>()
        .with::<TagH>()
        .with::<TagI>()
        .with::<TagJ>()
        .with::<TagK>()
        .with::<TagL>()
        .with::<TagM>()
        .with::<TagN>()
        .with::<TagO>()
        .with::<TagP>()
        .each_iter(move |it, _i, _| {
            let world = it.world();
            assert_eq!(it.count(), 1);
            assert!(it.entity(0) == e_id);
            assert_eq!(it.field_count(), 16);
            world.get::<&mut Count>(|count| {
                count.0 += 1;
            });
        });

    e.add::<TagA>()
        .add::<TagB>()
        .add::<TagC>()
        .add::<TagD>()
        .add::<TagE>()
        .add::<TagF>()
        .add::<TagG>()
        .add::<TagH>()
        .add::<TagI>()
        .add::<TagJ>()
        .add::<TagK>()
        .add::<TagL>()
        .add::<TagM>()
        .add::<TagN>()
        .add::<TagO>()
        .add::<TagP>()
        .add::<TagQ>()
        .add::<TagR>()
        .add::<TagS>()
        .add::<TagT>();

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_2_entities_iter() {
    let world = World::new();

    let e1 = world.entity();
    let e2 = world.entity();

    let e1_id = e1.id();
    let e2_id = e2.id();

    world.set(Count(0));
    world.set(LastEntity(Entity::null()));

    world
        .observer::<flecs::OnSet, &Position>()
        .run(move |mut it| {
            let world = it.world();
            while it.next() {
                let p = it.field::<&Position>(0).unwrap();

                for i in it.iter() {
                    world.get::<&mut Count>(|count| {
                        count.0 += 1;
                    });
                    if it.entity(i) == e1_id {
                        assert_eq!(p[i].x, 10);
                        assert_eq!(p[i].y, 20);
                    } else if it.entity(i) == e2_id {
                        assert_eq!(p[i].x, 30);
                        assert_eq!(p[i].y, 40);
                    } else {
                        unreachable!();
                    }

                    world.get::<&mut LastEntity>(|last| {
                        last.0 = it.entity(i).id();
                    });
                }
            }
        });

    e1.set(Position { x: 10, y: 20 });
    world.get::<&mut Count>(|count| {
        world.get::<&mut LastEntity>(|last| {
            assert_eq!(count, 1);
            assert!(last.0 == e1.id());
        });
    });

    e2.set(Position { x: 30, y: 40 });
    world.get::<&mut Count>(|count| {
        world.get::<&mut LastEntity>(|last| {
            assert_eq!(count, 2);
            assert!(last.0 == e2.id());
        });
    });
}

#[test]
fn observer_2_entities_table_column() {
    let world = World::new();

    let e1 = world.entity();
    let e2 = world.entity();

    let e1_id = e1.id();
    let e2_id = e2.id();

    world.set(Count(0));
    world.set(LastEntity(Entity::null()));

    world
        .observer::<flecs::OnSet, &Position>()
        .run(move |mut it| {
            let world = it.world();
            while it.next() {
                let table_range = it.range().unwrap();
                let p = table_range.get_mut::<Position>().unwrap();

                for i in it.iter() {
                    world.get::<&mut Count>(|count| {
                        count.0 += 1;
                    });
                    if it.entity(i) == e1_id {
                        assert_eq!(p[i].x, 10);
                        assert_eq!(p[i].y, 20);
                    } else if it.entity(i) == e2_id {
                        assert_eq!(p[i].x, 30);
                        assert_eq!(p[i].y, 40);
                    } else {
                        unreachable!();
                    }

                    world.get::<&mut LastEntity>(|last| {
                        last.0 = it.entity(i).id();
                    });
                }
            }
        });

    e1.set(Position { x: 10, y: 20 });
    world.get::<&mut Count>(|count| {
        world.get::<&mut LastEntity>(|last| {
            assert_eq!(count, 1);
            assert!(last.0 == e1.id());
        });
    });

    e2.set(Position { x: 30, y: 40 });
    world.get::<&mut Count>(|count| {
        world.get::<&mut LastEntity>(|last| {
            assert_eq!(count, 2);
            assert!(last.0 == e2.id());
        });
    });
}

#[test]
fn observer_2_entities_each() {
    let world = World::new();

    let e1 = world.entity();
    let e2 = world.entity();

    let e1_id = e1.id();
    let e2_id = e2.id();

    world.set(Count(0));
    world.set(LastEntity(Entity::null()));

    world
        .observer::<flecs::OnSet, &Position>()
        .each_entity(move |e, pos| {
            let world = e.world();
            world.get::<&mut Count>(|count| {
                count.0 += 1;
            });
            if e == e1_id {
                assert_eq!(pos.x, 10);
                assert_eq!(pos.y, 20);
            } else if e == e2_id {
                assert_eq!(pos.x, 30);
                assert_eq!(pos.y, 40);
            } else {
                unreachable!();
            }

            world.get::<&mut LastEntity>(|last| {
                last.0 = e.id();
            });
        });

    e1.set(Position { x: 10, y: 20 });
    world.get::<&mut Count>(|count| {
        world.get::<&mut LastEntity>(|last| {
            assert_eq!(count, 1);
            assert!(last.0 == e1);
        });
    });
    e2.set(Position { x: 30, y: 40 });
    world.get::<&mut Count>(|count| {
        world.get::<&mut LastEntity>(|last| {
            assert_eq!(count, 2);
            assert!(last.0 == e2);
        });
    });
}

#[test]
fn observer_create_w_no_template_args() {
    let world = World::new();

    let e1 = world.entity();
    let e1_id = e1.id();

    world.set(Count(0));

    world
        .observer::<flecs::OnAdd, ()>()
        .with::<Position>()
        .each_entity(move |e, _| {
            let world = e.world();
            assert!(e == e1_id);
            world.get::<&mut Count>(|count| {
                count.0 += 1;
            });
        });

    e1.set(Position { x: 10, y: 20 });

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_yield_existing() {
    let world = World::new();

    let e1 = world.entity().add::<TagA>();
    let e2 = world.entity().add::<TagA>();
    let e3 = world.entity().add::<TagA>().add::<TagB>();

    let e1_id = e1.id();
    let e2_id = e2.id();
    let e3_id = e3.id();

    world.set(Count(0));
    world
        .observer::<flecs::OnAdd, ()>()
        .with::<TagA>()
        .yield_existing()
        .run(move |mut it| {
            while it.next() {
                for i in it.iter() {
                    let e = it.entity(i);
                    let world = e.world();
                    world.get::<&mut Count>(|count| {
                        if e == e1_id {
                            count.0 += 1;
                        } else if e == e2_id {
                            count.0 += 2;
                        } else if e == e3_id {
                            count.0 += 3;
                        }
                    });
                }
            }
        });

    world.get::<&mut Count>(|count| {
        assert_eq!(count.0, 6);
    });
}

#[test]
fn observer_yield_existing_2_terms() {
    let world = World::new();

    let e1 = world.entity().add::<TagA>().add::<TagB>();
    let e2 = world.entity().add::<TagA>().add::<TagB>();
    let e3 = world.entity().add::<TagA>().add::<TagB>().add::<TagC>();

    let e1_id = e1.id();
    let e2_id = e2.id();
    let e3_id = e3.id();

    world.entity().add::<TagA>();
    world.entity().add::<TagB>();

    world.set(Count(0));
    world
        .observer::<flecs::OnAdd, ()>()
        .with::<TagA>()
        .with::<TagB>()
        .yield_existing()
        .each_entity(move |e, _| {
            let world = e.world();
            world.get::<&mut Count>(|count| {
                if e == e1_id {
                    count.0 += 1;
                } else if e == e2_id {
                    count.0 += 2;
                } else if e == e3_id {
                    count.0 += 3;
                }
            });
        });

    world.get::<&mut Count>(|count| {
        assert_eq!(count.0, 6);
    });
}

#[test]
fn observer_on_add() {
    let world = World::new();
    world.set(Count(0));
    world
        .observer::<flecs::OnAdd, ()>()
        .with::<Position>()
        .each_entity(|e, _| {
            let world = e.world();
            world.get::<&mut Count>(|count| {
                count.0 += 1;
            });
        });

    world.entity().add::<Position>();

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_on_remove() {
    let world = World::new();
    world.set(Count(0));
    world
        .observer::<flecs::OnRemove, &Position>()
        .each_entity(|e, _p| {
            e.world().get::<&mut Count>(|count| {
                count.0 += 1;
            });
        });
    let e = world.entity().add::<Position>();

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 0);
    });

    e.remove::<Position>();

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_on_add_tag_action() {
    let world = World::new();
    world.set(Count(0));
    world
        .observer::<flecs::OnAdd, ()>()
        .with::<TagA>()
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                world.get::<&mut Count>(|count| {
                    count.0 += 1;
                });
            }
        });
    world.entity().add::<TagA>();

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_on_add_tag_iter() {
    let world = World::new();
    world.set(Count(0));
    world
        .observer::<flecs::OnAdd, ()>()
        .with::<TagA>()
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                world.get::<&mut Count>(|count| {
                    count.0 += 1;
                });
            }
        });
    world.entity().add::<TagA>();
    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_on_add_tag_each() {
    let world = World::new();
    world.set(Count(0));
    world
        .observer::<flecs::OnAdd, ()>()
        .with::<TagA>()
        .run(|mut it| {
            while it.next() {
                for _ in it.iter() {
                    it.world().get::<&mut Count>(|count| {
                        count.0 += 1;
                    });
                }
            }
        });
    world.entity().add::<TagA>();
    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_on_add_expr() {
    let world = World::new();
    world.set(Count(0));
    world.component::<Tag>();
    world
        .observer::<flecs::OnAdd, ()>()
        .expr("flecs.common_test.Tag")
        .each_entity(|e, _| {
            e.world().get::<&mut Count>(|count| {
                count.0 += 1;
            });
        });
    let e = world.entity().add::<Tag>();
    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
    e.remove::<Tag>();
    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_observer_w_filter_term() {
    let world = World::new();
    let tag_a = world.entity();
    let tag_b = world.entity();
    world.set(Count(0));
    world
        .observer::<flecs::OnAdd, ()>()
        .with_id(tag_a)
        .with_id(tag_b)
        .filter()
        .each_entity(|e, _| {
            e.world().get::<&mut Count>(|count| {
                count.0 += 1;
            });
        });
    let e = world.entity();

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 0);
    });

    e.add_id(tag_b);

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 0);
    });

    e.add_id(tag_a);

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });

    e.remove_id(tag_b);

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });

    e.add_id(tag_b);

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });

    e.clear();

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });

    e.add_id(tag_a);

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_run_callback() {
    let world = World::new();

    world.set(Count(0));
    world
        .observer::<flecs::OnAdd, ()>()
        .with::<Position>()
        .run_each_entity(
            |mut it| {
                while it.next() {
                    it.each();
                }
            },
            |e, _p| {
                e.world().get::<&mut Count>(|count| {
                    count.0 += 1;
                });
            },
        );
    let e = world.entity();
    world.get::<&mut Count>(|count| {
        assert_eq!(count, 0);
    });

    e.add::<Position>();

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_get_query() {
    let world = World::new();
    world.entity().set(Position { x: 0, y: 0 });
    world.entity().set(Position { x: 1, y: 0 });
    world.entity().set(Position { x: 2, y: 0 });

    world.set(Count(0));
    let mut o = world
        .observer::<flecs::OnSet, &Position>()
        .each_entity(|_e, _p| {});
    let q = o.query();
    q.run(|mut it| {
        while it.next() {
            let pos = it.field::<&Position>(0).unwrap();
            for i in it.iter() {
                assert_eq!(i as i32, pos[i].x);
                world.get::<&mut Count>(|count| {
                    count.0 += 1;
                });
            }
        }
    });

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 3);
    });
}

#[test]
fn observer_on_set_w_set() {
    let world = World::new();

    world.set(Count(0));
    world
        .observer::<flecs::OnSet, &Position>()
        .each_entity(|e, _p| {
            e.world().get::<&mut Count>(|count| {
                count.0 += 1;
            });
        });
    let e = world.entity();

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 0);
    });

    e.set(Position { x: 10, y: 20 });

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_on_set_w_defer_set() {
    let world = World::new();

    world.set(Count(0));
    world
        .observer::<flecs::OnSet, &Position>()
        .each_entity(|e, _p| {
            e.world().get::<&mut Count>(|count| {
                count.0 += 1;
            });
        });
    let e = world.entity();
    world.get::<&mut Count>(|count| {
        assert_eq!(count, 0);
    });

    world.defer_begin();

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 0);
    });

    e.set(Position { x: 10, y: 20 });

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 0);
    });

    world.defer_end();

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_on_set_w_set_sparse() {
    let world = World::new();

    world.set(Count(0));
    world.component::<Position>().add_trait::<flecs::Sparse>();

    world
        .observer::<flecs::OnSet, &Position>()
        .each_entity(|e, _p| {
            e.world().get::<&mut Count>(|count| {
                count.0 += 1;
            });
        });

    let e = world.entity();
    world.get::<&mut Count>(|count| {
        assert_eq!(count, 0);
    });

    e.set(Position { x: 10, y: 20 });

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_on_add_singleton() {
    let world = World::new();

    world.set(Count(0));
    world
        .observer::<flecs::OnSet, &Position>()
        .term_at(0)
        .singleton()
        .each_iter(|it, _i, pos| {
            assert_eq!(pos.x, 10);
            assert_eq!(pos.y, 20);
            it.world().get::<&mut Count>(|count| {
                count.0 += 1;
            });
        });
    world.set(Position { x: 10, y: 20 });

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_on_add_pair_singleton() {
    let world = World::new();

    world.set(Count(0));
    let tgt = world.entity();
    world
        .observer::<flecs::OnSet, ()>()
        .with_first::<Position>(tgt)
        .singleton()
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                let pos = it.field::<&Position>(0).unwrap();
                assert_eq!(pos[0].x, 10);
                assert_eq!(pos[0].y, 20);
                world.get::<&mut Count>(|count| {
                    count.0 += 1;
                });
            }
        });
    world.set_first(tgt, Position { x: 10, y: 20 });
    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_on_add_pair_wildcard_singleton() {
    let world = World::new();

    world.set(Count(0));
    let tgt_1 = world.entity();
    let tgt_2 = world.entity();
    world
        .observer::<flecs::OnSet, &(Position, flecs::Wildcard)>()
        .term_at(0)
        .singleton()
        .each_iter(|it, _i, pos| {
            assert_eq!(pos.x, 10);
            assert_eq!(pos.y, 20);
            it.world().get::<&mut Count>(|count| {
                count.0 += 1;
            });
        });

    world.set_first::<Position>(tgt_1, Position { x: 10, y: 20 });

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });

    world.set_first::<Position>(tgt_2, Position { x: 10, y: 20 });

    world.get::<&mut Count>(|count| {
        assert_eq!(count, 2);
    });
}

#[test]
fn observer_on_add_with_pair_singleton() {
    let world = World::new();

    world.set(Count(0));
    let tgt = world.entity();
    world
        .observer::<flecs::OnSet, ()>()
        .with_first::<Position>(tgt)
        .singleton()
        .each_iter(|it, _, _| {
            it.world().get::<&mut Count>(|count| {
                count.0 += 1;
            });
        });
    world.set_first::<Position>(tgt, Position { x: 10, y: 20 });
    world.get::<&mut Count>(|count| {
        assert_eq!(count, 1);
    });
}

#[test]
fn observer_add_in_yield_existing() {
    let world = World::new();
    let e1 = world.entity().set(Position::default());
    let e2 = world.entity().set(Position::default());
    let e3 = world.entity().set(Position::default());
    world
        .observer::<flecs::OnAdd, ()>()
        .with::<Position>()
        .yield_existing()
        .each_entity(|e, _| {
            e.add::<Velocity>();
        });

    assert!(e1.has::<Position>());
    assert!(e1.has::<Velocity>());
    assert!(e2.has::<Position>());
    assert!(e2.has::<Velocity>());
    assert!(e3.has::<Position>());
    assert!(e3.has::<Velocity>());
}

#[test]
fn observer_add_in_yield_existing_multi() {
    let world = World::new();
    let e1 = world.entity().set(Position::default()).set(Mass::default());
    let e2 = world.entity().set(Position::default()).set(Mass::default());
    let e3 = world.entity().set(Position::default()).set(Mass::default());
    world
        .observer::<flecs::OnAdd, ()>()
        .with::<Position>()
        .with::<Mass>()
        .yield_existing()
        .each_entity(|e, _| {
            e.add::<Velocity>();
        });
    assert!(e1.has::<Position>());
    assert!(e1.has::<Mass>());
    assert!(e1.has::<Velocity>());
    assert!(e2.has::<Position>());
    assert!(e2.has::<Mass>());
    assert!(e2.has::<Velocity>());
    assert!(e3.has::<Position>());
    assert!(e3.has::<Mass>());
    assert!(e3.has::<Velocity>());
}

#[test]
fn observer_name_from_root() {
    let world = World::new();
    let o = world
        .observer_named::<flecs::OnSet, &Position>("::ns::MyObserver")
        .each(|_| {});

    assert_eq!(o.name(), "MyObserver");
    let ns = world.entity_named("::ns");
    assert!(ns == o.parent().unwrap());
}

// #[test]
// #[should_panic]
// fn observer_panic_inside() {
//     #[derive(Component)]
//     struct Tag;

//     let world = World::new();
//     world.observer::<flecs::OnAdd, ()>().with::<Tag>().run(|_| panic!());
//     world.add::<Tag>();
// }

#[test]
fn observer_register_twice_w_each() {
    let world = World::new();

    world.set(Count2 { a: 0, b: 0 });

    world
        .observer_named::<flecs::OnSet, &Position>("Test")
        .each_entity(|e, _| {
            e.world().get::<&mut Count2>(|count| {
                count.a += 1;
            });
        });

    world.entity().set(Position { x: 10, y: 20 });

    world.get::<&mut Count2>(|count| {
        assert_eq!(count.a, 1);
    });

    world
        .observer_named::<flecs::OnSet, &Position>("Test")
        .each_entity(|e, _| {
            e.world().get::<&mut Count2>(|count| {
                count.b += 1;
            });
        });

    world.entity().set(Position { x: 10, y: 20 });

    world.get::<&mut Count2>(|count| {
        assert_eq!(count.b, 1);
    });
}

#[test]
fn observer_register_twice_w_run() {
    let world = World::new();

    world.set(Count2 { a: 0, b: 0 });

    world
        .observer_named::<flecs::OnSet, &Position>("Test")
        .run(|mut it| {
            while it.next() {
                it.world().get::<&mut Count2>(|count| {
                    count.a += 1;
                });
            }
        });

    world.entity().set(Position { x: 10, y: 20 });

    world.get::<&mut Count2>(|count| {
        assert_eq!(count.a, 1);
    });

    world
        .observer_named::<flecs::OnSet, &Position>("Test")
        .run(|mut it| {
            while it.next() {
                it.world().get::<&mut Count2>(|count| {
                    count.b += 1;
                });
            }
        });

    world.entity().set(Position { x: 10, y: 20 });

    world.get::<&mut Count2>(|count| {
        assert_eq!(count.b, 1);
    });
}

#[test]
fn observer_register_twice_w_run_each() {
    let world = World::new();

    world.set(Count2 { a: 0, b: 0 });

    world
        .observer_named::<flecs::OnSet, &Position>("Test")
        .run(|mut it| {
            while it.next() {
                it.world().get::<&mut Count2>(|count| {
                    count.a += 1;
                });
            }
        });

    world.entity().set(Position { x: 10, y: 20 });

    world.get::<&mut Count2>(|count| {
        assert_eq!(count.a, 1);
    });

    world
        .observer_named::<flecs::OnSet, &Position>("Test")
        .each_entity(|e, _| {
            e.world().get::<&mut Count2>(|count| {
                count.b += 1;
            });
        });

    world.entity().set(Position { x: 10, y: 20 });

    world.get::<&mut Count2>(|count| {
        assert_eq!(count.b, 1);
    });
}

#[test]
fn observer_register_twice_w_each_run() {
    let world = World::new();

    world.set(Count2 { a: 0, b: 0 });

    world
        .observer_named::<flecs::OnSet, &Position>("Test")
        .each_entity(|e, _| {
            e.world().get::<&mut Count2>(|count| {
                count.a += 1;
            });
        });

    world.entity().set(Position { x: 10, y: 20 });

    world.get::<&mut Count2>(|count| {
        assert_eq!(count.a, 1);
    });

    world
        .observer_named::<flecs::OnSet, &Position>("Test")
        .run(|mut it| {
            while it.next() {
                it.world().get::<&mut Count2>(|count| {
                    count.b += 1;
                });
            }
        });

    world.entity().set(Position { x: 10, y: 20 });

    world.get::<&mut Count2>(|count| {
        assert_eq!(count.b, 1);
    });
}

//TODO other observer tests
