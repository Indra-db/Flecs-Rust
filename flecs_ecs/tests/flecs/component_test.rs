#![allow(dead_code)]
use crate::common_test::*;

#[test]
fn temp_test_hook() {
    static mut COUNT: u32 = 0;
    static mut COUNT2: u32 = 0;
    {
        structs!();
        let world = World::new();
        world
            .component::<Position>()
            .on_add(|_e: EntityView, p: &mut Position| {
                unsafe { COUNT += 1 };
                p.x = 10;
                p.y = 20;
            })
            .on_remove(|_e: EntityView, _p: &mut Position| {
                unsafe { COUNT -= 1 };
            });

        world
            .component::<Velocity>()
            .on_set(|_e: EntityView, v: &mut Velocity| {
                unsafe { COUNT2 += 1 };
                v.x *= 10;
                v.y *= 10;
            });

        assert_eq!(unsafe { COUNT }, 0);

        let entity = world.entity().add::<Position>();

        assert_eq!(unsafe { COUNT }, 1);

        entity.get::<&Position>(|pos| {
            assert_eq!(pos.x, 10);
            assert_eq!(pos.y, 20);
        });

        entity.add::<Position>();
        assert_eq!(unsafe { COUNT }, 1);

        entity.get::<&Position>(|pos| {
            assert_eq!(pos.x, 10);
            assert_eq!(pos.y, 20);
        });

        let entity2 = world.entity().add::<Position>();
        assert_eq!(unsafe { COUNT }, 2);

        entity2.get::<&Position>(|pos_e2| {
            assert_eq!(pos_e2.x, 10);
            assert_eq!(pos_e2.y, 20);

            entity.get::<&Position>(|pos_e1| {
                assert_eq!(pos_e1.x, 10);
                assert_eq!(pos_e1.y, 20);
            });
        });

        entity.remove::<Position>();
        assert_eq!(unsafe { COUNT }, 1);

        entity2.set(Velocity { x: 3, y: 5 });

        entity2.get::<&Velocity>(|vel_e2| {
            assert_eq!(vel_e2.x, 30);
            assert_eq!(vel_e2.y, 50);
        });

        assert_eq!(unsafe { COUNT2 }, 1);

        entity.remove::<Position>();
        assert_eq!(unsafe { COUNT }, 1);
        entity2.remove::<Position>();
        assert_eq!(unsafe { COUNT }, 0);
    }
}
