#![allow(dead_code)]

use std::ffi::c_void;

use flecs_ecs::core::*;
use flecs_ecs::macros::*;

use crate::common_test::*;

#[derive(Component)]
struct LastVal(i32);

#[test]
fn system_iter() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    world
        .system::<(&mut Position, &mut Velocity)>()
        .run(|mut it| {
            while it.next() {
                let mut p = it.field::<Position>(0).unwrap();
                let v = it.field::<Velocity>(1).unwrap();
                for i in it.iter() {
                    p[i].x += v[i].x;
                    p[i].y += v[i].y;
                }
            }
        });

    world.progress();

    entity.get::<(&Position, &Velocity)>(|(p, v)| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });
}

#[test]
fn system_iter_macro() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    system!(world, &mut Position, &Velocity).run(|mut it| {
        while it.next() {
            let mut p = it.field::<Position>(0).unwrap();
            let v = it.field::<Velocity>(1).unwrap();
            for i in it.iter() {
                p[i].x += v[i].x;
                p[i].y += v[i].y;
            }
        }
    });

    world.progress();

    entity.get::<(&Position, &Velocity)>(|(p, v)| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });
}

#[test]
fn system_iter_const() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    world.system::<(&mut Position, &Velocity)>().run(|mut it| {
        while it.next() {
            let mut p = it.field::<Position>(0).unwrap();
            let v = it.field::<&Velocity>(1).unwrap();
            for i in it.iter() {
                p[i].x += v[i].x;
                p[i].y += v[i].y;
            }
        }
    });

    world.progress();

    entity.get::<(&Position, &Velocity)>(|(p, v)| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });
}

#[test]
fn system_iter_shared() {
    let world = World::new();

    world
        .component::<Position>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();
    world
        .component::<Velocity>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

    let base = world.entity().set(Velocity { x: 1, y: 2 });

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .add_id((flecs::IsA::ID, base));

    let e2 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 3, y: 4 });

    world
        .system::<&mut Position>()
        .expr("flecs.common_test.Velocity(self|up IsA)")
        .run(|mut it| {
            while it.next() {
                let mut p = it.field::<Position>(0).unwrap();
                let v = it.field::<&Velocity>(1).unwrap();

                if !it.is_self(1) {
                    for i in it.iter() {
                        p[i].x += v[0].x;
                        p[i].y += v[0].y;
                    }
                } else {
                    for i in it.iter() {
                        p[i].x += v[i].x;
                        p[i].y += v[i].y;
                    }
                }
            }
        });

    world.progress();

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 13);
        assert_eq!(p.y, 24);
    });
}

#[test]
fn system_iter_optional() {
    let world = World::new();
    world.component_named::<Mass>("Mass");

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 })
        .set(Mass { value: 1 });

    let e2 = world
        .entity()
        .set(Position { x: 30, y: 40 })
        .set(Velocity { x: 3, y: 4 })
        .set(Mass { value: 1 });

    let e3 = world.entity().set(Position { x: 50, y: 60 });

    let e4 = world.entity().set(Position { x: 70, y: 80 });

    world
        .system::<(&mut Position, Option<&mut Velocity>, Option<&mut Mass>)>()
        .run(|mut it| {
            while it.next() {
                let mut p = it.field::<Position>(0).unwrap();

                if it.is_set(1) && it.is_set(2) {
                    let v = it.field::<Velocity>(1).unwrap();
                    let m = it.field::<Mass>(2).unwrap();
                    for i in it.iter() {
                        p[i].x += v[i].x * m[i].value;
                        p[i].y += v[i].y * m[i].value;
                    }
                } else {
                    for i in it.iter() {
                        p[i].x += 1;
                        p[i].y += 1;
                    }
                }
            }
        });

    world.progress();

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 33);
        assert_eq!(p.y, 44);
    });

    e3.get::<&Position>(|p| {
        assert_eq!(p.x, 51);
        assert_eq!(p.y, 61);
    });

    e4.get::<&Position>(|p| {
        assert_eq!(p.x, 71);
        assert_eq!(p.y, 81);
    });
}

#[test]
fn system_each() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    world
        .system::<(&mut Position, &mut Velocity)>()
        .each_entity(|_e, (p, v)| {
            p.x += v.x;
            p.y += v.y;
        });

    world.progress();

    entity.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}

#[test]
fn system_each_const() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    world
        .system::<(&mut Position, &Velocity)>()
        .each_entity(|_e, (p, v)| {
            p.x += v.x;
            p.y += v.y;
        });

    world.progress();

    entity.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}

#[test]
fn system_each_shared() {
    let world = World::new();

    let base = world.entity().set(Velocity { x: 1, y: 2 });

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .add_id((flecs::IsA::ID, base));

    let e2 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 3, y: 4 });

    world
        .system::<(&mut Position, &Velocity)>()
        .each_entity(|_e, (p, v)| {
            p.x += v.x;
            p.y += v.y;
        });

    world.progress();

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 13);
        assert_eq!(p.y, 24);
    });
}

#[test]
fn system_each_optional() {
    let world = World::new();
    world.component_named::<Mass>("Mass");

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 })
        .set(Mass { value: 1 });

    let e2 = world
        .entity()
        .set(Position { x: 30, y: 40 })
        .set(Velocity { x: 3, y: 4 })
        .set(Mass { value: 1 });

    let e3 = world.entity().set(Position { x: 50, y: 60 });

    let e4 = world.entity().set(Position { x: 70, y: 80 });

    world
        .system::<(&mut Position, Option<&mut Velocity>, Option<&mut Mass>)>()
        .each_entity(|_e, (p, v, m)| {
            if v.is_some() && m.is_some() {
                let v = v.unwrap();
                let m = m.unwrap();
                p.x += v.x * m.value;
                p.y += v.y * m.value;
            } else {
                p.x += 1;
                p.y += 1;
            }
        });

    world.progress();

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 33);
        assert_eq!(p.y, 44);
    });

    e3.get::<&Position>(|p| {
        assert_eq!(p.x, 51);
        assert_eq!(p.y, 61);
    });

    e4.get::<&Position>(|p| {
        assert_eq!(p.x, 71);
        assert_eq!(p.y, 81);
    });
}

#[test]
fn system_signature() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    world
        .system::<()>()
        .expr("flecs.common_test.Position, flecs.common_test.Velocity")
        .run(|mut it| {
            while it.next() {
                let mut p = it.field::<Position>(0).unwrap();
                let v = it.field::<Velocity>(1).unwrap();

                for i in it.iter() {
                    p[i].x += v[i].x;
                    p[i].y += v[i].y;
                }
            }
        });

    world.progress();

    entity.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    entity.get::<&Velocity>(|v| {
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });
}

#[test]
fn system_signature_const() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    world
        .system::<()>()
        .expr("flecs.common_test.Position, [in] flecs.common_test.Velocity")
        .run(|mut it| {
            while it.next() {
                let mut p = it.field::<Position>(0).unwrap();
                let v = it.field::<Velocity>(1).unwrap();

                for i in it.iter() {
                    p[i].x += v[i].x;
                    p[i].y += v[i].y;
                }
            }
        });

    world.progress();

    entity.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    entity.get::<&Velocity>(|v| {
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });
}

#[test]
fn system_signature_shared() {
    let world = World::new();

    world
        .component::<Position>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();
    world
        .component::<Velocity>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

    let base = world.entity().set(Velocity { x: 1, y: 2 });

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .add_id((flecs::IsA::ID, base));

    let e2 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 3, y: 4 });

    world
        .system::<()>()
        .expr("flecs.common_test.Position, [in] flecs.common_test.Velocity(self|up IsA)")
        .run(|mut it| {
            while it.next() {
                let mut p = it.field::<Position>(0).unwrap();
                let v = it.field::<Velocity>(1).unwrap();

                if !it.is_self(1) {
                    for i in it.iter() {
                        p[i].x += v[0].x;
                        p[i].y += v[0].y;
                    }
                } else {
                    for i in it.iter() {
                        p[i].x += v[i].x;
                        p[i].y += v[i].y;
                    }
                }
            }
        });

    world.progress();

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 13);
        assert_eq!(p.y, 24);
    });
}

#[test]
fn system_signature_optional() {
    let world = World::new();
    world.component_named::<Mass>("Mass");

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 })
        .set(Mass { value: 1 });

    let e2 = world
        .entity()
        .set(Position { x: 30, y: 40 })
        .set(Velocity { x: 3, y: 4 })
        .set(Mass { value: 1 });

    let e3 = world.entity().set(Position { x: 50, y: 60 });

    let e4 = world.entity().set(Position { x: 70, y: 80 });

    world
        .system::<()>()
        .expr("flecs.common_test.Position, ?flecs.common_test.Velocity, ?Mass")
        .run(|mut it| {
            while it.next() {
                let mut p = it.field::<Position>(0).unwrap();

                if it.is_set(1) && it.is_set(2) {
                    let v = it.field::<Velocity>(1).unwrap();
                    let m = it.field::<Mass>(2).unwrap();
                    for i in it.iter() {
                        p[i].x += v[i].x * m[i].value;
                        p[i].y += v[i].y * m[i].value;
                    }
                } else {
                    for i in it.iter() {
                        p[i].x += 1;
                        p[i].y += 1;
                    }
                }
            }
        });

    world.progress();

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 33);
        assert_eq!(p.y, 44);
    });

    e3.get::<&Position>(|p| {
        assert_eq!(p.x, 51);
        assert_eq!(p.y, 61);
    });

    e4.get::<&Position>(|p| {
        assert_eq!(p.x, 71);
        assert_eq!(p.y, 81);
    });
}

#[test]
fn system_copy_name_on_create() {
    let world = World::new();

    let mut name = "hello";

    let system_1 = world
        .system_named::<&mut Position>(name)
        .run(|mut it| while it.next() {});

    name = "world";
    let system_2 = world
        .system_named::<&mut Position>(name)
        .run(|mut it| while it.next() {});

    assert!(system_1.id() != system_2.id());
}

#[test]
fn system_nested_system() {
    let world = World::new();

    let system_1 = world
        .system_named::<&mut Position>("foo::bar")
        .run(|mut it| while it.next() {});

    assert_eq!(system_1.name(), "bar");

    let e = world.lookup("foo");
    assert!(e.id() != 0);
    assert_eq!(e.name(), "foo");

    let se = e.lookup("bar");
    assert!(se.id() != 0);
    assert_eq!(se.name(), "bar");
}

#[test]
fn system_empty_signature() {
    let world = World::new();

    world.set(Count(0));

    world.system::<()>().run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        }
    });

    world.progress();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 1);
    });
}

#[test]
fn system_iter_tag() {
    let world = World::new();

    world.set(Count(0));

    world.system::<&TagA>().run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        }
    });

    world.entity().add::<TagA>();

    world.progress();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 1);
    });
}

#[test]
fn system_each_tag() {
    let world = World::new();

    world.set(Count(0));

    world.system::<&TagA>().each_entity(|e, _tag_a| {
        let world = e.world();
        world.get::<&mut Count>(|c| {
            c.0 += 1;
        });
    });

    world.entity().add::<TagA>();

    world.progress();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 1);
    });
}

#[ignore = "implement missing functions"]
#[test]
fn system_set_interval() {
    let world = World::new();

    let _sys = world.system::<()>().kind_id(0).interval(1.0).run(|_it| {});

    // float i = sys.interval();
    // assert_eq!(i, 1.0f);

    // sys.interval(2.0f);

    // i = sys.interval();
    // assert_eq!(i, 2.0f);
}

#[test]
fn system_order_by_type() {
    let world = World::new();

    world.entity().set(Position { x: 3, y: 0 });
    world.entity().set(Position { x: 1, y: 0 });
    world.entity().set(Position { x: 5, y: 0 });
    world.entity().set(Position { x: 2, y: 0 });
    world.entity().set(Position { x: 4, y: 0 });

    world.set(LastVal(0));
    world.set(Count(0));

    let sys = world
        .system::<&Position>()
        .order_by::<Position>(|_e1, p1: &Position, _e2, p2: &Position| -> i32 {
            (p1.x > p2.x) as i32 - (p1.x < p2.x) as i32
        })
        .each_entity(|e, p| {
            let world = e.world();
            world.get::<&mut LastVal>(|last_val| {
                assert!(p.x > last_val.0);
                last_val.0 = p.x;
            });
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        });

    sys.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 5);
    });
}

#[test]
fn system_order_by_id() {
    let world = World::new();

    let pos = world.component::<Position>();

    world.entity().set(Position { x: 3, y: 0 });
    world.entity().set(Position { x: 1, y: 0 });
    world.entity().set(Position { x: 5, y: 0 });
    world.entity().set(Position { x: 2, y: 0 });
    world.entity().set(Position { x: 4, y: 0 });

    world.set(LastVal(0));
    world.set(Count(0));

    let sys = world
        .system::<&Position>()
        .order_by_id(
            pos,
            |_e1, p1: *const c_void, _e2, p2: *const c_void| -> i32 {
                let p1 = unsafe { &*(p1 as *const Position) };
                let p2 = unsafe { &*(p2 as *const Position) };
                (p1.x > p2.x) as i32 - (p1.x < p2.x) as i32
            },
        )
        .each_entity(|e, p| {
            let world = e.world();
            world.get::<&mut LastVal>(|last_val| {
                assert!(p.x > last_val.0);
                last_val.0 = p.x;
            });
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        });

    sys.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 5);
    });
}

#[test]
fn system_order_by_type_after_create() {
    let world = World::new();

    world.entity().set(Position { x: 3, y: 0 });
    world.entity().set(Position { x: 1, y: 0 });
    world.entity().set(Position { x: 5, y: 0 });
    world.entity().set(Position { x: 2, y: 0 });
    world.entity().set(Position { x: 4, y: 0 });

    world.set(LastVal(0));
    world.set(Count(0));

    let sys = world
        .system::<&Position>()
        .order_by::<Position>(|_e1, p1: &Position, _e2, p2: &Position| -> i32 {
            (p1.x > p2.x) as i32 - (p1.x < p2.x) as i32
        })
        .each_entity(|e, p| {
            let world = e.world();
            world.get::<&mut LastVal>(|last_val| {
                assert!(p.x > last_val.0);
                last_val.0 = p.x;
            });
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        });

    sys.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 5);
    });
}

#[test]
fn system_order_by_id_after_create() {
    let world = World::new();

    let pos = world.component::<Position>();

    world.entity().set(Position { x: 3, y: 0 });
    world.entity().set(Position { x: 1, y: 0 });
    world.entity().set(Position { x: 5, y: 0 });
    world.entity().set(Position { x: 2, y: 0 });
    world.entity().set(Position { x: 4, y: 0 });

    world.set(LastVal(0));
    world.set(Count(0));

    let sys = world
        .system::<&Position>()
        .order_by_id(
            pos,
            |_e1, p1: *const c_void, _e2, p2: *const c_void| -> i32 {
                let p1 = unsafe { &*(p1 as *const Position) };
                let p2 = unsafe { &*(p2 as *const Position) };
                (p1.x > p2.x) as i32 - (p1.x < p2.x) as i32
            },
        )
        .each_entity(|e, p| {
            let world = e.world();
            world.get::<&mut LastVal>(|last_val| {
                assert!(p.x > last_val.0);
                last_val.0 = p.x;
            });
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        });

    sys.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 5);
    });
}

#[test]
fn system_get_query() {
    let world = World::new();

    world.entity().set(Position { x: 0, y: 0 });
    world.entity().set(Position { x: 1, y: 0 });
    world.entity().set(Position { x: 2, y: 0 });

    world.set(Count(0));

    let sys = world.system::<&Position>().each_entity(|_e, _p| {
        // Not used
    });

    let q = sys.query();

    q.run(|mut it| {
        let world = it.world();
        while it.next() {
            let pos = it.field::<&Position>(0).unwrap();
            for i in it.iter() {
                assert_eq!(i as i32, pos[i].x);
                world.get::<&mut Count>(|c| {
                    c.0 += 1;
                });
            }
        }
    });

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 3);
    });
}

#[test]
fn system_add_from_each() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 0, y: 0 });
    let e2 = world.entity().set(Position { x: 1, y: 0 });
    let e3 = world.entity().set(Position { x: 2, y: 0 });

    world.system::<&Position>().each_entity(|e, _p| {
        e.add::<Velocity>();
        // Add is deferred
        assert!(!e.has::<Velocity>());
    });

    world.progress();

    assert!(e1.has::<Velocity>());
    assert!(e2.has::<Velocity>());
    assert!(e3.has::<Velocity>());
}

#[test]
fn system_delete_from_each() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 0, y: 0 });
    let e2 = world.entity().set(Position { x: 1, y: 0 });
    let e3 = world.entity().set(Position { x: 2, y: 0 });

    world.system::<&Position>().each_entity(|e, _p| {
        e.destruct();
        // Delete is deferred
        assert!(e.is_alive());
    });

    world.progress();

    assert!(!e1.is_alive());
    assert!(!e2.is_alive());
    assert!(!e3.is_alive());
}

#[test]
fn system_add_from_each_world_handle() {
    let world = World::new();

    let e1 = world.entity().set(EntityRef {
        value: world.entity().id(),
    });
    let e2 = world.entity().set(EntityRef {
        value: world.entity().id(),
    });
    let e3 = world.entity().set(EntityRef {
        value: world.entity().id(),
    });

    world.system::<&EntityRef>().each_entity(|e, c| {
        let world = e.world();
        let e = world.entity_from_id(c.value);
        e.mut_stage_of(e).add::<Position>();
    });

    world.progress();

    e1.get::<&EntityRef>(|c| {
        let e = world.entity_from_id(c.value);
        assert!(e.has::<Position>());
    });

    e2.get::<&EntityRef>(|c| {
        let e = world.entity_from_id(c.value);
        assert!(e.has::<Position>());
    });

    e3.get::<&EntityRef>(|c| {
        let e = world.entity_from_id(c.value);
        assert!(e.has::<Position>());
    });
}

#[test]
fn system_new_from_each() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 0, y: 0 });
    let e2 = world.entity().set(Position { x: 0, y: 0 });
    let e3 = world.entity().set(Position { x: 0, y: 0 });

    world.system::<&Position>().each_entity(|e, _p| {
        e.set(EntityRef {
            value: e.world().entity().add::<Velocity>().id(),
        });
    });

    world.progress();

    assert!(e1.has::<EntityRef>());
    assert!(e2.has::<EntityRef>());
    assert!(e3.has::<EntityRef>());

    e1.get::<&EntityRef>(|c| {
        assert!(world.entity_from_id(c.value).has::<Velocity>());
    });

    e2.get::<&EntityRef>(|c| {
        assert!(world.entity_from_id(c.value).has::<Velocity>());
    });

    e3.get::<&EntityRef>(|c| {
        assert!(world.entity_from_id(c.value).has::<Velocity>());
    });
}

#[test]
fn system_add_from_iter() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 0, y: 0 });
    let e2 = world.entity().set(Position { x: 1, y: 0 });
    let e3 = world.entity().set(Position { x: 2, y: 0 });

    world.system::<&Position>().run(|mut it| {
        while it.next() {
            for i in it.iter() {
                it.entity(i).add::<Velocity>();
                assert!(!it.entity(i).has::<Velocity>());
            }
        }
    });

    world.progress();

    assert!(e1.has::<Velocity>());
    assert!(e2.has::<Velocity>());
    assert!(e3.has::<Velocity>());
}

#[test]
fn system_delete_from_iter() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 0, y: 0 });
    let e2 = world.entity().set(Position { x: 1, y: 0 });
    let e3 = world.entity().set(Position { x: 2, y: 0 });

    world.system::<&Position>().run(|mut it| {
        while it.next() {
            for i in it.iter() {
                it.entity(i).destruct();
                // Delete is deferred
                assert!(it.entity(i).is_alive());
            }
        }
    });

    world.progress();

    assert!(!e1.is_alive());
    assert!(!e2.is_alive());
    assert!(!e3.is_alive());
}

#[test]
fn system_add_from_iter_world_handle() {
    let world = World::new();

    let e1 = world.entity().set(EntityRef {
        value: world.entity().id(),
    });
    let e2 = world.entity().set(EntityRef {
        value: world.entity().id(),
    });
    let e3 = world.entity().set(EntityRef {
        value: world.entity().id(),
    });

    world.system::<&EntityRef>().run(|mut it| {
        let world = it.world();
        while it.next() {
            let c = it.field::<EntityRef>(0).unwrap();
            for i in it.iter() {
                world
                    .entity_from_id(c[i].value)
                    .mut_current_stage(it.world())
                    .add::<Position>();
            }
        }
    });

    world.progress();

    e1.get::<&EntityRef>(|c| {
        let e = world.entity_from_id(c.value);
        assert!(e.has::<Position>());
    });

    e2.get::<&EntityRef>(|c| {
        let e = world.entity_from_id(c.value);
        assert!(e.has::<Position>());
    });

    e3.get::<&EntityRef>(|c| {
        let e = world.entity_from_id(c.value);
        assert!(e.has::<Position>());
    });
}

#[test]
fn system_new_from_iter() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 0, y: 0 });
    let e2 = world.entity().set(Position { x: 0, y: 0 });
    let e3 = world.entity().set(Position { x: 0, y: 0 });

    world.system::<&Position>().run(|mut it| {
        while it.next() {
            for i in it.iter() {
                it.entity(i).set(EntityRef {
                    value: it.world().entity().add::<Velocity>().id(),
                });
            }
        }
    });

    world.progress();

    assert!(e1.has::<EntityRef>());
    assert!(e2.has::<EntityRef>());
    assert!(e3.has::<EntityRef>());

    e1.get::<&EntityRef>(|c| {
        assert!(world.entity_from_id(c.value).has::<Velocity>());
    });

    e2.get::<&EntityRef>(|c| {
        assert!(world.entity_from_id(c.value).has::<Velocity>());
    });

    e3.get::<&EntityRef>(|c| {
        assert!(world.entity_from_id(c.value).has::<Velocity>());
    });
}

#[test]
fn system_each_w_mut_children_it() {
    let world = World::new();

    let parent = world.entity().set(Position { x: 0, y: 0 });
    let e1 = world
        .entity()
        .set(Position { x: 0, y: 0 })
        .child_of_id(parent);
    let e2 = world
        .entity()
        .set(Position { x: 0, y: 0 })
        .child_of_id(parent);
    let e3 = world
        .entity()
        .set(Position { x: 0, y: 0 })
        .child_of_id(parent);

    world.set(Count(0));

    world.system::<&Position>().run(|mut it| {
        let world = it.world();
        while it.next() {
            for i in it.iter() {
                it.entity(i).each_child(|child| {
                    child.add::<Velocity>();
                    world.get::<&mut Count>(|c| {
                        c.0 += 1;
                    });
                });
            }
        }
    });

    world.progress();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 3);
    });

    assert!(e1.has::<Velocity>());
    assert!(e2.has::<Velocity>());
    assert!(e3.has::<Velocity>());
}

#[test]
fn system_readonly_children_iter() {
    let world = World::new();

    let parent = world.entity();
    world.entity().set(EntityRef { value: parent.id() });
    world
        .entity()
        .set(Position { x: 1, y: 0 })
        .child_of_id(parent);
    world
        .entity()
        .set(Position { x: 1, y: 0 })
        .child_of_id(parent);
    world
        .entity()
        .set(Position { x: 1, y: 0 })
        .child_of_id(parent);

    world.set(Count(0));

    world.system::<&EntityRef>().run(|mut it| {
        let world = it.world();
        while it.next() {
            let c = it.field::<EntityRef>(0).unwrap();
            for i in it.iter() {
                world.entity_from_id(c[i].value).each_child(|child| {
                    // Dummy code to ensure we can access the entity
                    child.get::<&Position>(|p| {
                        assert_eq!(p.x, 1);
                        assert_eq!(p.y, 0);
                    });

                    world.get::<&mut Count>(|c| {
                        c.0 += 1;
                    });
                });
            }
        }
    });

    world.progress();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 3);
    });
}

#[test]
fn system_rate_filter() {
    let world = World::new();

    #[derive(Default, Component)]
    struct Counter {
        root: i32,
        l1_a: i32,
        l1_b: i32,
        l1_c: i32,
        l2_a: i32,
        l2_b: i32,
    }

    world.set(Counter::default());

    let root_mult: i32 = 1;
    let l1_a_mult: i32 = 1;
    let l1_b_mult: i32 = 2;
    let l1_c_mult: i32 = 3;
    let l2_a_mult: i32 = 2;
    let l2_b_mult: i32 = 4;
    let mut frame_count: i32 = 0;

    let root = world.system_named::<()>("root").run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Counter>(|c| {
                c.root += 1;
            });
        }
    });

    let l1_a = world
        .system_named::<()>("l1_a")
        .rate_w_tick_source(root.id(), 1)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                world.get::<&mut Counter>(|c| {
                    c.l1_a += 1;
                });
            }
        });

    let l1_b = world
        .system_named::<()>("l1_b")
        .rate_w_tick_source(root.id(), 2)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                world.get::<&mut Counter>(|c| {
                    c.l1_b += 1;
                });
            }
        });

    world
        .system_named::<()>("l1_c")
        .rate_w_tick_source(root.id(), 3)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                world.get::<&mut Counter>(|c| {
                    c.l1_c += 1;
                });
            }
        });

    world
        .system_named::<()>("l2_a")
        .rate_w_tick_source(l1_a.id(), 2)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                world.get::<&mut Counter>(|c| {
                    c.l2_a += 1;
                });
            }
        });

    world
        .system_named::<()>("l2_b")
        .rate_w_tick_source(l1_b.id(), 2)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                world.get::<&mut Counter>(|c| {
                    c.l2_b += 1;
                });
            }
        });

    for _i in 0..30 {
        world.progress();
        frame_count += 1;
        world.get::<&Counter>(|c| {
            assert_eq!(c.root, frame_count / root_mult);
            assert_eq!(c.l1_a, frame_count / l1_a_mult);
            assert_eq!(c.l1_b, frame_count / l1_b_mult);
            assert_eq!(c.l1_c, frame_count / l1_c_mult);
            assert_eq!(c.l2_a, frame_count / l2_a_mult);
            assert_eq!(c.l2_b, frame_count / l2_b_mult);
        });
    }
}

#[ignore = "implement missing functions"]
#[test]
fn system_update_rate_filter() {
    let world = World::new();

    #[derive(Default, Component)]
    struct Counter {
        root: i32,
        l1: i32,
        l2: i32,
    }

    world.set(Counter::default());

    let root_mult: i32 = 1;
    let mut l1_mult: i32 = 2;
    let mut l2_mult: i32 = 6;
    let mut frame_count: i32 = 0;

    let root = world.system_named::<()>("root").run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Counter>(|c| {
                c.root += 1;
            });
        }
    });

    let l1 = world
        .system_named::<()>("l1")
        .rate_w_tick_source(root.id(), 2)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                world.get::<&mut Counter>(|c| {
                    c.l1 += 1;
                });
            }
        });

    world
        .system_named::<()>("l2")
        .rate_w_tick_source(l1.id(), 3)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                world.get::<&mut Counter>(|c| {
                    c.l2 += 1;
                });
            }
        });

    for _i in 0..12 {
        world.progress();
        frame_count += 1;
        world.get::<&Counter>(|c| {
            assert_eq!(c.root, frame_count / root_mult);
            assert_eq!(c.l1, frame_count / l1_mult);
            assert_eq!(c.l2, frame_count / l2_mult);
        });
    }

    //FIXME
    //l1.rate(4); // Run twice as slow
    l1_mult *= 2;
    l2_mult *= 2;

    frame_count = 0;
    world.set(Counter::default());

    for _i in 0..32 {
        world.progress();
        frame_count += 1;
        world.get::<&Counter>(|c| {
            assert_eq!(c.root, frame_count / root_mult);
            assert_eq!(c.l1, frame_count / l1_mult);
            assert_eq!(c.l2, frame_count / l2_mult);
        });
    }
}

#[test]
fn system_test_let_defer_each() {
    let world = World::new();

    let e1 = world.entity().add::<Tag>().set(Value { value: 10 });
    let e2 = world.entity().add::<Tag>().set(Value { value: 20 });
    let e3 = world.entity().add::<Tag>().set(Value { value: 30 });

    let s = world
        .system::<&mut Value>()
        .with::<Tag>()
        .each_entity(|e, v| {
            v.value += 1;
            e.remove::<Tag>();
        });

    s.run();

    assert!(!e1.has::<Tag>());
    assert!(!e2.has::<Tag>());
    assert!(!e3.has::<Tag>());

    assert!(e1.has::<Value>());
    assert!(e2.has::<Value>());
    assert!(e3.has::<Value>());

    e1.get::<&Value>(|v| {
        assert_eq!(v.value, 11);
    });

    e2.get::<&Value>(|v| {
        assert_eq!(v.value, 21);
    });

    e3.get::<&Value>(|v| {
        assert_eq!(v.value, 31);
    });
}

#[test]
fn system_test_let_defer_iter() {
    let world = World::new();

    let e1 = world.entity().add::<Tag>().set(Value { value: 10 });
    let e2 = world.entity().add::<Tag>().set(Value { value: 20 });
    let e3 = world.entity().add::<Tag>().set(Value { value: 30 });

    let s = world.system::<&mut Value>().with::<Tag>().run(|mut it| {
        while it.next() {
            let mut v = it.field::<Value>(0).unwrap();
            for i in it.iter() {
                v[i].value += 1;
                it.entity(i).remove::<Tag>();
            }
        }
    });

    s.run();

    assert!(!e1.has::<Tag>());
    assert!(!e2.has::<Tag>());
    assert!(!e3.has::<Tag>());

    assert!(e1.has::<Value>());
    assert!(e2.has::<Value>());
    assert!(e3.has::<Value>());

    e1.get::<&Value>(|v| {
        assert_eq!(v.value, 11);
    });

    e2.get::<&Value>(|v| {
        assert_eq!(v.value, 21);
    });

    e3.get::<&Value>(|v| {
        assert_eq!(v.value, 31);
    });
}

#[ignore = "pre,on,post frame is internal only, replace or delete this test"]
#[test]
fn system_custom_pipeline() {
    // let world = World::new();

    // let mut preFrame = world.entity().add::<flecs::pipeline::Phase>();
    // let OnFrame = world.entity().add::<flecs::pipeline::Phase>().depends_on(PreFrame);
    // let mut postFrame = world.entity().add::<flecs::pipeline::Phase>().depends_on(OnFrame);
    // let tag = world.entity();

    // let pip = world.pipeline()
    //     .with::<flecs::system::System>()
    //     .with(flecs::pipeline::Phase).cascade(flecs::DependsOn)
    //     .with(Tag)
    //     .build();

    // int count = 0;

    // world.system::<()>()
    //     .kind(PostFrame)
    //     .run(|mut it| {
    //         while it.next() {
    //                 world.get::<&Count>(|c| {
    //                 assert_eq!(c.0,2);
    //             });
    //             world.get::<&mut Count>(|c| {
    //                 c.0 += 1;
    //             });
    //         }
    //     })
    //     .add(Tag);

    // world.system::<()>()
    //     .kind(OnFrame)
    //     .run(|mut it| {
    //         while it.next() {
    //             world.get::<&Count>(|c| {
    //                 assert_eq!(c.0,1);
    //             });
    //             world.get::<&mut Count>(|c| {
    //                 c.0 += 1;
    //             });
    //         }
    //     })
    //     .add(Tag);

    // world.system::<()>()
    //     .kind(PreFrame)
    //     .run(|mut it| {
    //         while it.next() {
    //             assert_eq!(count, 0);
    //             world.get::<&mut Count>(|c| {
    //                 c.0 += 1;
    //             });
    //         }
    //     })
    //     .add(Tag);

    // assert_eq!(count, 0);

    // world.set_pipeline_id(pip);

    // world.progress();

    //     world.get::<&Count>(|c| {
    //                 assert_eq!(c.0,3);
    //             });
}

#[test]
fn system_custom_pipeline_w_kind() {
    let world = World::new();

    let tag = world.entity();

    let pip = world
        .pipeline()
        .with::<flecs::system::System>()
        .with_id(tag)
        .build();

    world.set(Count(0));

    world.system::<()>().kind_id(tag).run(|mut it| {
        while it.next() {
            let world = it.world();
            world.get::<&mut Count>(|c| {
                assert_eq!(c, 0);
                c.0 += 1;
            });
        }
    });

    world.system::<()>().kind_id(tag).run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Count>(|c| {
                assert_eq!(c.0, 1);
                c.0 += 1;
            });
        }
    });

    world.system::<()>().kind_id(tag).run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Count>(|c| {
                assert_eq!(c.0, 2);
                c.0 += 1;
            });
        }
    });

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 0);
    });

    world.set_pipeline_id(pip.id());

    world.progress();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 3);
    });
}

#[test]
fn system_instanced_query_w_singleton_each() {
    let world = World::new();

    world.set(Velocity { x: 1, y: 2 });

    let e1 = world.entity().set(Position { x: 10, y: 20 });
    e1.set(SelfRef { value: e1.id() });
    let e2 = world.entity().set(Position { x: 20, y: 30 });
    e2.set(SelfRef { value: e2.id() });
    let e3 = world.entity().set(Position { x: 30, y: 40 });
    e3.set(SelfRef { value: e3.id() });
    let e4 = world.entity().set(Position { x: 40, y: 50 });
    e4.set(SelfRef { value: e4.id() });
    let e5 = world.entity().set(Position { x: 50, y: 60 });
    e5.set(SelfRef { value: e5.id() });

    e4.add::<Tag>();
    e5.add::<Tag>();

    world.set(Count(0));

    let s = world
        .system::<(&SelfRef, &mut Position, &Velocity)>()
        .term_at(2)
        .singleton()
        .instanced()
        .each_entity(|e, (s, p, v)| {
            let world = e.world();
            assert!(e == s.value);
            p.x += v.x;
            p.y += v.y;
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        });

    s.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 5);
    });

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 21);
        assert_eq!(p.y, 32);
    });

    e3.get::<&Position>(|p| {
        assert_eq!(p.x, 31);
        assert_eq!(p.y, 42);
    });

    e4.get::<&Position>(|p| {
        assert_eq!(p.x, 41);
        assert_eq!(p.y, 52);
    });

    e5.get::<&Position>(|p| {
        assert_eq!(p.x, 51);
        assert_eq!(p.y, 62);
    });
}

#[test]
fn system_instanced_query_w_base_each() {
    let world = World::new();

    let base = world.entity().set(Velocity { x: 1, y: 2 });

    let e1 = world.entity().is_a_id(base).set(Position { x: 10, y: 20 });
    e1.set(SelfRef { value: e1.id() });
    let e2 = world.entity().is_a_id(base).set(Position { x: 20, y: 30 });
    e2.set(SelfRef { value: e2.id() });
    let e3 = world.entity().is_a_id(base).set(Position { x: 30, y: 40 });
    e3.set(SelfRef { value: e3.id() });
    let e4 = world
        .entity()
        .is_a_id(base)
        .set(Position { x: 40, y: 50 })
        .add::<Tag>();
    e4.set(SelfRef { value: e4.id() });
    let e5 = world
        .entity()
        .is_a_id(base)
        .set(Position { x: 50, y: 60 })
        .add::<Tag>();
    e5.set(SelfRef { value: e5.id() });
    let e6 = world
        .entity()
        .set(Position { x: 60, y: 70 })
        .set(Velocity { x: 2, y: 3 });
    e6.set(SelfRef { value: e6.id() });
    let e7 = world
        .entity()
        .set(Position { x: 70, y: 80 })
        .set(Velocity { x: 4, y: 5 });
    e7.set(SelfRef { value: e7.id() });

    world.set(Count(0));
    let s = world
        .system::<(&SelfRef, &mut Position, &Velocity)>()
        .instanced()
        .each_entity(|e, (s, p, v)| {
            let world = e.world();
            assert!(e == s.value);
            p.x += v.x;
            p.y += v.y;
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        });

    s.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 7);
    });

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 21);
        assert_eq!(p.y, 32);
    });

    e3.get::<&Position>(|p| {
        assert_eq!(p.x, 31);
        assert_eq!(p.y, 42);
    });

    e4.get::<&Position>(|p| {
        assert_eq!(p.x, 41);
        assert_eq!(p.y, 52);
    });

    e5.get::<&Position>(|p| {
        assert_eq!(p.x, 51);
        assert_eq!(p.y, 62);
    });

    e6.get::<&Position>(|p| {
        assert_eq!(p.x, 62);
        assert_eq!(p.y, 73);
    });

    e7.get::<&Position>(|p| {
        assert_eq!(p.x, 74);
        assert_eq!(p.y, 85);
    });
}

#[test]
fn system_un_instanced_query_w_singleton_each() {
    let world = World::new();

    world.set(Velocity { x: 1, y: 2 });

    let e1 = world.entity().set(Position { x: 10, y: 20 });
    e1.set(SelfRef { value: e1.id() });
    let e2 = world.entity().set(Position { x: 20, y: 30 });
    e2.set(SelfRef { value: e2.id() });
    let e3 = world.entity().set(Position { x: 30, y: 40 });
    e3.set(SelfRef { value: e3.id() });
    let e4 = world.entity().set(Position { x: 40, y: 50 });
    e4.set(SelfRef { value: e4.id() });
    let e5 = world.entity().set(Position { x: 50, y: 60 });
    e5.set(SelfRef { value: e5.id() });

    e4.add::<Tag>();
    e5.add::<Tag>();

    world.set(Count(0));

    let s = world
        .system::<(&SelfRef, &mut Position, &Velocity)>()
        .term_at(2)
        .singleton()
        .each_entity(|e, (s, p, v)| {
            let world = e.world();
            assert!(e == s.value);
            p.x += v.x;
            p.y += v.y;
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        });

    s.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 5);
    });

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 21);
        assert_eq!(p.y, 32);
    });

    e3.get::<&Position>(|p| {
        assert_eq!(p.x, 31);
        assert_eq!(p.y, 42);
    });

    e4.get::<&Position>(|p| {
        assert_eq!(p.x, 41);
        assert_eq!(p.y, 52);
    });

    e5.get::<&Position>(|p| {
        assert_eq!(p.x, 51);
        assert_eq!(p.y, 62);
    });
}

#[test]
fn system_un_instanced_query_w_base_each() {
    let world = World::new();

    let base = world.entity().set(Velocity { x: 1, y: 2 });

    let e1 = world.entity().is_a_id(base).set(Position { x: 10, y: 20 });
    e1.set(SelfRef { value: e1.id() });
    let e2 = world.entity().is_a_id(base).set(Position { x: 20, y: 30 });
    e2.set(SelfRef { value: e2.id() });
    let e3 = world.entity().is_a_id(base).set(Position { x: 30, y: 40 });
    e3.set(SelfRef { value: e3.id() });
    let e4 = world
        .entity()
        .is_a_id(base)
        .set(Position { x: 40, y: 50 })
        .add::<Tag>();
    e4.set(SelfRef { value: e4.id() });
    let e5 = world
        .entity()
        .is_a_id(base)
        .set(Position { x: 50, y: 60 })
        .add::<Tag>();
    e5.set(SelfRef { value: e5.id() });
    let e6 = world
        .entity()
        .set(Position { x: 60, y: 70 })
        .set(Velocity { x: 2, y: 3 });
    e6.set(SelfRef { value: e6.id() });
    let e7 = world
        .entity()
        .set(Position { x: 70, y: 80 })
        .set(Velocity { x: 4, y: 5 });
    e7.set(SelfRef { value: e7.id() });

    world.set(Count(0));

    let s = world
        .system::<(&SelfRef, &mut Position, &Velocity)>()
        .each_entity(|e, (s, p, v)| {
            let world = e.world();
            assert!(e == s.value);
            p.x += v.x;
            p.y += v.y;
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        });

    s.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 7);
    });

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 21);
        assert_eq!(p.y, 32);
    });

    e3.get::<&Position>(|p| {
        assert_eq!(p.x, 31);
        assert_eq!(p.y, 42);
    });

    e4.get::<&Position>(|p| {
        assert_eq!(p.x, 41);
        assert_eq!(p.y, 52);
    });

    e5.get::<&Position>(|p| {
        assert_eq!(p.x, 51);
        assert_eq!(p.y, 62);
    });

    e6.get::<&Position>(|p| {
        assert_eq!(p.x, 62);
        assert_eq!(p.y, 73);
    });

    e7.get::<&Position>(|p| {
        assert_eq!(p.x, 74);
        assert_eq!(p.y, 85);
    });
}

#[test]
fn system_instanced_query_w_singleton_iter() {
    let world = World::new();

    world.set(Velocity { x: 1, y: 2 });

    let e1 = world.entity().set(Position { x: 10, y: 20 });
    e1.set(SelfRef { value: e1.id() });
    let e2 = world.entity().set(Position { x: 20, y: 30 });
    e2.set(SelfRef { value: e2.id() });
    let e3 = world.entity().set(Position { x: 30, y: 40 });
    e3.set(SelfRef { value: e3.id() });
    let e4 = world.entity().set(Position { x: 40, y: 50 });
    e4.set(SelfRef { value: e4.id() });
    let e5 = world.entity().set(Position { x: 50, y: 60 });
    e5.set(SelfRef { value: e5.id() });

    e4.add::<Tag>();
    e5.add::<Tag>();

    world.set(Count(0));

    let s = world
        .system::<(&SelfRef, &mut Position, &Velocity)>()
        .term_at(2)
        .singleton()
        .instanced()
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                let s = it.field::<SelfRef>(0).unwrap();
                let mut p = it.field::<Position>(1).unwrap();
                let v = it.field::<Velocity>(2).unwrap();

                assert!(it.count() > 1);

                for i in it.iter() {
                    p[i].x += v[0].x;
                    p[i].y += v[0].y;
                    assert!(it.entity(i) == s[i].value);
                    world.get::<&mut Count>(|c| {
                        c.0 += 1;
                    });
                }
            }
        });

    s.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 5);
    });

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 21);
        assert_eq!(p.y, 32);
    });

    e3.get::<&Position>(|p| {
        assert_eq!(p.x, 31);
        assert_eq!(p.y, 42);
    });

    e4.get::<&Position>(|p| {
        assert_eq!(p.x, 41);
        assert_eq!(p.y, 52);
    });

    e5.get::<&Position>(|p| {
        assert_eq!(p.x, 51);
        assert_eq!(p.y, 62);
    });
}

#[test]
fn system_instanced_query_w_singleton_each_macro() {
    let world = World::new();

    world.set(Velocity { x: 1, y: 2 });

    let e1 = world.entity().set(Position { x: 10, y: 20 });
    e1.set(SelfRef { value: e1.id() });
    let e2 = world.entity().set(Position { x: 20, y: 30 });
    e2.set(SelfRef { value: e2.id() });
    let e3 = world.entity().set(Position { x: 30, y: 40 });
    e3.set(SelfRef { value: e3.id() });
    let e4 = world.entity().set(Position { x: 40, y: 50 });
    e4.set(SelfRef { value: e4.id() });
    let e5 = world.entity().set(Position { x: 50, y: 60 });
    e5.set(SelfRef { value: e5.id() });

    e4.add::<Tag>();
    e5.add::<Tag>();

    world.set(Count(0));

    let s = system!(world, &SelfRef, &mut Position, &Velocity($))
        .term_at(2)
        .instanced()
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                let s = it.field::<SelfRef>(0).unwrap();
                let mut p = it.field::<Position>(1).unwrap();
                let v = it.field::<Velocity>(2).unwrap();

                assert!(it.count() > 1);

                for i in it.iter() {
                    p[i].x += v[0].x;
                    p[i].y += v[0].y;
                    assert!(it.entity(i) == s[i].value);
                    world.get::<&mut Count>(|c| {
                        c.0 += 1;
                    });
                }
            }
        });

    s.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 5);
    });

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 21);
        assert_eq!(p.y, 32);
    });

    e3.get::<&Position>(|p| {
        assert_eq!(p.x, 31);
        assert_eq!(p.y, 42);
    });

    e4.get::<&Position>(|p| {
        assert_eq!(p.x, 41);
        assert_eq!(p.y, 52);
    });

    e5.get::<&Position>(|p| {
        assert_eq!(p.x, 51);
        assert_eq!(p.y, 62);
    });
}

#[test]
fn system_instanced_query_w_base_iter() {
    let world = World::new();

    let base = world.entity().set(Velocity { x: 1, y: 2 });

    let e1 = world.entity().is_a_id(base).set(Position { x: 10, y: 20 });
    e1.set(SelfRef { value: e1.id() });
    let e2 = world.entity().is_a_id(base).set(Position { x: 20, y: 30 });
    e2.set(SelfRef { value: e2.id() });
    let e3 = world.entity().is_a_id(base).set(Position { x: 30, y: 40 });
    e3.set(SelfRef { value: e3.id() });
    let e4 = world
        .entity()
        .is_a_id(base)
        .set(Position { x: 40, y: 50 })
        .add::<Tag>();
    e4.set(SelfRef { value: e4.id() });
    let e5 = world
        .entity()
        .is_a_id(base)
        .set(Position { x: 50, y: 60 })
        .add::<Tag>();
    e5.set(SelfRef { value: e5.id() });
    let e6 = world
        .entity()
        .set(Position { x: 60, y: 70 })
        .set(Velocity { x: 2, y: 3 });
    e6.set(SelfRef { value: e6.id() });
    let e7 = world
        .entity()
        .set(Position { x: 70, y: 80 })
        .set(Velocity { x: 4, y: 5 });
    e7.set(SelfRef { value: e7.id() });

    world.set(Count(0));

    let s = world
        .system::<(&SelfRef, &mut Position, &Velocity)>()
        .instanced()
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                let s = it.field::<SelfRef>(0).unwrap();
                let mut p = it.field::<Position>(1).unwrap();
                let v = it.field::<&Velocity>(2).unwrap();
                assert!(it.count() > 1);

                for i in it.iter() {
                    if it.is_self(2) {
                        p[i].x += v[i].x;
                        p[i].y += v[i].y;
                    } else {
                        p[i].x += v[0].x;
                        p[i].y += v[0].y;
                    }

                    assert!(it.entity(i) == s[i].value);
                    world.get::<&mut Count>(|c| {
                        c.0 += 1;
                    });
                }
            }
        });

    s.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 7);
    });

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 21);
        assert_eq!(p.y, 32);
    });

    e3.get::<&Position>(|p| {
        assert_eq!(p.x, 31);
        assert_eq!(p.y, 42);
    });

    e4.get::<&Position>(|p| {
        assert_eq!(p.x, 41);
        assert_eq!(p.y, 52);
    });

    e5.get::<&Position>(|p| {
        assert_eq!(p.x, 51);
        assert_eq!(p.y, 62);
    });

    e6.get::<&Position>(|p| {
        assert_eq!(p.x, 62);
        assert_eq!(p.y, 73);
    });

    e7.get::<&Position>(|p| {
        assert_eq!(p.x, 74);
        assert_eq!(p.y, 85);
    });
}

#[test]
fn system_un_instanced_query_w_singleton_iter() {
    let world = World::new();

    world.set(Velocity { x: 1, y: 2 });

    let e1 = world.entity().set(Position { x: 10, y: 20 });
    e1.set(SelfRef { value: e1.id() });
    let e2 = world.entity().set(Position { x: 20, y: 30 });
    e2.set(SelfRef { value: e2.id() });
    let e3 = world.entity().set(Position { x: 30, y: 40 });
    e3.set(SelfRef { value: e3.id() });
    let e4 = world.entity().set(Position { x: 40, y: 50 });
    e4.set(SelfRef { value: e4.id() });
    let e5 = world.entity().set(Position { x: 50, y: 60 });
    e5.set(SelfRef { value: e5.id() });

    e4.add::<Tag>();
    e5.add::<Tag>();

    world.set(Count(0));

    let s = world
        .system::<(&SelfRef, &mut Position, &Velocity)>()
        .term_at(2)
        .singleton()
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                let s = it.field::<SelfRef>(0).unwrap();
                let mut p = it.field::<Position>(1).unwrap();
                let v = it.field::<&Velocity>(2).unwrap();

                assert!(it.count() == 1);

                for i in it.iter() {
                    p[i].x += v[i].x;
                    p[i].y += v[i].y;
                    assert!(it.entity(i) == s[i].value);
                    world.get::<&mut Count>(|c| {
                        c.0 += 1;
                    });
                }
            }
        });

    s.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 5);
    });

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 21);
        assert_eq!(p.y, 32);
    });

    e3.get::<&Position>(|p| {
        assert_eq!(p.x, 31);
        assert_eq!(p.y, 42);
    });

    e4.get::<&Position>(|p| {
        assert_eq!(p.x, 41);
        assert_eq!(p.y, 52);
    });

    e5.get::<&Position>(|p| {
        assert_eq!(p.x, 51);
        assert_eq!(p.y, 62);
    });
}

#[test]
fn system_un_instanced_query_w_base_iter() {
    let world = World::new();

    let base = world.entity().set(Velocity { x: 1, y: 2 });

    let e1 = world.entity().is_a_id(base).set(Position { x: 10, y: 20 });
    e1.set(SelfRef { value: e1.id() });
    let e2 = world.entity().is_a_id(base).set(Position { x: 20, y: 30 });
    e2.set(SelfRef { value: e2.id() });
    let e3 = world.entity().is_a_id(base).set(Position { x: 30, y: 40 });
    e3.set(SelfRef { value: e3.id() });
    let e4 = world
        .entity()
        .is_a_id(base)
        .set(Position { x: 40, y: 50 })
        .add::<Tag>();
    e4.set(SelfRef { value: e4.id() });
    let e5 = world
        .entity()
        .is_a_id(base)
        .set(Position { x: 50, y: 60 })
        .add::<Tag>();
    e5.set(SelfRef { value: e5.id() });
    let e6 = world
        .entity()
        .set(Position { x: 60, y: 70 })
        .set(Velocity { x: 2, y: 3 });
    e6.set(SelfRef { value: e6.id() });
    let e7 = world
        .entity()
        .set(Position { x: 70, y: 80 })
        .set(Velocity { x: 4, y: 5 });
    e7.set(SelfRef { value: e7.id() });

    world.set(Count(0));

    let s = world
        .system::<(&SelfRef, &mut Position, &Velocity)>()
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                let s = it.field::<SelfRef>(0).unwrap();
                let mut p = it.field::<Position>(1).unwrap();
                let v = it.field::<&Velocity>(2).unwrap();

                for i in it.iter() {
                    p[i].x += v[i].x;
                    p[i].y += v[i].y;
                    assert!(it.entity(i) == s[i].value);
                    world.get::<&mut Count>(|c| {
                        c.0 += 1;
                    });
                }
            }
        });

    s.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 7);
    });

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 21);
        assert_eq!(p.y, 32);
    });

    e3.get::<&Position>(|p| {
        assert_eq!(p.x, 31);
        assert_eq!(p.y, 42);
    });

    e4.get::<&Position>(|p| {
        assert_eq!(p.x, 41);
        assert_eq!(p.y, 52);
    });

    e5.get::<&Position>(|p| {
        assert_eq!(p.x, 51);
        assert_eq!(p.y, 62);
    });

    e6.get::<&Position>(|p| {
        assert_eq!(p.x, 62);
        assert_eq!(p.y, 73);
    });

    e7.get::<&Position>(|p| {
        assert_eq!(p.x, 74);
        assert_eq!(p.y, 85);
    });
}

#[test]
fn system_create_w_no_template_args() {
    let world = World::new();

    let entity = world.entity().set(Position { x: 10, y: 20 });

    let entity_id = entity.id();
    world.set(Count(0));

    let s = world
        .system::<()>()
        .with::<Position>()
        .each_entity(move |e, _| {
            let world = e.world();
            assert!(e == entity_id);
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        });

    s.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 1);
    });
}

#[derive(Component)]
struct PipelineType;
#[derive(Component)]
struct First;
#[derive(Component)]
struct Second;

#[test]
fn system_system_w_type_kind_type_pipeline() {
    let world = World::new();

    world
        .component::<Second>()
        .add::<flecs::pipeline::Phase>()
        .depends_on_id(world.component::<First>().add::<flecs::pipeline::Phase>());

    world
        .pipeline_type::<PipelineType>()
        .with::<flecs::system::System>()
        .with::<flecs::pipeline::Phase>()
        .cascade_type::<flecs::DependsOn>()
        .build();

    world.set_pipeline::<PipelineType>();

    let entity = world.entity().add::<Tag>();
    let entity_id = entity.id();

    world.set(Count2 { a: 0, b: 0 });

    world
        .system::<&Tag>()
        .kind::<Second>()
        .each_entity(move |e, _tag| {
            let world = e.world();
            assert!(e == entity_id);
            world.get::<&mut Count2>(|c| {
                assert_eq!(c.a, 0);
                assert_eq!(c.b, 1);
                c.a += 1;
            });
        });

    world
        .system::<&Tag>()
        .kind::<First>()
        .each_entity(move |e, _tag| {
            let world = e.world();
            assert!(e == entity_id);
            world.get::<&mut Count2>(|c| {
                assert_eq!(c.b, 0);
                c.b += 1;
            });
        });

    world.progress();

    world.get::<&Count2>(|c| {
        assert_eq!(c.a, 1);
        assert_eq!(c.b, 1);
    });
}

#[test]
fn system_entity_ctor() {
    let world = World::new();

    world.set(Count(0));

    let sys = world.system::<()>().run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        }
    });

    let sys_from_id = world.system_from(sys.entity_view(&world));

    sys_from_id.run();
    world.get::<&Count>(|c| {
        assert_eq!(c.0, 1);
    });
}

#[test]
fn system_ensure_instanced_w_each() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 10, y: 20 });
    let e1_id = e1.id();

    world.set(Count(0));

    let sys = world
        .system::<&mut Position>()
        .each_iter(move |mut it, i, _p| {
            let world = it.world();
            assert!(it.iter_mut().flags & flecs_ecs_sys::EcsIterIsInstanced != 0);
            assert!(it.entity(i) == e1_id);
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        });

    let q = sys.query();
    assert!(unsafe { (*q.query_ptr()).flags } & flecs_ecs_sys::EcsQueryIsInstanced != 0);

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 0);
    });
    sys.run();
    world.get::<&Count>(|c| {
        assert_eq!(c.0, 1);
    });
}

#[test]
fn system_multithread_system_w_query_each() {
    let world = World::new();

    world.set_threads(2);

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.new_query::<&Velocity>();

    world
        .system::<&mut Position>()
        .multi_threaded()
        .each_entity(move |e, p| {
            let world = e.world();
            q.iter_stage(world).each(|v| {
                p.x += v.x;
                p.y += v.y;
            });
        });

    world.progress();

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}

#[test]
fn system_multithread_system_w_query_each_w_iter() {
    let world = World::new();

    world.set_threads(2);

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.new_query::<&Velocity>();

    world
        .system::<&mut Position>()
        .multi_threaded()
        .each_iter(move |it, _i, p| {
            q.iter_stage(it.world()).each(|v| {
                p.x += v.x;
                p.y += v.y;
            });
        });

    world.progress();

    e.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}

#[test]
fn system_multithread_system_w_query_each_w_world() {
    let world = World::new();

    world.set_threads(2);

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.new_query::<&Velocity>();
    world
        .system::<&mut Position>()
        .multi_threaded()
        .each_iter(move |it, _i, p| {
            let world = it.world();
            q.iter_stage(world).each(|v| {
                p.x += v.x;
                p.y += v.y;
            });
        });

    world.progress();

    e.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}

#[test]
fn system_multithread_system_w_query_iter() {
    let world = World::new();

    world.set_threads(2);

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.new_query::<&Velocity>();

    world
        .system::<&mut Position>()
        .multi_threaded()
        .each_entity(move |e, p| {
            q.iter_stage(e).run(|mut it| {
                while it.next() {
                    let v = it.field::<Velocity>(0).unwrap();

                    for i in it.iter() {
                        p.x += v[i].x;
                        p.y += v[i].y;
                    }
                }
            });
        });

    world.progress();

    e.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}

#[test]
fn system_multithread_system_w_query_iter_w_iter() {
    let world = World::new();

    world.set_threads(2);

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.new_query::<&Velocity>();

    world
        .system::<&mut Position>()
        .multi_threaded()
        .each_iter(move |it, _i, p| {
            q.iter_stage(it.world()).run(|mut it| {
                while it.next() {
                    let v = it.field::<Velocity>(0).unwrap();
                    for i in it.iter() {
                        p.x += v[i].x;
                        p.y += v[i].y;
                    }
                }
            });
        });

    world.progress();

    e.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}

#[test]
fn system_multithread_system_w_query_iter_w_world() {
    let world = World::new();

    world.set_threads(2);

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.new_query::<&Velocity>();

    world
        .system::<&mut Position>()
        .multi_threaded()
        .each_iter(move |it, _i, p| {
            q.iter_stage(it.world()).run(|mut it| {
                while it.next() {
                    let v = it.field::<Velocity>(0).unwrap();
                    for i in it.iter() {
                        p.x += v[i].x;
                        p.y += v[i].y;
                    }
                }
            });
        });

    world.progress();

    e.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}

#[test]
fn system_run_callback() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    world.system::<(&mut Position, &Velocity)>().run_each(
        |mut it| {
            while it.next() {
                it.each();
            }
        },
        |(p, v)| {
            p.x += v.x;
            p.y += v.y;
        },
    );

    world.progress();

    entity.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    entity.get::<&Velocity>(|v| {
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });
}

#[test]
fn system_startup_system() {
    let world = World::new();

    world.set(Count2 { a: 0, b: 0 });

    world
        .system::<()>()
        .kind_id(flecs::pipeline::OnStart::ID)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                assert!(it.delta_time() == 0.0);
                world.get::<&mut Count2>(|c| {
                    c.a += 1;
                });
            }
        });

    world
        .system::<()>()
        .kind_id(flecs::pipeline::OnUpdate::ID)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                assert!(it.delta_time() != 0.0);
                world.get::<&mut Count2>(|c| {
                    c.b += 1;
                });
            }
        });

    world.progress();
    world.get::<&Count2>(|c| {
        assert_eq!(c.a, 1);
        assert_eq!(c.b, 1);
    });
    world.progress();
    world.get::<&Count2>(|c| {
        assert_eq!(c.a, 1);
        assert_eq!(c.b, 2);
    });
}

#[ignore = "timer addon not implemented"]
#[test]
fn system_interval_tick_source() {
    //let world = World::new();

    // flecs::timer t = world.timer().interval(2.1);

    // flecs::Timer& timer = t.ensure<flecs::Timer>();
    // timer.time = 0;

    // world.set(Count2 { a: 0, b: 0 });

    // world.system::<()>()
    //     .tick_source(t)
    //     .run(|mut it| {
    //         let world = it.world();
    //         while it.next() {
    //             world.get::<&mut Count2>(|c| {
    //                 c.a += 1;
    //             });
    //         }
    //     });

    // world.system::<()>()
    //     .tick_source(t)
    //     .run(|mut it| {
    //         let world = it.world();
    //         while it.next() {
    //             world.get::<&mut Count2>(|c| {
    //                 c.b += 1;
    //             });
    //         }
    //     });

    // world.get::<&Count2>(|c| {
    //     world.progress_time(1.0);
    //     assert_eq!(c.a, 0);
    //     assert_eq!(c.b, 0);

    //     world.progress_time(1.0);

    //     assert_eq!(c.a, 0);
    //     assert_eq!(c.b, 0);

    //     world.progress_time(1.0);
    //     assert_eq!(c.a, 1);
    //     assert_eq!(c.b, 1);
    // });
}

#[ignore = "timer addon not implemented"]
#[test]
fn system_rate_tick_source() {
    // let world = World::new();

    // flecs::timer t = world.timer().rate(3);

    // int32_t sys_a_invoked = 0, sys_b_invoked = 0;

    // world.system::<()>()
    //     .tick_source(t)
    //     .run(|mut it| {
    //         while it.next() {
    //             sys_a_world.get::<&mut Count>(|c| {
    //                 c.0 += 1;
    //             });
    //         }
    //     });

    // world.system::<()>()
    //     .tick_source(t)
    //     .run(|mut it| {
    //         while it.next() {
    //             sys_b_world.get::<&mut Count>(|c| {
    //                 c.0 += 1;
    //             });
    //         }
    //     });

    // world.progress(1.0);
    // assert_eq!(0, sys_a_invoked);
    // assert_eq!(0, sys_b_invoked);

    // world.progress(1.0);
    // assert_eq!(0, sys_a_invoked);
    // assert_eq!(0, sys_b_invoked);

    // world.progress(1.0);
    // assert_eq!(1, sys_a_invoked);
    // assert_eq!(1, sys_b_invoked);
}

#[ignore = "timer addon not implemented"]
#[test]
fn system_nested_rate_tick_source() {
    // let world = World::new();

    // flecs::timer t_3 = world.timer().rate(3);
    // flecs::timer t_6 = world.timer().rate(2, t_3);

    // int32_t sys_a_invoked = 0, sys_b_invoked = 0;

    // world.system::<()>()
    //     .tick_source(t_3)
    //     .run(|mut it| {
    //         while it.next() {
    //             sys_a_world.get::<&mut Count>(|c| {
    //                 c.0 += 1;
    //             });
    //         }
    //     });

    // world.system::<()>()
    //     .tick_source(t_6)
    //     .run(|mut it| {
    //         while it.next() {
    //             sys_b_world.get::<&mut Count>(|c| {
    //                 c.0 += 1;
    //             });
    //         }
    //     });

    // world.progress(1.0);
    // assert_eq!(0, sys_a_invoked);
    // assert_eq!(0, sys_b_invoked);

    // world.progress(1.0);
    // assert_eq!(0, sys_a_invoked);
    // assert_eq!(0, sys_b_invoked);

    // world.progress(1.0);
    // assert_eq!(1, sys_a_invoked);
    // assert_eq!(0, sys_b_invoked);

    // world.progress(1.0);
    // assert_eq!(1, sys_a_invoked);
    // assert_eq!(0, sys_b_invoked);

    // world.progress(1.0);
    // assert_eq!(1, sys_a_invoked);
    // assert_eq!(0, sys_b_invoked);

    // world.progress(1.0);
    // assert_eq!(2, sys_a_invoked);
    // assert_eq!(1, sys_b_invoked);
}

// #[test] fn system_table_get() {
//     let world = World::new();

//     let e1 = world.entity().set(Position{x: 10, y: 20});
//     flecs::entity e2 = world.entity().set(Position{x: 20, y: 30});

//     let s = world.system::<()>()
//         .with::<Position>()
//         .each([&](flecs::iter& iter, size_t index) {
//             let e = iter.entity(index);
//             &Position *p = &iter.table().get<Position>()[index];
//             assert!(p != nullptr);
//             assert!(e == e1 || e == e2);
//             if (e == e1) {
//                 assert_eq!(p.x, 10);
//                 assert_eq!(p.y, 20);
//             } else if (e == e2) {
//                 assert_eq!(p.x, 20);
//                 assert_eq!(p.y, 30);
//             }
//         });

//     s.run();
// }

// #[test] fn system_range_get() {
//     let world = World::new();

//     let e1 = world.entity().set(Position{x: 10, y: 20});
//     flecs::entity e2 = world.entity().set(Position{x: 20, y: 30});

//     let s = world.system::<()>()
//         .with::<Position>()
//         .each([&](flecs::iter& iter, size_t index) {
//             let e = iter.entity(index);
//             &Position *p = &iter.range().get<Position>()[index];
//             assert!(p != nullptr);
//             assert!(e == e1 || e == e2);
//             if (e == e1) {
//                 assert_eq!(p.x, 10);
//                 assert_eq!(p.y, 20);
//             } else if (e == e2) {
//                 assert_eq!(p.x, 20);
//                 assert_eq!(p.y, 30);
//             }
//         });

//     s.run();
// }

// #[test] fn system_randomize_timers() {
//     let world = World::new();

//     flecs::entity s1 = world.system::<()>()
//         .interval(1.0)
//         .run(|mut it| { while it.next() {} });

//     {
//         const flecs::Timer *t = s1.get<flecs::Timer>();
//         assert!(t != nullptr);
//         assert!(t.time == 0);
//     }

//     world.randomize_timers();

//     flecs::entity s2 = world.system::<()>()
//         .interval(1.0)
//         .run(|mut it| { while it.next() {} });

//     {
//         const flecs::Timer *t = s1.get<flecs::Timer>();
//         assert!(t != nullptr);
//         assert!(t.time != 0);
//     }

//     {
//         const flecs::Timer *t = s2.get<flecs::Timer>();
//         assert!(t != nullptr);
//         assert!(t.time != 0);
//     }
// }

// #[test] fn system_optional_pair_term() {
//     let world = World::new();

//     world.entity()
//         .add::<TagA>()
//         .emplace<Position, Tag>(1.0f, 2.0f);
//     world.entity()
//         .add::<TagA>();

//     int32_t with_pair = 0, without_pair = 0;

//     world.system::<flecs::pair<Position, Tag>*>()
//         .with::<TagA>()
//         .each_entity(|e, Position* p)
//         {
//             if (p)
//             {
//                 with_pair++;
//                 test_flt(1.0f, p.x);
//                 test_flt(2.0f, p.y);
//             }
//             else
//             {
//                 without_pair++;
//             }
//         });

//     world.progress(1.0);

//     assert_eq!(1, with_pair);
//     assert_eq!(1, without_pair);
// }

// #[test] fn system_singleton_tick_source() {
//     let world = World::new();

//     world.timer<TagA>().timeout(1.5);

//     int32_t sys_invoked = 0;

//     world.system::<()>()
//         .tick_source<TagA>()
//         .run(|mut it| {
//             sys_world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         });

//     world.progress(1.0);
//     assert_eq!(0, sys_invoked);

//     world.progress(1.0);
//     assert_eq!(1, sys_invoked);

//     world.progress(2.0);
//     assert_eq!(1, sys_invoked);
// }

// enum class PipelineStepEnum
// {
//     CustomStep,
//     CustomStep2
// };

// #[test] fn system_pipeline_step_with_kind_enum() {
//     let world = World::new();

//     world.entity(PipelineStepEnum::CustomStep).add::<flecs::pipeline::Phase>().depends_on(flecs::OnStart);

//     bool ran_test = false;

//     world.system::<()>().kind(PipelineStepEnum::CustomStep)
//         .run([&ran_test](flecs::iter& it) {
//             while it.next() {
//                 ran_test = true;
//             }
//         });

//     world.progress();
//     assert!(ran_test);
// }

// #[test] fn system_pipeline_step_depends_on_pipeline_step_with_enum() {
//     let world = World::new();

//     world.entity(PipelineStepEnum::CustomStep).add::<flecs::pipeline::Phase>().depends_on(flecs::OnStart);
//     world.entity(PipelineStepEnum::CustomStep2).add::<flecs::pipeline::Phase>().depends_on(PipelineStepEnum::CustomStep);

//     bool ran_test = false;

//     world.system::<()>().kind(PipelineStepEnum::CustomStep2)
//         .run([&ran_test](flecs::iter& it) {
//             while it.next() {
//                 ran_test = true;
//             }
//         });

//     world.progress();
//     assert!(ran_test);
// }

// ////////////////////////////////////////
// ////////////////////////////////////////
// ////////////////////////////////////////
// ////////////////////////////////////////
// ////////////////////////////////////////
// ////////////////////////////////////////
// ////////////////////////////////////////
// ////////////////////////////////////////
// ///
// ///

// #include <cpp.h>

// #[test] fn system_builder_builder_assign_same_type() {
//     let world = World::new();

//     let e1 = world.entity().add::<Position>().add::<Velocity>();
//     world.entity().add::<Position>();

//     world.set(Count(0));

//     flecs::system::System s =
//         world.system::<(&mut Position, &mut Velocity)>()
//             .each_entity(|e, (p,v)| {
//                 world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//                 assert!(e == e1);
//             });

//     assert_eq!(count, 0);
//     s.run();
//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn system_builder_builder_build_to_let() {
//     let world = World::new();

//     let e1 = world.entity().add::<Position>().add::<Velocity>();
//     world.entity().add::<Position>();

//     world.set(Count(0));

//     let s = world.system::<(&mut Position, &mut Velocity)>()
//         .each_entity(|e, (p,v)| {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             assert!(e == e1);
//         });

//     assert_eq!(count, 0);
//     s.run();
//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn system_builder_builder_build_n_statements() {
//     let world = World::new();

//     let e1 = world.entity().add::<Position>().add::<Velocity>();
//     world.entity().add::<Position>();

//     world.set(Count(0));

//     let qb = world.system::<()>();
//     qb.with::<Position>();
//     qb.with::<Velocity>();
//     let s = qb.each_entity(|e| {
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         assert!(e == e1);
//     });

//     s.run();

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn system_builder_1_type() {
//     let world = World::new();

//     let e1 = world.entity().add::<Position>();
//     world.entity().add::<Velocity>();

//     world.set(Count(0));

//     let s = world.system::<&mut Position>()
//         .each_entity(|e, p| {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             assert!(e == e1);
//         });

//     assert_eq!(count, 0);
//     s.run();
//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn system_builder_add_1_type() {
//     let world = World::new();

//     let e1 = world.entity().add::<Position>();
//     world.entity().add::<Velocity>();

//     world.set(Count(0));

//     let s = world.system::<()>()
//         .with::<Position>()
//         .each_entity(|e| {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             assert!(e == e1);
//         });

//     assert_eq!(count, 0);
//     s.run();
//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn system_builder_add_2_types() {
//     let world = World::new();

//     let e1 = world.entity().add::<Position>().add::<Velocity>();
//     world.entity().add::<Velocity>();

//     world.set(Count(0));

//     let s = world.system::<()>()
//         .with::<Position>()
//         .with::<Velocity>()
//         .each_entity(|e| {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             assert!(e == e1);
//         });

//     assert_eq!(count, 0);
//     s.run();
//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn system_builder_add_1_type_w_1_type() {
//     let world = World::new();

//     let e1 = world.entity().add::<Position>().add::<Velocity>();
//     world.entity().add::<Velocity>();

//     world.set(Count(0));

//     let s = world.system::<&mut Position>()
//         .with::<Velocity>()
//         .each_entity(|e, p| {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             assert!(e == e1);
//         });

//     assert_eq!(count, 0);
//     s.run();
//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn system_builder_add_2_types_w_1_type() {
//     let world = World::new();

//     let e1 = world.entity().add::<Position>().add::<Velocity>().add::<Mass>();
//     world.entity().add::<Velocity>();

//     world.set(Count(0));

//     let s = world.system::<&mut Position>()
//         .with::<Velocity>()
//         .with::<Mass>()
//         .each_entity(|e, p| {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             assert!(e == e1);
//         });

//     assert_eq!(count, 0);
//     s.run();
//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn system_builder_add_pair() {
//     let world = World::new();

//     let Likes = world.entity();
//     let Bob = world.entity();
//     let Alice = world.entity();

//     let e1 = world.entity().add(Likes, Bob);
//     world.entity().add(Likes, Alice);

//     world.set(Count(0));

//     let s = world.system::<()>()
//         .with(Likes, Bob)
//         .each_entity(|e| {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             assert!(e == e1);
//         });

//     assert_eq!(count, 0);
//     s.run();
//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn system_builder_add_not() {
//     let world = World::new();

//     let e1 = world.entity().add::<Position>();
//     world.entity().add::<Position>().add::<Velocity>();

//     world.set(Count(0));

//     let s = world.system::<&mut Position>()
//         .with::<Velocity>().oper(flecs::Not)
//         .each_entity(|e, p| {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             assert!(e == e1);
//         });

//     assert_eq!(count, 0);
//     s.run();
//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn system_builder_add_or() {
//     let world = World::new();

//     let e1 = world.entity().add::<Position>();
//     let e2 = world.entity().add::<Velocity>();
//     world.entity().add::<Mass>();

//     world.set(Count(0));

//     let s = world.system::<()>()
//         .with::<Position>().oper(flecs::Or)
//         .with::<Velocity>()
//         .each_entity(|e| {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             assert!(e == e1 || e == e2);
//         });

//     assert_eq!(count, 0);
//     s.run();
//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });
// }

// #[test] fn system_builder_add_optional() {
//     let world = World::new();

//     let e1 = world.entity().add::<Position>();
//     let e2 = world.entity().add::<Position>().add::<Velocity>();
//     world.entity().add::<Velocity>().add::<Mass>();

//     world.set(Count(0));

//     let s = world.system::<()>()
//         .with::<Position>()
//         .with::<Velocity>().oper(flecs::Optional)
//         .each_entity(|e| {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             assert!(e == e1 || e == e2);
//         });

//     assert_eq!(count, 0);
//     s.run();
//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });
// }

// #[test] fn system_builder_ptr_type() {
//     let world = World::new();

//     let e1 = world.entity().add::<Position>();
//     let e2 = world.entity().add::<Position>().add::<Velocity>();
//     world.entity().add::<Velocity>().add::<Mass>();

//     world.set(Count(0));

//     let s = world.system::<&mut Position, Option<&mut Velocity>>()
//         .each_entity(|e, (p,v)| {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             assert!(e == e1 || e == e2);
//         });

//     assert_eq!(count, 0);
//     s.run();
//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });
// }

// #[test] fn system_builder_const_type() {
//     let world = World::new();

//     let e1 = world.entity().add::<Position>();
//     world.entity().add::<Velocity>();

//     world.set(Count(0));

//     let s = world.system::<&Position>()
//         .each_entity(|e, p| {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             assert!(e == e1);
//         });

//     assert_eq!(count, 0);
//     s.run();
//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn system_builder_string_term() {
//     let world = World::new();

//     let e1 = world.entity().add::<Position>();
//     world.entity().add::<Velocity>();

//     world.set(Count(0));

//     let s = world.system::<()>()
//         .expr("flecs.common_test.Position")
//         .each_entity(|e| {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             assert!(e == e1);
//         });

//     s.run();

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn system_builder_singleton_term() {
//     let world = World::new();

//     struct Entity {
//         flecs::entity_view value;
//     };

//     struct Singleton {
//         int32_t value;
//     };

//     world.set<Singleton>({10});

//     world.set(Count(0));

//     let s = world.system::<Entity>()
//         .with::<Singleton>().singleton().in()
//         .run(|mut it| {
//             while it.next() {
//                 let e = it.field::<Entity>(0).unwrap();
//                 let s = it.field::<const Singleton>(1).unwrap();
//                 assert!(!it.is_self(1));
//                 assert_eq!(s.value, 10);

//                 const Singleton& s_ref = *s;
//                 assert_eq!(s_ref.value, 10);

//                 for i in it.iter() {
//                     assert!(it.entity(i) == e[i].value);
//                     world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//                 }
//             }
//         });

//     let
//     e = world.entity(); e.set<Entity>({e});
//     e = world.entity(); e.set<Entity>({e});
//     e = world.entity(); e.set<Entity>({e});

//     s.run();

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,3);
//                 });
// }

// #[test] fn system_builder_10_terms() {
//     let world = World::new();

//     int count = 0;

//     let e = world.entity()
//         .add::<TagA>()
//         .add::<TagB>()
//         .add::<TagC>()
//         .add::<TagD>()
//         .add::<TagE>()
//         .add::<TagF>()
//         .add::<TagG>()
//         .add::<TagH>()
//         .add::<TagI>()
//         .add::<TagJ>();

//     let s = world.system::<()>()
//         .with::<TagA>()
//         .with::<TagB>()
//         .with::<TagC>()
//         .with::<TagD>()
//         .with::<TagE>()
//         .with::<TagF>()
//         .with::<TagG>()
//         .with::<TagH>()
//         .with::<TagI>()
//         .with::<TagJ>()
//         .run(|mut it| {
//             while it.next() {
//                 assert_eq!(it.count(), 1);
//                 assert!(it.entity(0) == e);
//                 assert_eq!(it.field_count(), 10);
//                 world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             }
//         });

//     s.run();

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn system_builder_16_terms() {
//     let world = World::new();

//     int count = 0;

//     let e = world.entity()
//         .add::<TagA>()
//         .add::<TagB>()
//         .add::<TagC>()
//         .add::<TagD>()
//         .add::<TagE>()
//         .add::<TagF>()
//         .add::<TagG>()
//         .add::<TagH>()
//         .add::<TagI>()
//         .add::<TagJ>()
//         .add::<TagK>()
//         .add::<TagL>()
//         .add::<TagM>()
//         .add::<TagN>()
//         .add::<TagO>()
//         .add::<TagP>();

//     let s = world.system::<()>()
//         .with::<TagA>()
//         .with::<TagB>()
//         .with::<TagC>()
//         .with::<TagD>()
//         .with::<TagE>()
//         .with::<TagF>()
//         .with::<TagG>()
//         .with::<TagH>()
//         .with::<TagI>()
//         .with::<TagJ>()
//         .with::<TagK>()
//         .with::<TagL>()
//         .with::<TagM>()
//         .with::<TagN>()
//         .with::<TagO>()
//         .with::<TagP>()
//         .run(|mut it| {
//             while it.next() {
//                 assert_eq!(it.count(), 1);
//                 assert!(it.entity(0) == e);
//                 assert_eq!(it.field_count(), 16);
//                 world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             }
//         });

//     s.run();

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn system_builder_name_arg() {
//     let world = World::new();

//     let s = world.system::<&Position>("MySystem")
//         .term_at(0).src().name("MySystem")
//         .run(|mut it| {
//             while it.next() {}
//         });

//     assert!(s.has::<Position>());
// }

// #[test] fn system_builder_create_w_no_template_args() {
//     let world = World::new();

//     let e1 = world.entity().add::<Position>();

//     world.set(Count(0));

//     let s = world.system::<()>()
//         .with::<Position>()
//         .each_entity(|e| {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             assert!(e == e1);
//         });

//     assert_eq!(count, 0);
//     s.run();
//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn system_builder_write_annotation() {
//     let world = World::new();

//     struct TagA { };
//     struct TagB { };

//     let e1 = world.entity().add::<TagA>();

//     int32_t a_count = 0, b_count = 0;

//     world.system::<TagA>()
//         .with::<TagB>().write()
//         .each_entity(|e, TagA) {
//             a_world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             assert!(e == e1);
//             e.add::<TagB>();
//         });

//     world.system::<TagB>()
//         .each_entity(|e, TagB) {
//             b_world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             assert!(e == e1);
//             assert!(e.has::<TagB>());
//         });

//     assert_eq!(a_count, 0);
//     assert_eq!(b_count, 0);

//     world.progress();

//     assert_eq!(a_count, 1);
//     assert_eq!(b_count, 1);

//     assert!(e1.has::<TagB>());
// }

// #[test] fn system_builder_name_from_root() {
//     let world = World::new();

//     let sys = world.system("::ns::MySystem")
//         .each([](let e) { });

//     assert_eq!(sys.name(), "MySystem");

//     flecs::entity ns = world.entity("::ns");
//     assert!(ns == sys.parent());
// }

// ///////////////////////////////////////////////////////
// ///////////////////////////////////////////////////////
// ///////////////////////////////////////////////////////
// ///////////////////////////////////////////////////////
// ///////////////////////////////////////////////////////
// ///////////////////////////////////////////////////////
// ///////////////////////////////////////////////////////
// ///////////////////////////////////////////////////////
// ///////////////////////////////////////////////////////
// ///////////////////////////////////////////////////////
// ///////////////////////////////////////////////////////
// ///////////////////////////////////////////////////////

// #include <cpp.h>

// struct Pair {
//     float value;
// };

// #[test] fn query_term_each_component() {
//     let world = World::new();

//     let e_1 = world.entity().set(Position{x: 1, y: 2});
//     let e_2 = world.entity().set(Position{x: 3, y: 4});
//     let e_3 = world.entity().set(Position{x: 5, y: 6});

//     e_3.add::<Tag>();

//     world.set(Count(0));
//     world.each_entity::<&mut Position>(|e, p| {
//         if (e == e_1) {
//             assert_eq!(p.x, 1);
//             assert_eq!(p.y, 2);
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         }
//         if (e == e_2) {
//             assert_eq!(p.x, 3);
//             assert_eq!(p.y, 4);
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         }
//         if (e == e_3) {
//             assert_eq!(p.x, 5);
//             assert_eq!(p.y, 6);
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,3);
//                 });
// }

// #[test] fn query_term_each_tag() {
//     let world = World::new();

//     struct Foo { };

//     let e_1 = world.entity().add::<Foo>();
//     let e_2 = world.entity().add::<Foo>();
//     let e_3 = world.entity().add::<Foo>();

//     e_3.add::<Tag>();

//     world.set(Count(0));
//     world.each_entity::<&mut Foo>(|e, Foo) {
//         if (e == e_1 || e == e_2 || e == e_3) {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,3);
//                 });
// }

// #[test] fn query_term_each_id() {
//     let world = World::new();

//     let foo = world.entity();

//     let e_1 = world.entity().add(foo);
//     let e_2 = world.entity().add(foo);
//     let e_3 = world.entity().add(foo);

//     e_3.add::<Tag>();

//     world.set(Count(0));
//     world.each(foo, [&](let e) {
//         if (e == e_1 || e == e_2 || e == e_3) {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,3);
//                 });
// }

// #[test] fn query_term_each_pair_type() {
//     let world = World::new();

//     struct Rel { };
//     struct Obj { };

//     let e_1 = world.entity().add::<Rel, Obj>();
//     let e_2 = world.entity().add::<Rel, Obj>();
//     let e_3 = world.entity().add::<Rel, Obj>();

//     e_3.add::<Tag>();

//     world.set(Count(0));
//     world.each_entity::<&mut flecs::pair<Rel, Obj>>(|e, flecs::pair<Rel,Obj>) {
//         if (e == e_1 || e == e_2 || e == e_3) {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,3);
//                 });
// }

// #[test] fn query_term_each_pair_id() {
//     let world = World::new();

//     let rel = world.entity();
//     let obj = world.entity();

//     let e_1 = world.entity().add(rel, obj);
//     let e_2 = world.entity().add(rel, obj);
//     let e_3 = world.entity().add(rel, obj);

//     e_3.add::<Tag>();

//     world.set(Count(0));
//     world.each(world.pair(rel, obj), [&](let e) {
//         if (e == e_1 || e == e_2 || e == e_3) {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,3);
//                 });
// }

// #[test] fn query_term_each_pair_relation_wildcard() {
//     let world = World::new();

//     let rel_1 = world.entity();
//     let rel_2 = world.entity();
//     let obj = world.entity();

//     let e_1 = world.entity().add(rel_1, obj);
//     let e_2 = world.entity().add(rel_1, obj);
//     let e_3 = world.entity().add(rel_2, obj);

//     e_3.add::<Tag>();

//     world.set(Count(0));
//     world.each(world.pair(flecs::Wildcard, obj), [&](let e) {
//         if (e == e_1 || e == e_2 || e == e_3) {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,3);
//                 });
// }

// #[test] fn query_term_each_pair_object_wildcard() {
//     let world = World::new();

//     let rel = world.entity();
//     let obj_1 = world.entity();
//     let obj_2 = world.entity();

//     let e_1 = world.entity().add(rel, obj_1);
//     let e_2 = world.entity().add(rel, obj_1);
//     let e_3 = world.entity().add(rel, obj_2);

//     e_3.add::<Tag>();

//     world.set(Count(0));
//     world.each(world.pair(rel, flecs::Wildcard), [&](let e) {
//         if (e == e_1 || e == e_2 || e == e_3) {
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,3);
//                 });
// }

// #[test] fn query_default_ctor_no_assign() {
//     flecs::query::<> f;

//     // Make sure code compiles & works
//     assert!(true);
// }

// #[test] fn query_term_get_id() {
//     let world = World::new();

//     let Foo = world.entity();
//     let Bar = world.entity();

//     let q = world.query_builder()
//         .with::<Position>()
//         .with::<Velocity>()
//         .with(Foo, Bar)
//         .build();

//     assert_eq!(q.field_count(), 3);

//     flecs::term
//     t = q.term(0);
//     assert!(t.id() == world.id<Position>());
//     t = q.term(1);
//     assert!(t.id() == world.id<Velocity>());
//     t = q.term(2);
//     assert!(t.id() == world.pair(Foo, Bar));
// }

// #[test] fn query_term_get_subj() {
//     let world = World::new();

//     let Foo = world.entity();
//     let Bar = world.entity();
//     let Src = world.entity();

//     let q = world.query_builder()
//         .with::<Position>()
//         .with::<Velocity>().src(Src)
//         .with(Foo, Bar)
//         .build();

//     assert_eq!(q.field_count(), 3);

//     flecs::term
//     t = q.term(0);
//     assert!(t.get_src() == flecs::This);
//     t = q.term(1);
//     assert!(t.get_src() == Src);
//     t = q.term(2);
//     assert!(t.get_src() == flecs::This);
// }

// #[test] fn query_term_get_pred() {
//     let world = World::new();

//     let Foo = world.entity();
//     let Bar = world.entity();
//     let Src = world.entity();

//     let q = world.query_builder()
//         .with::<Position>()
//         .with::<Velocity>().src(Src)
//         .with(Foo, Bar)
//         .build();

//     assert_eq!(q.field_count(), 3);

//     flecs::term
//     t = q.term(0);
//     assert!(t.get_first() == world.id<Position>());
//     t = q.term(1);
//     assert!(t.get_first() == world.id<Velocity>());
//     t = q.term(2);
//     assert!(t.get_first() == Foo);
// }

// #[test] fn query_term_get_obj() {
//     let world = World::new();

//     let Foo = world.entity();
//     let Bar = world.entity();
//     let Src = world.entity();

//     let q = world.query_builder()
//         .with::<Position>()
//         .with::<Velocity>().src(Src)
//         .with(Foo, Bar)
//         .build();

//     assert_eq!(q.field_count(), 3);

//     flecs::term
//     t = q.term(0);
//     assert!(t.get_second() == 0);
//     t = q.term(1);
//     assert!(t.get_second() == 0);
//     t = q.term(2);
//     assert!(t.get_second() == Bar);
// }

// #[test] fn query_get_first() {
//     let world = World::new();

//     struct A {};

//     let e1 = world.entity().add::<A>();
//     world.entity().add::<A>();
//     world.entity().add::<A>();

//     let q = world.query::<A>();

//     let first = q.iter().first();
//     assert!(first != 0);
//     assert!(first == e1);
// }

// #[test] fn query_get_count_direct() {
//     let world = World::new();

//     struct A {};

//     world.entity().add::<A>();
//     world.entity().add::<A>();
//     world.entity().add::<A>();

//     let q = world.query::<A>();

//     test_int(3, q.count());
// }

// #[test] fn query_get_is_true_direct() {
//     let world = World::new();

//     struct A {};
//     struct B {};

//     world.entity().add::<A>();
//     world.entity().add::<A>();
//     world.entity().add::<A>();

//     let q_1 = world.query::<A>();
//     let q_2 = world.query::<B>();

//     test_bool(true, q_1.is_true());
//     test_bool(false, q_2.is_true());
// }

// #[test] fn query_get_first_direct() {
//     let world = World::new();

//     struct A {};

//     let e1 = world.entity().add::<A>();
//     world.entity().add::<A>();
//     world.entity().add::<A>();

//     let q = world.query::<A>();

//     let first = q.first();
//     assert!(first != 0);
//     assert!(first == e1);
// }

// #[test] fn query_each_w_no_this() {
//     let world = World::new();

//     let e = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2});

//     let f = world.query_builder<&mut Position, &mut Velocity>()
//         .term_at(0).src(e)
//         .term_at(1).src(e)
//         .build();

//     world.set(Count(0));

//     f.each([&](Position& p, Velocity& v) {
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         assert_eq!(p.x, 10);
//         assert_eq!(p.y, 20);
//         assert_eq!(v.x, 1);
//         assert_eq!(v.y, 2);
//     });

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn query_each_w_iter_no_this() {
//     let world = World::new();

//     let e = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2});

//     let f = world.query_builder<&mut Position, &mut Velocity>()
//         .term_at(0).src(e)
//         .term_at(1).src(e)
//         .build();

//     world.set(Count(0));

//     f.each([&](flecs::iter& it, size_t index, Position& p, Velocity& v) {
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         assert_eq!(p.x, 10);
//         assert_eq!(p.y, 20);
//         assert_eq!(v.x, 1);
//         assert_eq!(v.y, 2);
//         assert_eq!(index, 0);
//         assert_eq!(it.count(), 0);
//     });

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn query_invalid_each_w_no_this() {
//     install_test_abort();

//     let world = World::new();

//     let e = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2});

//     let f = world.query_builder<&mut Position, &mut Velocity>()
//         .term_at(0).src(e)
//         .term_at(1).src(e)
//         .build();

//     test_expect_abort();

//     f.each_entity(|e, (p,v)| { });
// }

// #[test] fn query_named_query() {
//     let world = World::new();

//     let e1 = world.entity().add::<Position>();
//     let e2 = world.entity().add::<Position>();

//     let q = world.query::<Position>("my_query");

//     world.set(Count(0));
//     q.each_entity(|e, Position&) {
//         assert!(e == e1 || e == e2);
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });
//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });

//     flecs::entity qe = q.entity();
//     assert!(qe != 0);
//     assert_eq!(qe.name(), "my_query");
// }

// #[test] fn query_named_scoped_query() {
//     let world = World::new();

//     let e1 = world.entity().add::<Position>();
//     let e2 = world.entity().add::<Position>();

//     let q = world.query::<Position>("my::query");

//     world.set(Count(0));
//     q.each_entity(|e, Position&) {
//         assert!(e == e1 || e == e2);
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });
//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });

//     flecs::entity qe = q.entity();
//     assert!(qe != 0);
//     assert_eq!(qe.name(), "query");
//     assert_eq!(qe.path(), "::my::query");
// }

// #[test] fn query_set_this_var() {
//     let world = World::new();

//     /* let e_1 = */ world.entity().set(Position{x: 1, y: 2});
//     let e_2 = world.entity().set(Position{x: 3, y: 4});
//     /* let e_3 = */ world.entity().set(Position{x: 5, y: 6});

//     let q = world.query::<Position>("my::query");

//     world.set(Count(0));
//     q.iter().set_var(0, e_2).each_entity(|e, Position&) {
//         assert!(e == e_2);
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });
//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn query_inspect_terms_w_expr() {
//     let world = World::new();

//     flecs::query::<> f = world.query_builder()
//         .expr("(ChildOf,0)")
//         .build();

//     world.set(Count(0));
//     f.each_term([&](flecs::term &term) {
//         assert!(term.id().is_pair());
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn query_find() {
//     let world = World::new();

//     /* let e1 = */ world.entity().set(Position{x: 10, y: 20});
//     let e2 = world.entity().set(Position{x: 20, y: 30});

//     let q = world.query::<Position>();

//     let r = q.find([](Position& p) {
//         return p.x == 20;
//     });

//     assert!(r == e2);
// }

// #[test] fn query_find_not_found() {
//     let world = World::new();

//     /* let e1 = */ world.entity().set(Position{x: 10, y: 20});
//     /* let e2 = */ world.entity().set(Position{x: 20, y: 30});

//     let q = world.query::<Position>();

//     let r = q.find([](Position& p) {
//         return p.x == 30;
//     });

//     assert!(!r);
// }

// #[test] fn query_find_w_entity() {
//     let world = World::new();

//     /* let e1 = */ world.entity().set(Position{x: 10, y: 20}).set(Velocity{x: 20, y: 30});
//     let e2 = world.entity().set(Position{x: 20, y: 30}).set(Velocity{x: 20, y: 30});

//     let q = world.query::<Position>();

//     let r = q.find([](let e, p| {
//         return p.x == e.get<Velocity>().x &&
//                p.y == e.get<Velocity>().y;
//     });

//     assert!(r == e2);
// }

// #[test] fn query_optional_pair_term() {
//     let world = World::new();

//     world.entity()
//         .add::<TagA>()
//         .emplace<Position, Tag>(1.0f, 2.0f);
//     world.entity()
//         .add::<TagA>();

//     int32_t with_pair = 0, without_pair = 0;

//     let f = world.query_builder<flecs::pair<Position, Tag>*>()
//         .with::<TagA>()
//         .build();

//     f.each_entity(|e, Position* p) {
//         if (p) {
//             with_pair++;
//             test_flt(1.0f, p.x);
//             test_flt(2.0f, p.y);
//         } else {
//             without_pair++;
//         }
//     });

//     world.progress(1.0);

//     assert_eq!(1, with_pair);
//     assert_eq!(1, without_pair);
// }

// #[test] fn query_action() {
//     let world = World::new();

//     world.component::<Position>();
//     world.component::<Velocity>();

//     let entity = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2});

//     let q = world.query::<&mut Position, &mut Velocity>();

//     q.run(|mut it| {
//         while it.next() {
//             let mut p = it.field::<Position>(0).unwrap();
//             let v = it.field::<Velocity>(1).unwrap();

//             for i in it.iter() {
//                 p[i].x += v[i].x;
//                 p[i].y += v[i].y;
//             }
//         }
//     });

//     entity.get::<&Position>(|p| {
//     assert_eq!(p.x, 11);
//     assert_eq!(p.y, 22);
// });
// }

// #[test] fn query_action_const() {
//     let world = World::new();

//     world.component::<Position>();
//     world.component::<Velocity>();

//     let entity = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2});

//     let q = world.query::<Position, &Velocity>();

//     q.run(|mut it| {
//         while it.next() {
//             let mut p = it.field::<Position>(0).unwrap();
//             let v = it.field::<&Velocity>(1).unwrap();

//             for i in it.iter() {
//                 p[i].x += v[i].x;
//                 p[i].y += v[i].y;
//             }
//         }
//     });

//     entity.get::<&Position>(|p| {
//     assert_eq!(p.x, 11);
//     assert_eq!(p.y, 22);
// });
// }

// #[test] fn query_action_shared() {
//     let world = World::new();

//     world.component::<Position>().add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();
//     world.component::<Velocity>().add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

//     let base = world.entity()
//         .set(Velocity{x: 1, y: 2});

//     let e1 = world.entity()
//         .set(Position{x: 10, y: 20})
//         .add_id((flecs::IsA::ID, base));

//     let e2 = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 3, y: 4});

//     let q = world.query_builder<Position>()
//         .expr("flecs.common_test.Velocity(self|up IsA)")
//         .build();

//     q.run([](flecs::iter&it) {
//         while it.next() {
//             let mut p = it.field::<Position>(0).unwrap();
//             let v = it.field::<&Velocity>(1).unwrap();

//             if (!it.is_self(1)) {
//                 for i in it.iter() {
//                     p[i].x += v.x;
//                     p[i].y += v.y;
//                 }
//             } else {
//                 for i in it.iter() {
//                     p[i].x += v[i].x;
//                     p[i].y += v[i].y;
//                 }
//             }
//         }
//     });

//     e1.get::<&Position>(|p| {
//     assert_eq!(p.x, 11);
//     assert_eq!(p.y, 22);
// });

//     e2.get::<&Position>(|p| {
//     assert_eq!(p.x, 13);
//     assert_eq!(p.y, 24);
// });
// }

// #[test] fn query_action_optional() {
//     let world = World::new();

//     world.component::<Position>();
//     world.component::<Velocity>();
//     world.component_named::<Mass>("Mass");

//     let e1 = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2})
//         .set(Mass{value:1});

//     let e2 = world.entity()
//         .set(Position{x: 30, y: 40})
//         .set(Velocity{x: 3, y: 4})
//         .set(Mass{value:1});

//     let e3 = world.entity()
//         .set(Position{x: 50, y: 60});

//     let e4 = world.entity()
//         .set(Position{x: 70, y: 80});

//     let q = world.query::<Position, Velocity*, Mass*>();

//     q.run(|mut it| {
//         while it.next() {
//             let mut p = it.field::<Position>(0).unwrap();
//             let v = it.field::<Velocity>(1).unwrap();
//             let m = it.field::<Mass>(2).unwrap();

//             if (it.is_set(1) && it.is_set(2)) {
//                 for i in it.iter() {
//                     p[i].x += v[i].x * m[i].value;
//                     p[i].y += v[i].y * m[i].value;
//                 }
//             } else {
//                 for i in it.iter() {
//                     p[i].x += 1;
//                     p[i].y += 1;
//                 }
//             }
//         }
//     });

//     e1.get::<&Position>(|p| {
//     assert_eq!(p.x, 11);
//     assert_eq!(p.y, 22);
// });

//     e2.get::<&Position>(|p| {
//     assert_eq!(p.x, 33);
//     assert_eq!(p.y, 44);
// });

//     e3.get::<&Position>(|p| {
//     assert_eq!(p.x, 51);
//     assert_eq!(p.y, 61);
// });

//     e4.get::<&Position>(|p| {
//     assert_eq!(p.x, 71);
//     assert_eq!(p.y, 81);
// });
// }

// #[test] fn query_each() {
//     let world = World::new();

//     world.component::<Position>();
//     world.component::<Velocity>();

//     let entity = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2});

//     let q = world.query::<&mut Position, &mut Velocity>();

//     q.each_entity(|e, (p,v)| {
//         p.x += v.x;
//         p.y += v.y;
//     });

//     entity.get::<&Position>(|p| {
//     assert_eq!(p.x, 11);
//     assert_eq!(p.y, 22);
// });
// }

// #[test] fn query_each_const() {
//     let world = World::new();

//     world.component::<Position>();
//     world.component::<Velocity>();

//     let entity = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2});

//     let q = world.query::<Position, &Velocity>();

//     q.each_entity(|e, (p,v)| {
//         p.x += v.x;
//         p.y += v.y;
//     });

//     entity.get::<&Position>(|p| {
//     assert_eq!(p.x, 11);
//     assert_eq!(p.y, 22);
// });
// }

// #[test] fn query_each_shared() {
//     let world = World::new();

//     world.component::<Position>();
//     world.component::<Velocity>();

//     let base = world.entity()
//         .set(Velocity{x: 1, y: 2});

//     let e1 = world.entity()
//         .set(Position{x: 10, y: 20})
//         .add_id((flecs::IsA::ID, base));

//     let e2 = world.entity()
//         .set(Position{x: 20, y: 30})
//         .add_id((flecs::IsA::ID, base));

//     let e3 = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 3, y: 4});

//     let q = world.query::<Position, &Velocity>();

//     q.each_entity(|e, (p,v)| {
//         p.x += v.x;
//         p.y += v.y;
//     });

//     e1.get::<&Position>(|p| {
//     assert_eq!(p.x, 11);
//     assert_eq!(p.y, 22);
// });

//     e2.get::<&Position>(|p| {
//     assert_eq!(p.x, 21);
//     assert_eq!(p.y, 32);
// });

//     e3.get::<&Position>(|p| {
//     assert_eq!(p.x, 13);
//     assert_eq!(p.y, 24);
// });
// }

// #[test] fn query_each_optional() {
//     let world = World::new();

//     world.component::<Position>();
//     world.component::<Velocity>();
//     world.component_named::<Mass>("Mass");

//     let e1 = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2})
//         .set(Mass{value:1});

//     let e2 = world.entity()
//         .set(Position{x: 30, y: 40})
//         .set(Velocity{x: 3, y: 4})
//         .set(Mass{value:1});

//     let e3 = world.entity()
//         .set(Position{x: 50, y: 60});

//     let e4 = world.entity()
//         .set(Position{x: 70, y: 80});

//     let q = world.query::<Position, Velocity*, Mass*>();

//     q.each_entity(|e, (p,v,m)| {
//         if (v && m) {
//             p.x += v.x * m.value;
//             p.y += v.y * m.value;
//         } else {
//             p.x += 1;
//             p.y += 1;
//         }
//     });

//     e1.get::<&Position>(|p| {
//     assert_eq!(p.x, 11);
//     assert_eq!(p.y, 22);
// });

//     e2.get::<&Position>(|p| {
//     assert_eq!(p.x, 33);
//     assert_eq!(p.y, 44);
// });

//     e3.get::<&Position>(|p| {
//     assert_eq!(p.x, 51);
//     assert_eq!(p.y, 61);
// });

//     e4.get::<&Position>(|p| {
//     assert_eq!(p.x, 71);
//     assert_eq!(p.y, 81);
// });
// }

// #[test] fn query_signature() {
//     let world = World::new();

//     world.component::<Position>();
//     world.component::<Velocity>();

//     let entity = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2});

//     let q = world.query_builder<>().expr("flecs.common_test.Position, flecs.common_test.Velocity").build();

//     q.run(|mut it| {
//         while it.next() {
//             let mut p = it.field::<Position>(0).unwrap();
//             let v = it.field::<Velocity>(1).unwrap();

//             for i in it.iter() {
//                 p[i].x += v[i].x;
//                 p[i].y += v[i].y;
//             }
//         }
//     });

//     entity.get::<&Position>(|p| {
//     assert_eq!(p.x, 11);
//     assert_eq!(p.y, 22);
// });
// }

// #[test] fn query_signature_const() {
//     let world = World::new();

//     world.component::<Position>();
//     world.component::<Velocity>();

//     let entity = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2});

//     let q = world.query_builder<>().expr("flecs.common_test.Position, [in] flecs.common_test.Velocity").build();

//     q.run(|mut it| {
//         while it.next() {
//             let mut p = it.field::<Position>(0).unwrap();
//             let v = it.field::<&Velocity>(1).unwrap();

//             for i in it.iter() {
//                 p[i].x += v[i].x;
//                 p[i].y += v[i].y;
//             }
//         }
//     });

//     entity.get::<&Position>(|p| {
//     assert_eq!(p.x, 11);
//     assert_eq!(p.y, 22);
// });
// }

// #[test] fn query_signature_shared() {
//     let world = World::new();

//     world.component::<Position>().add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();
//     world.component::<Velocity>().add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

//     let base = world.entity()
//         .set(Velocity{x: 1, y: 2});

//     let e1 = world.entity()
//         .set(Position{x: 10, y: 20})
//         .add_id((flecs::IsA::ID, base));

//     let e2 = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 3, y: 4});

//     let q = world.query_builder<>()
//         .expr("flecs.common_test.Position, [in] flecs.common_test.Velocity(self|up IsA)")
//         .build();

//     q.run([](flecs::iter&it) {
//         while it.next() {
//             let mut p = it.field::<Position>(0).unwrap();
//             let v = it.field::<&Velocity>(1).unwrap();

//             if (!it.is_self(1)) {
//                 for i in it.iter() {
//                     p[i].x += v.x;
//                     p[i].y += v.y;
//                 }
//             } else {
//                 for i in it.iter() {
//                     p[i].x += v[i].x;
//                     p[i].y += v[i].y;
//                 }
//             }
//         }
//     });

//     e1.get::<&Position>(|p| {
//     assert_eq!(p.x, 11);
//     assert_eq!(p.y, 22);
// });

//     e2.get::<&Position>(|p| {
//     assert_eq!(p.x, 13);
//     assert_eq!(p.y, 24);
// });
// }

// #[test] fn query_signature_optional() {
//     let world = World::new();

//     world.component::<Position>();
//     world.component::<Velocity>();
//     world.component_named::<Mass>("Mass");

//     let e1 = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2})
//         .set(Mass{value:1});

//     let e2 = world.entity()
//         .set(Position{x: 30, y: 40})
//         .set(Velocity{x: 3, y: 4})
//         .set(Mass{value:1});

//     let e3 = world.entity()
//         .set(Position{x: 50, y: 60});

//     let e4 = world.entity()
//         .set(Position{x: 70, y: 80});

//     let q = world.query_builder<>().expr("flecs.common_test.Position, ?flecs.common_test.Velocity, ?Mass").build();

//     q.run(|mut it| {
//         while it.next() {
//             let mut p = it.field::<Position>(0).unwrap();
//             let v = it.field::<&Velocity>(1).unwrap();
//             let m = it.field::<const Mass>(2).unwrap();

//             if (it.is_set(1) && it.is_set(2)) {
//                 for i in it.iter() {
//                     p[i].x += v[i].x * m[i].value;
//                     p[i].y += v[i].y * m[i].value;
//                 }
//             } else {
//                 for i in it.iter() {
//                     p[i].x += 1;
//                     p[i].y += 1;
//                 }
//             }
//         }
//     });

//     e1.get::<&Position>(|p| {
//     assert_eq!(p.x, 11);
//     assert_eq!(p.y, 22);
// });

//     e2.get::<&Position>(|p| {
//     assert_eq!(p.x, 33);
//     assert_eq!(p.y, 44);
// });

//     e3.get::<&Position>(|p| {
//     assert_eq!(p.x, 51);
//     assert_eq!(p.y, 61);
// });

//     e4.get::<&Position>(|p| {
//     assert_eq!(p.x, 71);
//     assert_eq!(p.y, 81);
// });
// }

// #[test] fn query_query_single_pair() {
//     let world = World::new();

//     world.entity().add::<Pair, Position>();
//     let e2 = world.entity().add::<Pair, Velocity>();

//     let q = world.query_builder<>()
//         .expr("(flecs.common_test.Pair, flecs.common_test.Velocity)")
//         .build();

//     int32_t table_count = 0;
//     int32_t entity_count = 0;

//     q.run(|mut it| {
//         while it.next() {
//             table_world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             for i in it.iter() {
//                 assert!(it.entity(i) == e2);
//                 entity_world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             }
//         }
//     });

//     assert_eq!(table_count, 1);
//     assert_eq!(entity_count, 1);
// }

// #[test] fn query_tag_w_each() {
//     let world = World::new();

//     let q = world.query::<Tag>();

//     let e = world.entity()
//         .add::<Tag>();

//     q.each([&](flecs::entity qe, Tag) {
//         assert!(qe == e);
//     });
// }

// #[test] fn query_shared_tag_w_each() {
//     let world = World::new();

//     let q = world.query::<Tag>();

//     let base = world.prefab()
//         .add::<Tag>();

//     let e = world.entity()
//         .add_id((flecs::IsA::ID, base));

//     q.each([&](flecs::entity qe, Tag) {
//         assert!(qe == e);
//     });
// }

// static
// int compare_position(
//     flecs::entity_t e1,
//     &Position *p1,
//     flecs::entity_t e2,
//     &Position *p2)
// {
//     return (p1.x > p2.x) - (p1.x < p2.x);
// }

// #[test] fn query_sort_by() {
//     let world = World::new();

//     world.entity().set(Position{x: 1, y: 0});
//     world.entity().set(Position{x: 6, y: 0});
//     world.entity().set(Position{x: 2, y: 0});
//     world.entity().set(Position{x: 5, y: 0});
//     world.entity().set(Position{x: 4, y: 0});

//     let q = world.query_builder<Position>()
//         .order_by(compare_position)
//         .build();

//     q.run([](flecs::iter it) {
//         while it.next() {
//             let mut p = it.field::<Position>(0).unwrap();
//             assert_eq!(it.count(), 5);
//             assert_eq!(p[0].x, 1);
//             assert_eq!(p[1].x, 2);
//             assert_eq!(p[2].x, 4);
//             assert_eq!(p[3].x, 5);
//             assert_eq!(p[4].x, 6);
//         }
//     });
// }

// #[test] fn query_changed() {
//     let world = World::new();

//     let e = world.entity().set(Position{x: 1, y: 0});

//     let q = world.query_builder<&Position>()
//         .cached()
//         .build();

//     let q_w = world.query::<Position>();

//     test_bool(q.changed(), true);

//     q.each([](&Position& p) { });
//     test_bool(q.changed(), false);

//     e.set(Position{x: 2, y: 0});
//     test_bool(q.changed(), true);

//     q.each([](&Position& p) { });
//     test_bool(q.changed(), false); // Reset state

//     q_w.each([](Position& p) { }); // Query has out term
//     test_bool(q.changed(), true);
// }

// #[test] fn query_default_ctor() {
//     let world = World::new();

//     flecs::query::<Position> q_var;

//     int count = 0;
//     let q = world.query::<Position>();

//     world.entity().set(Position{x: 10, y: 20});

//     q_var = q;

//     q_var.each_entity(|e, p| {
//         assert_eq!(p.x, 10);
//         assert_eq!(p.y, 20);
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn query_expr_w_template() {
//     let world = World::new();

//     let comp = world.component::<Template<int>>();
//     assert_eq!(comp.name(), "Template<int>");

//     int count = 0;
//     let q = world.query_builder<Position>().expr("flecs.common_test.Template<int>").build();

//     world.entity()
//         .set(Position{x: 10, y: 20})
//         .set<Template<int>>({30, 40});

//     q.each_entity(|e, p| {
//         assert_eq!(p.x, 10);
//         assert_eq!(p.y, 20);

//         const Template<int> *t = e.get<Template<int>>();
//         assert_eq!(t.x, 30);
//         assert_eq!(t.y, 40);

//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn query_query_type_w_template() {
//     let world = World::new();

//     let comp = world.component::<Template<int>>();
//     assert_eq!(comp.name(), "Template<int>");

//     int count = 0;
//     let q = world.query::<Position, Template<int>>();

//     world.entity()
//         .set(Position{x: 10, y: 20})
//         .set<Template<int>>({30, 40});

//     q.each_entity(|e, Position& p, Template<int>& t) {
//         assert_eq!(p.x, 10);
//         assert_eq!(p.y, 20);

//         assert_eq!(t.x, 30);
//         assert_eq!(t.y, 40);

//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn query_compare_term_id() {
//     let world = World::new();

//     int count = 0;
//     let e = world.entity().add::<Tag>();

//     let q = world.query_builder<>()
//         .with::<Tag>()
//         .build();

//     q.run(|mut it| {
//         while it.next() {
//             assert!(it.id(0) == it.world().id<Tag>());
//             assert!(it.entity(0) == e);
//         }
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn query_test_no_defer_each() {
//     install_test_abort();

//     let world = World::new();

//     struct Value { int value; };

//     world.entity().add::<Tag>().set(Value{value:10});

//     let q = world.query_builder<Value>()
//         .with::<Tag>()
//         .build();

//     q.each_entity(|e, v| {
//         test_expect_abort();
//         e.remove::<Tag>();
//     });

//     assert!(false); // Should never get here
// }

// #[test] fn query_test_no_defer_iter() {
//     install_test_abort();

//     let world = World::new();

//     struct Value { int value; };

//     world.entity().add::<Tag>().set(Value{value:10});

//     let q = world.query_builder<Value>()
//         .with::<Tag>()
//         .build();

//     q.run(|mut it| {
//         while it.next() {
//             for i in it.iter() {
//                 test_expect_abort();
//                 it.entity(i).remove::<Tag>();
//             }
//         }
//     });

//     assert!(false); // Should never get here
// }

// #[test] fn query_inspect_terms() {
//     let world = World::new();

//     let mut p = world.entity();

//     let q = world.query_builder<Position>()
//         .with::<Velocity>()
//         .with(flecs::ChildOf, p)
//         .build();

//     test_int(3, q.field_count());

//     let t = q.term(0);
//     test_int(t.id(), world.id<Position>());
//     assert_eq!(t.oper(), flecs::And);
//     assert_eq!(t.inout(), flecs::InOutDefault);

//     t = q.term(1);
//     test_int(t.id(), world.id<Velocity>());
//     assert_eq!(t.oper(), flecs::And);
//     assert_eq!(t.inout(), flecs::InOutNone);

//     t = q.term(2);
//     test_int(t.id(), world.pair(flecs::ChildOf, p));
//     assert_eq!(t.oper(), flecs::And);
//     assert_eq!(t.inout(), flecs::InOutNone);
//     assert!(t.id().second() == p);
// }

// #[test] fn query_inspect_terms_w_each() {
//     let world = World::new();

//     let mut p = world.entity();

//     let q = world.query_builder<Position>()
//         .with::<Velocity>()
//         .with(flecs::ChildOf, p)
//         .build();

//     int32_t count =  0;
//     q.each_term([&](flecs::term& t) {
//         if (count == 0) {
//             test_int(t.id(), world.id<Position>());
//             assert_eq!(t.inout(), flecs::InOutDefault);
//         } else if (count == 1) {
//             test_int(t.id(), world.id<Velocity>());
//             assert_eq!(t.inout(), flecs::InOutNone);
//         } else if (count == 2) {
//             test_int(t.id(), world.pair(flecs::ChildOf, p));
//             assert!(t.id().second() == p);
//             assert_eq!(t.inout(), flecs::InOutNone);
//         } else {
//             assert!(false);
//         }

//         assert_eq!(t.oper(), flecs::And);

//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,3);
//                 });
// }

// #[test] fn query_comp_to_str() {
//     let world = World::new();

//     let q = world.query_builder<Position>()
//         .with::<Velocity>()
//         .build();
//     assert_eq!(q.str(), "Position($this), [none] Velocity($this)");
// }

// struct Eats { int amount; };
// struct Apples { };
// struct Pears { };

// #[test] fn query_pair_to_str() {
//     let world = World::new();

//     let q = world.query_builder<Position>()
//         .with::<Velocity>()
//         .with::<Eats, Apples>()
//         .build();
//     assert_eq!(q.str(), "Position($this), [none] Velocity($this), [none] Eats($this,Apples)");
// }

// #[test] fn query_oper_not_to_str() {
//     let world = World::new();

//     let q = world.query_builder<Position>()
//         .with::<Velocity>().oper(flecs::Not)
//         .build();
//     assert_eq!(q.str(), "Position($this), !Velocity($this)");
// }

// #[test] fn query_oper_optional_to_str() {
//     let world = World::new();

//     let q = world.query_builder<Position>()
//         .with::<Velocity>().oper(flecs::Optional)
//         .build();
//     assert_eq!(q.str(), "Position($this), [none] ?Velocity($this)");
// }

// #[test] fn query_oper_or_to_str() {
//     let world = World::new();

//     let q = world.query_builder<>()
//         .with::<Position>().oper(flecs::Or)
//         .with::<Velocity>()
//         .build();
//     assert_eq!(q.str(), "[none] Position($this) || Velocity($this)");
// }

// using EatsApples = flecs::pair<Eats, Apples>;
// using EatsPears = flecs::pair<Eats, Pears>;

// #[test] fn query_each_pair_type() {
//     let world = World::new();

//     let e1 = world.entity()
//         .set<EatsApples>({10});

//     world.entity()
//         .set<EatsPears>({20});

//     let q = world.query::<EatsApples>();

//     int count = 0;
//     q.each_entity(|e, EatsApples&& a) {
//         assert_eq!(a.amount, 10);
//         assert!(e == e1);
//         a.amount ++;
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });

//     let v = e1.get<EatsApples>();
//     assert!(v != NULL);
//     assert_eq!(v.amount, 11);
// }

// #[test] fn query_iter_pair_type() {
//     let world = World::new();

//     let e1 = world.entity()
//         .set<EatsApples>({10});

//     world.entity()
//         .set<EatsPears>({20});

//     let q = world.query::<EatsApples>();

//     int count = 0;
//     q.run(|mut it| {
//         while it.next() {
//             let a = it.field::<Eats>(0).unwrap();
//             assert_eq!(it.count(), 1);

//             assert_eq!(a.amount, 10);
//             assert!(it.entity(0) == e1);

//             a.amount ++;
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         }
//     });

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });

//     let v = e1.get<EatsApples>();
//     assert!(v != NULL);
//     assert_eq!(v.amount, 11);
// }

// #[test] fn query_term_pair_type() {
//     let world = World::new();

//     let e1 = world.entity()
//         .set<EatsApples>({10});

//     world.entity()
//         .set<EatsPears>({20});

//     let q = world.query_builder<>()
//         .with::<EatsApples>().inout()
//         .build();

//     int count = 0;
//     q.run(|mut it| {
//         while it.next() {
//             assert_eq!(it.count(), 1);

//             let a = it.field::<EatsApples>(0).unwrap();

//             assert_eq!(a.amount, 10);
//             assert!(it.entity(0) == e1);

//             a.amount ++;
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         }
//     });

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });

//     let v = e1.get<EatsApples>();
//     assert!(v != NULL);
//     assert_eq!(v.amount, 11);
// }

// #[test] fn query_each_no_entity_1_comp() {
//     let world = World::new();

//     let e = world.entity()
//         .set(Position{1, 2});

//     let q = world.query::<Position>();

//     world.set(Count(0));
//     q.each([&](Position& p) {
//         assert_eq!(p.x, 1);
//         assert_eq!(p.y, 2);
//         p.x += 1;
//         p.y += 2;
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });

//     let mut pos = e.get<Position>();
//     assert_eq!(pos.x, 2);
//     assert_eq!(pos.y, 4);
// }

// #[test] fn query_each_no_entity_2_comps() {
//     let world = World::new();

//     let e = world.entity()
//         .set(Position{1, 2})
//         .set(Velocity{10, 20});

//     let q = world.query::<&mut Position, &mut Velocity>();

//     world.set(Count(0));
//     q.each([&](Position& p, Velocity& v) {
//         assert_eq!(p.x, 1);
//         assert_eq!(p.y, 2);
//         assert_eq!(v.x, 10);
//         assert_eq!(v.y, 20);

//         p.x += 1;
//         p.y += 2;
//         v.x += 1;
//         v.y += 2;
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });

//     test_bool(e.get([](const (p,v)| {
//         assert_eq!(p.x, 2);
//         assert_eq!(p.y, 4);

//         assert_eq!(v.x, 11);
//         assert_eq!(v.y, 22);
//     }), true);

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn query_iter_no_comps_1_comp() {
//     let world = World::new();

//     world.entity().add::<Position>();
//     world.entity().add::<Position>();
//     world.entity().add::<Position>().add::<Velocity>();
//     world.entity().add::<Velocity>();

//     let q = world.query::<Position>();

//     world.set(Count(0));
//     q.run(|mut it| {
//         while it.next() {
//             count += it.count();
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,3);
//                 });
// }

// #[test] fn query_iter_no_comps_2_comps() {
//     let world = World::new();

//     world.entity().add::<Velocity>();
//     world.entity().add::<Position>();
//     world.entity().add::<Position>().add::<Velocity>();
//     world.entity().add::<Position>().add::<Velocity>();

//     let q = world.query::<&mut Position, &mut Velocity>();

//     world.set(Count(0));
//     q.run(|mut it| {
//         while it.next() {
//             count += it.count();
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });
// }

// #[test] fn query_iter_no_comps_no_comps() {
//     let world = World::new();

//     world.entity().add::<Velocity>();
//     world.entity().add::<Position>();
//     world.entity().add::<Position>().add::<Velocity>();
//     world.entity().add::<Position>().add::<Velocity>();

//     let q = world.query_builder<>()
//         .with::<Position>()
//         .build();

//     world.set(Count(0));
//     q.run(|mut it| {
//         while it.next() {
//             count += it.count();
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,3);
//                 });
// }

// #include <iostream>

// struct Event {
//     const char *value;
// };

// struct Begin { };
// struct End { };

// using BeginEvent = flecs::pair<Begin, Event>;
// using EndEvent = flecs::pair<End, Event>;

// #[test] fn query_each_pair_object() {
//     let world = World::new();

//     let e1 = world.entity()
//         .set_second<Begin, Event>({"Big Bang"})
//         .set<EndEvent>({"Heat Death"});

//     let q = world.query::<BeginEvent, EndEvent>();

//     world.set(Count(0));
//     q.each_entity(|e, BeginEvent b_e, EndEvent e_e) {
//         assert!(e == e1);
//         assert_eq!(b_e.value, "Big Bang");
//         assert_eq!(e_e.value, "Heat Death");
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn query_iter_pair_object() {
//     let world = World::new();

//     let e1 = world.entity()
//         .set_second<Begin, Event>({"Big Bang"})
//         .set<EndEvent>({"Heat Death"});

//     let q = world.query::<BeginEvent, EndEvent>();

//     world.set(Count(0));
//     q.run(|mut it| {
//         while it.next() {
//             let b_e = it.field::<BeginEvent>(0).unwrap();
//             let e_e = it.field::<EndEvent>(1).unwrap();

//             for i in it.iter() {
//                 assert!(it.entity(i) == e1);
//                 assert_eq!(b_e[i].value, "Big Bang");
//                 assert_eq!(e_e[i].value, "Heat Death");
//                 world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             }
//         }
//     });

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn query_iter_query_in_system() {
//     let world = World::new();

//     world.entity().add::<Position>().add::<Velocity>();

//     let q = world.query::<Velocity>();

//     world.set(Count(0));
//     world.system::<&mut Position>()
//         .each([&](let e1, Position&) {
//             q.each([&](flecs::entity e2, Velocity&) {
//                 world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             });
//         });

//     world.progress();

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn query_iter_type() {
//     let world = World::new();

//     world.entity().add::<Position>();
//     world.entity().add::<Position>().add::<Velocity>();

//     let q = world.query::<Position>();

//     q.run(|mut it| {
//         while it.next() {
//             assert!(it.type().count() >= 1);
//             assert!(it.table().has::<Position>());
//         }
//     });
// }

// #[test] fn query_instanced_query_w_singleton_each() {
//     let world = World::new();

//     world.set(Velocity{x: 1, y: 2});

//     let e1 = world.entity().set(Position{x: 10, y: 20}); e1.set(SelfRef{value: e1.id()});
//     let e2 = world.entity().set(Position{x: 20, y: 30}); e2.set(SelfRef{value: e2.id()});
//     let e3 = world.entity().set(Position{x: 30, y: 40}); e3.set(SelfRef{value: e3.id()});
//     let e4 = world.entity().set(Position{x: 40, y: 50}); e4.set(SelfRef{value: e4.id()});
//     let e5 = world.entity().set(Position{x: 50, y: 60}); e5.set(SelfRef{value: e5.id()});

//     e4.add::<Tag>();
//     e5.add::<Tag>();

//     let q = world.query_builder<SelfRef, Position, &Velocity>()
//         .term_at(2).singleton()
//         .instanced()
//         .build();

//     world.set(Count(0));
//     q.each_entity(|e, (s,p,v)| {
//         assert!(e == s.value);
//         p.x += v.x;
//         p.y += v.y;
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,5);
//                 });

//     assert!(e1.get([](&Position& p) {
//         assert_eq!(p.x, 11);
//         assert_eq!(p.y, 22);
//     }));

//     assert!(e2.get([](&Position& p) {
//         assert_eq!(p.x, 21);
//         assert_eq!(p.y, 32);
//     }));

//     assert!(e3.get([](&Position& p) {
//         assert_eq!(p.x, 31);
//         assert_eq!(p.y, 42);
//     }));

//     assert!(e4.get([](&Position& p) {
//         assert_eq!(p.x, 41);
//         assert_eq!(p.y, 52);
//     }));

//     assert!(e5.get([](&Position& p) {
//         assert_eq!(p.x, 51);
//         assert_eq!(p.y, 62);
//     }));
// }

// #[test] fn query_instanced_query_w_base_each() {
//     let world = World::new();

//     let base = world.entity().set(Velocity{x: 1, y: 2});

//     let e1 = world.entity().is_a_id(base).set(Position{x: 10, y: 20}); e1.set(SelfRef{value: e1.id()});
//     let e2 = world.entity().is_a_id(base).set(Position{x: 20, y: 30}); e2.set(SelfRef{value: e2.id()});
//     let e3 = world.entity().is_a_id(base).set(Position{x: 30, y: 40}); e3.set(SelfRef{value: e3.id()});
//     let e4 = world.entity().is_a_id(base).set(Position{x: 40, y: 50}).add::<Tag>(); e4.set(SelfRef{value: e4.id()});
//     let e5 = world.entity().is_a_id(base).set(Position{x: 50, y: 60}).add::<Tag>(); e5.set(SelfRef{value: e5.id()});
//     let e6 = world.entity().set(Position{x: 60, y: 70}).set(Velocity{x: 2, y: 3}); e6.set(SelfRef{value: e6.id()});
//     let e7 = world.entity().set(Position{x: 70, y: 80}).set(Velocity{x: 4, y: 5}); e7.set(SelfRef{value: e7.id()});

//     let q = world.query_builder<SelfRef, Position, &Velocity>()
//         .instanced()
//         .build();

//     world.set(Count(0));
//     q.each_entity(|e, (s,p,v)| {
//         assert!(e == s.value);
//         p.x += v.x;
//         p.y += v.y;
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,7);
//                 });

//     assert!(e1.get([](&Position& p) {
//         assert_eq!(p.x, 11);
//         assert_eq!(p.y, 22);
//     }));

//     assert!(e2.get([](&Position& p) {
//         assert_eq!(p.x, 21);
//         assert_eq!(p.y, 32);
//     }));

//     assert!(e3.get([](&Position& p) {
//         assert_eq!(p.x, 31);
//         assert_eq!(p.y, 42);
//     }));

//     assert!(e4.get([](&Position& p) {
//         assert_eq!(p.x, 41);
//         assert_eq!(p.y, 52);
//     }));

//     assert!(e5.get([](&Position& p) {
//         assert_eq!(p.x, 51);
//         assert_eq!(p.y, 62);
//     }));

//     assert!(e6.get([](&Position& p) {
//         assert_eq!(p.x, 62);
//         assert_eq!(p.y, 73);
//     }));

//     assert!(e7.get([](&Position& p) {
//         assert_eq!(p.x, 74);
//         assert_eq!(p.y, 85);
//     }));
// }

// #[test] fn query_un_instanced_query_w_singleton_each() {
//     let world = World::new();

//     world.set(Velocity{x: 1, y: 2});

//     let e1 = world.entity().set(Position{x: 10, y: 20}); e1.set(SelfRef{value: e1.id()});
//     let e2 = world.entity().set(Position{x: 20, y: 30}); e2.set(SelfRef{value: e2.id()});
//     let e3 = world.entity().set(Position{x: 30, y: 40}); e3.set(SelfRef{value: e3.id()});
//     let e4 = world.entity().set(Position{x: 40, y: 50}); e4.set(SelfRef{value: e4.id()});
//     let e5 = world.entity().set(Position{x: 50, y: 60}); e5.set(SelfRef{value: e5.id()});

//     e4.add::<Tag>();
//     e5.add::<Tag>();

//     let q = world.query_builder<SelfRef, Position, &Velocity>()
//         .term_at(2).singleton()
//         .build();

//     world.set(Count(0));
//     q.each_entity(|e, (s,p,v)| {
//         assert!(e == s.value);
//         p.x += v.x;
//         p.y += v.y;
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,5);
//                 });

//     assert!(e1.get([](&Position& p) {
//         assert_eq!(p.x, 11);
//         assert_eq!(p.y, 22);
//     }));

//     assert!(e2.get([](&Position& p) {
//         assert_eq!(p.x, 21);
//         assert_eq!(p.y, 32);
//     }));

//     assert!(e3.get([](&Position& p) {
//         assert_eq!(p.x, 31);
//         assert_eq!(p.y, 42);
//     }));

//     assert!(e4.get([](&Position& p) {
//         assert_eq!(p.x, 41);
//         assert_eq!(p.y, 52);
//     }));

//     assert!(e5.get([](&Position& p) {
//         assert_eq!(p.x, 51);
//         assert_eq!(p.y, 62);
//     }));
// }

// #[test] fn query_un_instanced_query_w_base_each() {
//     let world = World::new();

//     let base = world.entity().set(Velocity{x: 1, y: 2});

//     let e1 = world.entity().is_a_id(base).set(Position{x: 10, y: 20}); e1.set(SelfRef{value: e1.id()});
//     let e2 = world.entity().is_a_id(base).set(Position{x: 20, y: 30}); e2.set(SelfRef{value: e2.id()});
//     let e3 = world.entity().is_a_id(base).set(Position{x: 30, y: 40}); e3.set(SelfRef{value: e3.id()});
//     let e4 = world.entity().is_a_id(base).set(Position{x: 40, y: 50}).add::<Tag>(); e4.set(SelfRef{value: e4.id()});
//     let e5 = world.entity().is_a_id(base).set(Position{x: 50, y: 60}).add::<Tag>(); e5.set(SelfRef{value: e5.id()});
//     let e6 = world.entity().set(Position{x: 60, y: 70}).set(Velocity{x: 2, y: 3}); e6.set(SelfRef{value: e6.id()});
//     let e7 = world.entity().set(Position{x: 70, y: 80}).set(Velocity{x: 4, y: 5}); e7.set(SelfRef{value: e7.id()});

//     let q = world.query_builder<SelfRef, Position, &Velocity>()
//         .build();

//     world.set(Count(0));
//     q.each_entity(|e, (s,p,v)| {
//         assert!(e == s.value);
//         p.x += v.x;
//         p.y += v.y;
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,7);
//                 });

//     assert!(e1.get([](&Position& p) {
//         assert_eq!(p.x, 11);
//         assert_eq!(p.y, 22);
//     }));

//     assert!(e2.get([](&Position& p) {
//         assert_eq!(p.x, 21);
//         assert_eq!(p.y, 32);
//     }));

//     assert!(e3.get([](&Position& p) {
//         assert_eq!(p.x, 31);
//         assert_eq!(p.y, 42);
//     }));

//     assert!(e4.get([](&Position& p) {
//         assert_eq!(p.x, 41);
//         assert_eq!(p.y, 52);
//     }));

//     assert!(e5.get([](&Position& p) {
//         assert_eq!(p.x, 51);
//         assert_eq!(p.y, 62);
//     }));

//     assert!(e6.get([](&Position& p) {
//         assert_eq!(p.x, 62);
//         assert_eq!(p.y, 73);
//     }));

//     assert!(e7.get([](&Position& p) {
//         assert_eq!(p.x, 74);
//         assert_eq!(p.y, 85);
//     }));
// }

// #[test] fn query_instanced_query_w_singleton_iter() {
//     let world = World::new();

//     world.set(Velocity{x: 1, y: 2});

//     let e1 = world.entity().set(Position{x: 10, y: 20}); e1.set(SelfRef{value: e1.id()});
//     let e2 = world.entity().set(Position{x: 20, y: 30}); e2.set(SelfRef{value: e2.id()});
//     let e3 = world.entity().set(Position{x: 30, y: 40}); e3.set(SelfRef{value: e3.id()});
//     let e4 = world.entity().set(Position{x: 40, y: 50}); e4.set(SelfRef{value: e4.id()});
//     let e5 = world.entity().set(Position{x: 50, y: 60}); e5.set(SelfRef{value: e5.id()});

//     e4.add::<Tag>();
//     e5.add::<Tag>();

//     let q = world.query_builder<SelfRef, Position, &Velocity>()
//         .term_at(2).singleton()
//         .instanced()
//         .build();

//     world.set(Count(0));

//     q.run(|mut it| {
//         while it.next() {
//             let s = it.field::<SelfRef>(0).unwrap();
//             let mut p = it.field::<Position>(1).unwrap();
//             let v = it.field::<&Velocity>(2).unwrap();

//             assert!(it.count() > 1);
//             for i in it.iter() {
//                 p[i].x += v.x;
//                 p[i].y += v.y;
//                 assert!(it.entity(i) == s[i].value);
//                 world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             }
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,5);
//                 });

//     assert!(e1.get([](&Position& p) {
//         assert_eq!(p.x, 11);
//         assert_eq!(p.y, 22);
//     }));

//     assert!(e2.get([](&Position& p) {
//         assert_eq!(p.x, 21);
//         assert_eq!(p.y, 32);
//     }));

//     assert!(e3.get([](&Position& p) {
//         assert_eq!(p.x, 31);
//         assert_eq!(p.y, 42);
//     }));

//     assert!(e4.get([](&Position& p) {
//         assert_eq!(p.x, 41);
//         assert_eq!(p.y, 52);
//     }));

//     assert!(e5.get([](&Position& p) {
//         assert_eq!(p.x, 51);
//         assert_eq!(p.y, 62);
//     }));
// }

// #[test] fn query_instanced_query_w_base_iter() {
//     let world = World::new();

//     let base = world.entity().set(Velocity{x: 1, y: 2});

//     let e1 = world.entity().is_a_id(base).set(Position{x: 10, y: 20}); e1.set(SelfRef{value: e1.id()});
//     let e2 = world.entity().is_a_id(base).set(Position{x: 20, y: 30}); e2.set(SelfRef{value: e2.id()});
//     let e3 = world.entity().is_a_id(base).set(Position{x: 30, y: 40}); e3.set(SelfRef{value: e3.id()});
//     let e4 = world.entity().is_a_id(base).set(Position{x: 40, y: 50}).add::<Tag>(); e4.set(SelfRef{value: e4.id()});
//     let e5 = world.entity().is_a_id(base).set(Position{x: 50, y: 60}).add::<Tag>(); e5.set(SelfRef{value: e5.id()});
//     let e6 = world.entity().set(Position{x: 60, y: 70}).set(Velocity{x: 2, y: 3}); e6.set(SelfRef{value: e6.id()});
//     let e7 = world.entity().set(Position{x: 70, y: 80}).set(Velocity{x: 4, y: 5}); e7.set(SelfRef{value: e7.id()});

//     let q = world.query_builder<SelfRef, Position, &Velocity>()
//         .instanced()
//         .build();

//     world.set(Count(0));
//     q.run(|mut it| {
//         while it.next() {
//             let s = it.field::<SelfRef>(0).unwrap();
//             let mut p = it.field::<Position>(1).unwrap();
//             let v = it.field::<&Velocity>(2).unwrap();

//             assert!(it.count() > 1);
//             for i in it.iter() {
//                 if (it.is_self(2)) {
//                     p[i].x += v[i].x;
//                     p[i].y += v[i].y;
//                 } else {
//                     p[i].x += v.x;
//                     p[i].y += v.y;
//                 }

//                 assert!(it.entity(i) == s[i].value);
//                 world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             }
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,7);
//                 });

//     assert!(e1.get([](&Position& p) {
//         assert_eq!(p.x, 11);
//         assert_eq!(p.y, 22);
//     }));

//     assert!(e2.get([](&Position& p) {
//         assert_eq!(p.x, 21);
//         assert_eq!(p.y, 32);
//     }));

//     assert!(e3.get([](&Position& p) {
//         assert_eq!(p.x, 31);
//         assert_eq!(p.y, 42);
//     }));

//     assert!(e4.get([](&Position& p) {
//         assert_eq!(p.x, 41);
//         assert_eq!(p.y, 52);
//     }));

//     assert!(e5.get([](&Position& p) {
//         assert_eq!(p.x, 51);
//         assert_eq!(p.y, 62);
//     }));

//     assert!(e6.get([](&Position& p) {
//         assert_eq!(p.x, 62);
//         assert_eq!(p.y, 73);
//     }));

//     assert!(e7.get([](&Position& p) {
//         assert_eq!(p.x, 74);
//         assert_eq!(p.y, 85);
//     }));
// }

// #[test] fn query_un_instanced_query_w_singleton_iter() {
//     let world = World::new();

//     world.set(Velocity{x: 1, y: 2});

//     let e1 = world.entity().set(Position{x: 10, y: 20}); e1.set(SelfRef{value: e1.id()});
//     let e2 = world.entity().set(Position{x: 20, y: 30}); e2.set(SelfRef{value: e2.id()});
//     let e3 = world.entity().set(Position{x: 30, y: 40}); e3.set(SelfRef{value: e3.id()});
//     let e4 = world.entity().set(Position{x: 40, y: 50}); e4.set(SelfRef{value: e4.id()});
//     let e5 = world.entity().set(Position{x: 50, y: 60}); e5.set(SelfRef{value: e5.id()});

//     e4.add::<Tag>();
//     e5.add::<Tag>();

//     let q = world.query_builder<SelfRef, Position, &Velocity>()
//         .term_at(2).singleton()
//         .build();

//     world.set(Count(0));
//     q.run(|mut it| {
//         while it.next() {
//             let s = it.field::<SelfRef>(0).unwrap();
//             let mut p = it.field::<Position>(1).unwrap();
//             let v = it.field::<&Velocity>(2).unwrap();

//             assert!(it.count() == 1);
//             for i in it.iter() {
//                 p[i].x += v[i].x;
//                 p[i].y += v[i].y;
//                 assert!(it.entity(i) == s[i].value);
//                 world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             }
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,5);
//                 });

//     assert!(e1.get([](&Position& p) {
//         assert_eq!(p.x, 11);
//         assert_eq!(p.y, 22);
//     }));

//     assert!(e2.get([](&Position& p) {
//         assert_eq!(p.x, 21);
//         assert_eq!(p.y, 32);
//     }));

//     assert!(e3.get([](&Position& p) {
//         assert_eq!(p.x, 31);
//         assert_eq!(p.y, 42);
//     }));

//     assert!(e4.get([](&Position& p) {
//         assert_eq!(p.x, 41);
//         assert_eq!(p.y, 52);
//     }));

//     assert!(e5.get([](&Position& p) {
//         assert_eq!(p.x, 51);
//         assert_eq!(p.y, 62);
//     }));
// }

// #[test] fn query_un_instanced_query_w_base_iter() {
//     let world = World::new();

//     let base = world.entity().set(Velocity{x: 1, y: 2});

//     let e1 = world.entity().is_a_id(base).set(Position{x: 10, y: 20}); e1.set(SelfRef{value: e1.id()});
//     let e2 = world.entity().is_a_id(base).set(Position{x: 20, y: 30}); e2.set(SelfRef{value: e2.id()});
//     let e3 = world.entity().is_a_id(base).set(Position{x: 30, y: 40}); e3.set(SelfRef{value: e3.id()});
//     let e4 = world.entity().is_a_id(base).set(Position{x: 40, y: 50}).add::<Tag>(); e4.set(SelfRef{value: e4.id()});
//     let e5 = world.entity().is_a_id(base).set(Position{x: 50, y: 60}).add::<Tag>(); e5.set(SelfRef{value: e5.id()});
//     let e6 = world.entity().set(Position{x: 60, y: 70}).set(Velocity{x: 2, y: 3}); e6.set(SelfRef{value: e6.id()});
//     let e7 = world.entity().set(Position{x: 70, y: 80}).set(Velocity{x: 4, y: 5}); e7.set(SelfRef{value: e7.id()});

//     let q = world.query_builder<SelfRef, Position, &Velocity>()
//         .build();

//     world.set(Count(0));
//     q.run(|mut it| {
//         while it.next() {
//             let s = it.field::<SelfRef>(0).unwrap();
//             let mut p = it.field::<Position>(1).unwrap();
//             let v = it.field::<&Velocity>(2).unwrap();

//             for i in it.iter() {
//                 p[i].x += v[i].x;
//                 p[i].y += v[i].y;
//                 assert!(it.entity(i) == s[i].value);
//                 world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             }
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,7);
//                 });

//     assert!(e1.get([](&Position& p) {
//         assert_eq!(p.x, 11);
//         assert_eq!(p.y, 22);
//     }));

//     assert!(e2.get([](&Position& p) {
//         assert_eq!(p.x, 21);
//         assert_eq!(p.y, 32);
//     }));

//     assert!(e3.get([](&Position& p) {
//         assert_eq!(p.x, 31);
//         assert_eq!(p.y, 42);
//     }));

//     assert!(e4.get([](&Position& p) {
//         assert_eq!(p.x, 41);
//         assert_eq!(p.y, 52);
//     }));

//     assert!(e5.get([](&Position& p) {
//         assert_eq!(p.x, 51);
//         assert_eq!(p.y, 62);
//     }));

//     assert!(e6.get([](&Position& p) {
//         assert_eq!(p.x, 62);
//         assert_eq!(p.y, 73);
//     }));

//     assert!(e7.get([](&Position& p) {
//         assert_eq!(p.x, 74);
//         assert_eq!(p.y, 85);
//     }));
// }

// #[test] fn query_query_each_from_component() {
//     flecs::world w;

//     w.entity().set<Position>({}).set<Velocity>({});
//     w.entity().set<Position>({}).set<Velocity>({});

//     struct QueryComponent {
//         flecs::query::<&mut Position, &mut Velocity> q;
//     };

//     let q = w.query::<&mut Position, &mut Velocity>();
//     let e = w.entity().set<QueryComponent>({ q });

//     const QueryComponent *qc = e.get<QueryComponent>();
//     assert!(qc != nullptr);

//     int count = 0;
//     qc.q.each([&](Position&, Velocity&) { // Ensure we can iterate const query
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });
//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });
// }

// #[test] fn query_query_iter_from_component() {
//     flecs::world w;

//     w.entity().set<Position>({}).set<Velocity>({});
//     w.entity().set<Position>({}).set<Velocity>({});

//     struct QueryComponent {
//         flecs::query::<&mut Position, &mut Velocity> q;
//     };

//     let q = w.query::<&mut Position, &mut Velocity>();
//     let e = w.entity().set<QueryComponent>({ q });

//     const QueryComponent *qc = e.get<QueryComponent>();
//     assert!(qc != nullptr);

//     int count = 0;
//     qc.q.run(|mut it| { // Ensure we can iterate const query
//         while it.next() {
//             count += it.count();
//         }
//     });
//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });
// }

// static int invoked_count = 0;

// void EachFunc(let e, p| {
//     invoked_world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     p.x += 1;
//     p.y += 1;
// }

// void RunFunc(flecs::iter& it) {
//     test_bool(true, it.next());
//     assert_eq!(it.count(), 1);
//     let mut p = it.field::<Position>(0).unwrap();
//     invoked_world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     p.x += 1;
//     p.y += 1;
//     test_bool(false, it.next());
// }

// #[test] fn query_query_each_w_func_ptr() {
//     flecs::world w;

//     let e = w.entity().set(Position{x: 10, y: 20});

//     let q = w.query::<Position>();

//     q.each(&EachFunc);

//     assert_eq!(invoked_count, 1);

//     &Position *ptr = e.get<Position>();
//     assert_eq!(ptr.x, 11);
//     assert_eq!(ptr.y, 21);
// }

// #[test] fn query_query_iter_w_func_ptr() {
//     flecs::world w;

//     let e = w.entity().set(Position{x: 10, y: 20});

//     let q = w.query::<Position>();

//     q.run(&RunFunc);

//     assert_eq!(invoked_count, 1);

//     &Position *ptr = e.get<Position>();
//     assert_eq!(ptr.x, 11);
//     assert_eq!(ptr.y, 21);
// }

// #[test] fn query_query_each_w_func_no_ptr() {
//     flecs::world w;

//     let e = w.entity().set(Position{x: 10, y: 20});

//     let q = w.query::<Position>();

//     q.each(EachFunc);

//     assert_eq!(invoked_count, 1);

//     &Position *ptr = e.get<Position>();
//     assert_eq!(ptr.x, 11);
//     assert_eq!(ptr.y, 21);
// }

// #[test] fn query_query_iter_w_func_no_ptr() {
//     flecs::world w;

//     let e = w.entity().set(Position{x: 10, y: 20});

//     let q = w.query::<Position>();

//     q.run(RunFunc);

//     assert_eq!(invoked_count, 1);

//     &Position *ptr = e.get<Position>();
//     assert_eq!(ptr.x, 11);
//     assert_eq!(ptr.y, 21);
// }

// #[test] fn query_query_each_w_iter() {
//     flecs::world w;

//     let e1 = w.entity(); e1.set(SelfRef{value: e1.id()});
//     e1.set(Position{x: 10, y: 20});
//     let e2 = w.entity(); e2.set(SelfRef{value: e2.id()});
//     e2.set(Position{x: 20, y: 30});

//     let q = w.query::<SelfRef, Position>();

//     int32_t invoked = 0;
//     q.each([&](flecs::iter& it, int32_t i, Self& s, Position& p) {
//         assert_eq!(it.count(), 2);
//         assert_eq!(it.entity(i), s.value);
//         p.x += 1;
//         p.y += 1;
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });

//     assert_eq!(invoked, 2);

//     &Position *ptr = e1.get<Position>();
//     assert_eq!(ptr.x, 11);
//     assert_eq!(ptr.y, 21);

//     ptr = e2.get<Position>();
//     assert_eq!(ptr.x, 21);
//     assert_eq!(ptr.y, 31);
// }

// #[test] fn query_invalid_field_from_each_w_iter() {
//     install_test_abort();

//     let world = World::new();

//     world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2});

//     let q = world.query_builder<Position>()
//         .with::<Velocity>().inout()
//         .build();

//     test_expect_abort();

//     q.each([&](flecs::iter& it, size_t index, Position& p) {
//         it.field(1); // not allowed from each
//     });
// }

// #[test] fn query_invalid_field_T_from_each_w_iter() {
//     install_test_abort();

//     let world = World::new();

//     world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2});

//     let q = world.query_builder<Position>()
//         .with::<Velocity>().inout()
//         .build();

//     test_expect_abort();

//     q.each([&](flecs::iter& it, size_t index, Position& p) {
//         it.field::<Velocity>(1).unwrap(); // not allowed from each
//     });
// }

// #[test] fn query_invalid_field_const_T_from_each_w_iter() {
//     install_test_abort();

//     let world = World::new();

//     world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2});

//     let q = world.query_builder<Position>()
//         .with::<Velocity>().inout()
//         .build();

//     test_expect_abort();

//     q.each([&](flecs::iter& it, size_t index, Position& p) {
//         it.field::<&Velocity>(1).unwrap(); // not allowed from each
//     });
// }

// #[test] fn query_field_at_from_each_w_iter() {
//     let world = World::new();

//     let e1 = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2});

//     flecs::entity e2 = world.entity()
//         .set(Position{x: 20, y: 30})
//         .set(Velocity{x: 3, y: 4});

//     let q = world.query_builder<Position>()
//         .with::<Velocity>().inout()
//         .build();

//     world.set(Count(0));

//     q.each([&](flecs::iter& it, size_t row, Position& p) {
//         Velocity* v = static_cast<Velocity*>(it.field_at(1, row));
//         if (it.entity(row) == e1) {
//             assert_eq!(v.x, 1);
//             assert_eq!(v.y, 2);
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         } else if (it.entity(row) == e2) {
//             assert_eq!(v.x, 3);
//             assert_eq!(v.y, 4);
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });
// }

// #[test] fn query_field_at_T_from_each_w_iter() {
//     let world = World::new();

//     let e1 = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2});

//     flecs::entity e2 = world.entity()
//         .set(Position{x: 20, y: 30})
//         .set(Velocity{x: 3, y: 4});

//     let q = world.query_builder<Position>()
//         .with::<Velocity>().inout()
//         .build();

//     world.set(Count(0));

//     q.each([&](flecs::iter& it, size_t row, Position& p) {
//         Velocity& v = it.field_at<Velocity>(1, row);
//         if (it.entity(row) == e1) {
//             assert_eq!(v.x, 1);
//             assert_eq!(v.y, 2);
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         } else if (it.entity(row) == e2) {
//             assert_eq!(v.x, 3);
//             assert_eq!(v.y, 4);
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });
// }

// #[test] fn query_field_at_const_T_from_each_w_iter() {
//     let world = World::new();

//     let e1 = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set(Velocity{x: 1, y: 2});

//     flecs::entity e2 = world.entity()
//         .set(Position{x: 20, y: 30})
//         .set(Velocity{x: 3, y: 4});

//     let q = world.query_builder<Position>()
//         .with::<Velocity>().inout()
//         .build();

//     world.set(Count(0));

//     q.each([&](flecs::iter& it, size_t row, Position& p) {
//         &Velocity& v = it.field_at<&Velocity>(1, row);
//         if (it.entity(row) == e1) {
//             assert_eq!(v.x, 1);
//             assert_eq!(v.y, 2);
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         } else if (it.entity(row) == e2) {
//             assert_eq!(v.x, 3);
//             assert_eq!(v.y, 4);
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });
// }

// struct VelocityDerived : public Velocity {
//     VelocityDerived() { }

//     VelocityDerived(float _x, float _y, float _z) {
//         x = _x;
//         y = _y;
//         z = _z;
//     }

//     float z;
// };

// #[test] fn query_field_at_from_each_w_iter_w_base_type() {
//     let world = World::new();

//     let e1 = world.entity()
//         .set(Position{x: 10, y: 20})
//         .set<VelocityDerived>({1, 2, 3});

//     flecs::entity e2 = world.entity()
//         .set(Position{x: 20, y: 30})
//         .set<VelocityDerived>({3, 4, 5});

//     let q = world.query_builder<Position>()
//         .with::<VelocityDerived>().inout()
//         .build();

//     world.set(Count(0));

//     q.each([&](flecs::iter& it, size_t row, Position& p) {
//         Velocity* v = static_cast<Velocity*>(it.field_at(1, row));
//         if (it.entity(row) == e1) {
//             assert_eq!(v.x, 1);
//             assert_eq!(v.y, 2);
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         } else if (it.entity(row) == e2) {
//             assert_eq!(v.x, 3);
//             assert_eq!(v.y, 4);
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });
// }

// #[test] fn query_change_tracking() {
//     flecs::world w;

//     let qw = w.query::<Position>();
//     let qr = w.query_builder<&Position>()
//         .cached()
//         .build();

//     let e1 = w.entity().add::<Tag>().set(Position{x: 10, y: 20});
//     w.entity().set(Position{x: 20, y: 30});

//     test_bool(qr.changed(), true);
//     qr.run([](flecs::iter &it) { while it.next() {} });
//     test_bool(qr.changed(), false);

//     int32_t count = 0, change_count = 0;

//     qw.run(|mut it| {
//         while it.next() {
//             assert_eq!(it.count(), 1);

//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });

//             if (it.entity(0) == e1) {
//                 it.skip();
//                 continue;
//             }

//             change_world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });
//     assert_eq!(change_count, 1);

//     count = 0;
//     change_count = 0;

//     test_bool(qr.changed(), true);

//     qr.run(|mut it| {
//         while it.next() {
//             assert_eq!(it.count(), 1);

//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });

//             if (it.entity(0) == e1) {
//                 test_bool(it.changed(), false);
//             } else {
//                 test_bool(it.changed(), true);
//                 change_world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//             }
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });
//     assert_eq!(change_count, 1);
// }

// #[test] fn query_not_w_write() {
//     let world = World::new();

//     struct A {};
//     struct B {};

//     let q = world.query_builder()
//         .with::<A>()
//         .with::<B>().oper(flecs::Not).write()
//         .build();

//     let e = world.entity().add::<A>();

//     world.set(Count(0));
//     world.defer([&] {
//         q.each_entity(|e| {
//             e.add::<B>();
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         });
//     });

//     assert_eq!(1, count);
//     assert!(e.has::<B>());

//     q.each_entity(|e| {
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });

//     assert_eq!(1, count);
// }

// #[test] fn query_instanced_nested_query_w_iter() {
//     let world = World::new();

//     flecs::query::<> q1 = world.query_builder()
//         .with::<Position>()
//         .with::<Mass>().singleton().inout()
//         .build();

//     flecs::query::<> q2 = world.query_builder()
//         .with::<Velocity>()
//         .build();

//     world.add::<Mass>();
//     world.entity().add::<Velocity>();
//     world.entity().add::<Position>();
//     world.entity().add::<Position>();

//     world.set(Count(0));

//     q2.run([&](flecs::iter& it_2) {
//         while (it_2.next()) {
//             q1.iter(it_2).run([&](flecs::iter& it_1) {
//                 while (it_1.next()) {
//                     assert_eq!(it_1.count(), 1);
//                     count += it_1.count();
//                 }
//             });
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });
// }

// #[test] fn query_instanced_nested_query_w_entity() {
//     let world = World::new();

//     flecs::query::<> q1 = world.query_builder()
//         .with::<Position>()
//         .with::<Mass>().singleton().inout()
//         .build();

//     flecs::query::<> q2 = world.query_builder()
//         .with::<Velocity>()
//         .build();

//     world.add::<Mass>();
//     world.entity().add::<Velocity>();
//     world.entity().add::<Position>();
//     world.entity().add::<Position>();

//     world.set(Count(0));

//     q2.each([&](flecs::entity e_2) {
//         q1.iter(e_2).run([&](flecs::iter& it_1) {
//             while (it_1.next()) {
//                 assert_eq!(it_1.count(), 1);
//                 count += it_1.count();
//             }
//         });
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });
// }

// #[test] fn query_instanced_nested_query_w_world() {
//     let world = World::new();

//     flecs::query::<> q1 = world.query_builder()
//         .with::<Position>()
//         .with::<Mass>().singleton().inout()
//         .build();

//     flecs::query::<> q2 = world.query_builder()
//         .with::<Velocity>()
//         .build();

//     world.add::<Mass>();
//     world.entity().add::<Velocity>();
//     world.entity().add::<Position>();
//     world.entity().add::<Position>();

//     world.set(Count(0));

//     q2.run([&](flecs::iter& it_2) {
//         while (it_2.next()) {
//             q1.iter(it_2.world()).run([&](flecs::iter& it_1) {
//                 while (it_1.next()) {
//                     assert_eq!(it_1.count(), 1);
//                     count += it_1.count();
//                 }
//             });
//         }
//     });

//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });
// }

// #[test] fn query_captured_query() {
//     let world = World::new();

//     flecs::query::<Position> q = world.query::<Position>();
//     flecs::entity e_1 = world.entity().set(Position{x: 10, y: 20});

//     [=]() {
//         int count = 0;
//         q.each_entity(|e, p| {
//             assert!(e == e_1);
//             assert_eq!(p.x, 10);
//             assert_eq!(p.y, 20);
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         });
//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
//     }();
// }

// #[test] fn query_page_iter_captured_query() {
//     let world = World::new();

//     flecs::query::<Position> q = world.query::<Position>();
//     /* flecs::entity e_1 = */ world.entity().set(Position{x: 10, y: 20});
//     flecs::entity e_2 = world.entity().set(Position{x: 20, y: 30});
//     /* flecs::entity e_3 = */ world.entity().set(Position{x: 10, y: 20});

//     [=]() {
//         int count = 0;
//         q.iter().page(1, 1).each_entity(|e, p| {
//             assert!(e == e_2);
//             assert_eq!(p.x, 20);
//             assert_eq!(p.y, 30);
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         });
//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
//     }();
// }

// #[test] fn query_worker_iter_captured_query() {
//     let world = World::new();

//     flecs::query::<Position> q = world.query::<Position>();
//     /* flecs::entity e_1 = */ world.entity().set(Position{x: 10, y: 20});
//     flecs::entity e_2 = world.entity().set(Position{x: 20, y: 30});
//     /* flecs::entity e_3 = */ world.entity().set(Position{x: 10, y: 20});

//     [=]() {
//         int count = 0;
//         q.iter().worker(1, 3).each_entity(|e, p| {
//             assert!(e == e_2);
//             assert_eq!(p.x, 20);
//             assert_eq!(p.y, 30);
//             world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         });
//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
//     }();
// }

// #[test] fn query_iter_entities() {
//     let world = World::new();

//     let e1 = world.entity().set(Position{x: 10, y: 20});
//     let e2 = world.entity().set(Position{x: 10, y: 20});
//     let e3 = world.entity().set(Position{x: 10, y: 20});

//     world.query::<Position>()
//         .run(|mut it| {
//             while it.next() {
//                 assert_eq!(it.count(), 3);

//                 let entities = it.entities();
//                 assert!(entities[0] == e1);
//                 assert!(entities[1] == e2);
//                 assert!(entities[2] == e3);
//             }
//         });
// }

// #[test] fn query_iter_get_pair_w_id() {
//     let world = World::new();

//     flecs::entity Rel = world.entity();
//     flecs::entity Tgt = world.entity();
//     let e = world.entity().add(Rel, Tgt);

//     flecs::query::<> q = world.query_builder()
//         .with(Rel, flecs::Wildcard)
//         .build();

//     world.set(Count(0));

//     q.each([&](flecs::iter& it, size_t i) {
//         test_bool(true, it.id(0).is_pair());
//         assert!(it.id(0).first() == Rel);
//         assert!(it.id(0).second() == Tgt);
//         assert!(e == it.entity(i));
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//     });

//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });
// }

// #[test] fn query_query_from_entity() {
//     let world = World::new();

//     flecs::entity qe = world.entity();
//     flecs::query::<&mut Position, &mut Velocity> q1 = world.query_builder<&mut Position, &mut Velocity>(qe)
//         .build();

//     /* let e1 = */ world.entity().add::<Position>();
//     flecs::entity e2 = world.entity().add::<Position>().add::<Velocity>();

//     world.set(Count(0));
//     q1.each_entity(|e, Position&, Velocity&) {
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         assert!(e == e2);
//     });
//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });

//     flecs::query::<> q2 = world.query(qe);
//     q2.each_entity(|e| {
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         assert!(e == e2);
//     });
//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });
// }

// #[test] fn query_query_from_entity_name() {
//     let world = World::new();

//     flecs::query::<&mut Position, &mut Velocity> q1 = world.query_builder<&mut Position, &mut Velocity>("qe")
//         .build();

//     /* let e1 = */ world.entity().add::<Position>();
//     flecs::entity e2 = world.entity().add::<Position>().add::<Velocity>();

//     world.set(Count(0));
//     q1.each_entity(|e, Position&, Velocity&) {
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         assert!(e == e2);
//     });
//     world.get::<&Count>(|c| {
//                     assert_eq!(c.0,1);
//                 });

//     flecs::query::<> q2 = world.query("qe");
//     q2.each_entity(|e| {
//         world.get::<&mut Count>(|c| {
//                     c.0 += 1;
//                 });
//         assert!(e == e2);
//     });
//         world.get::<&Count>(|c| {
//                     assert_eq!(c.0,2);
//                 });
// }
