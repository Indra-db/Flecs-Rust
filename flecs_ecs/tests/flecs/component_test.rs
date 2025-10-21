#![allow(dead_code)]
use crate::common_test::*;

#[test]
fn temp_test_hook() {
    static mut COUNT_ADD_POS: u32 = 0;
    static mut COUNT_SET_POS: u32 = 0;
    static mut COUNT_SET_VEL: u32 = 0;
    {
        let world = World::new();
        world
            .component::<Position>()
            .on_add(|_e: EntityView, _p: &mut Position| {
                unsafe { COUNT_ADD_POS += 1 };
            })
            .on_remove(|_e: EntityView, _p: &mut Position| {
                unsafe { COUNT_ADD_POS -= 1 };
            })
            .on_set(|_e: EntityView, p: &mut Position| {
                unsafe { COUNT_SET_POS += 1 };
                p.x = 10;
                p.y = 20;
            });

        world
            .component::<Velocity>()
            .on_set(|_e: EntityView, v: &mut Velocity| {
                unsafe { COUNT_SET_VEL += 1 };
                v.x *= 10;
                v.y *= 10;
            });

        assert_eq!(unsafe { COUNT_ADD_POS }, 0);
        assert_eq!(unsafe { COUNT_SET_POS }, 0);

        let entity = world.entity().set(Position { x: 0, y: 0 });

        assert_eq!(unsafe { COUNT_ADD_POS }, 1);
        assert_eq!(unsafe { COUNT_SET_POS }, 1);

        entity.get::<&Position>(|pos| {
            assert_eq!(pos.x, 10);
            assert_eq!(pos.y, 20);
        });

        entity.set(Position { x: 0, y: 0 });
        assert_eq!(unsafe { COUNT_ADD_POS }, 1);
        assert_eq!(unsafe { COUNT_SET_POS }, 2);

        entity.get::<&Position>(|pos| {
            assert_eq!(pos.x, 10);
            assert_eq!(pos.y, 20);
        });

        let entity2 = world.entity().set(Position { x: 0, y: 0 });
        assert_eq!(unsafe { COUNT_ADD_POS }, 2);
        assert_eq!(unsafe { COUNT_SET_POS }, 3);

        entity2.get::<&Position>(|pos_e2| {
            assert_eq!(pos_e2.x, 10);
            assert_eq!(pos_e2.y, 20);

            entity.get::<&Position>(|pos_e1| {
                assert_eq!(pos_e1.x, 10);
                assert_eq!(pos_e1.y, 20);
            });
        });

        entity.remove(Position::id());
        assert_eq!(unsafe { COUNT_ADD_POS }, 1);
        assert_eq!(unsafe { COUNT_SET_POS }, 3);

        entity2.set(Velocity { x: 3, y: 5 });

        entity2.get::<&Velocity>(|vel_e2| {
            assert_eq!(vel_e2.x, 30);
            assert_eq!(vel_e2.y, 50);
        });

        assert_eq!(unsafe { COUNT_SET_VEL }, 1);

        entity.remove(Position::id());
        assert_eq!(unsafe { COUNT_ADD_POS }, 1);
        assert_eq!(unsafe { COUNT_SET_POS }, 3);
        entity2.remove(Position::id());
        assert_eq!(unsafe { COUNT_ADD_POS }, 0);
        assert_eq!(unsafe { COUNT_SET_POS }, 3);
    }
}

#[test]
fn on_component_registration() {
    #[derive(Component)]
    #[flecs(on_registration)]
    struct OnRegistration {
        x: i32,
    }

    impl OnComponentRegistration for OnRegistration {
        fn on_component_registration(world: WorldRef, component_id: Entity) {
            world.get::<&mut Count>(|count| {
                count.0 += 1;
            });

            world
                .component_untyped_from(component_id)
                .add_trait::<flecs::Prefab>();
        }
    }

    #[derive(Component)]
    #[flecs(on_registration)]
    struct OnRegistrationTag {
        x: i32,
    }

    impl OnComponentRegistration for OnRegistrationTag {
        fn on_component_registration(world: WorldRef, _component_id: Entity) {
            world.get::<&mut Count>(|count| {
                count.0 += 1;
            });
        }
    }

    #[derive(Component)]
    struct NoOnRegistration {
        x: i32,
    }

    let world = World::new();

    world.set(Count(0));

    world.component::<OnRegistration>();

    world.get::<&Count>(|count| {
        assert_eq!(count.0, 1);
    });

    assert!(
        world
            .component::<OnRegistration>()
            .has(id::<flecs::Prefab>())
    );

    world.component::<OnRegistrationTag>();

    world.get::<&Count>(|count| {
        assert_eq!(count.0, 2);
    });

    world.component::<NoOnRegistration>();

    world.get::<&Count>(|count| {
        assert_eq!(count.0, 2);
    });
}

#[test]
fn on_component_registration_named() {
    #[derive(Component)]
    #[flecs(on_registration)]
    struct OnRegistration {
        x: i32,
    }

    impl OnComponentRegistration for OnRegistration {
        fn on_component_registration(world: WorldRef, component_id: Entity) {
            world.get::<&mut Count>(|count| {
                count.0 += 1;
            });

            world
                .component_untyped_from(component_id)
                .add_trait::<flecs::Prefab>();
        }
    }

    #[derive(Component)]
    #[flecs(on_registration)]
    struct OnRegistrationTag {
        x: i32,
    }

    impl OnComponentRegistration for OnRegistrationTag {
        fn on_component_registration(world: WorldRef, _component_id: Entity) {
            world.get::<&mut Count>(|count| {
                count.0 += 1;
            });
        }
    }

    #[derive(Component)]
    struct NoOnRegistration {
        x: i32,
    }

    let world = World::new();

    world.set(Count(0));

    world.component_named::<OnRegistration>("OnRegistration");

    world.get::<&Count>(|count| {
        assert_eq!(count.0, 1);
    });

    assert!(
        world
            .component::<OnRegistration>()
            .has(id::<flecs::Prefab>())
    );

    world.component::<OnRegistrationTag>();

    world.get::<&Count>(|count| {
        assert_eq!(count.0, 2);
    });

    world.component::<NoOnRegistration>();

    world.get::<&Count>(|count| {
        assert_eq!(count.0, 2);
    });
}
