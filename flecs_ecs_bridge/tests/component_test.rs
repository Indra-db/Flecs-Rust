use flecs_ecs_bridge::core::{entity::Entity, world::World};

mod common;
use common::*;

#[test]
fn temp_test_hook() {
    static mut COUNT: u32 = 0;
    static mut COUNT2: u32 = 0;
    {
        let world = World::default();
        world
            .component::<Position>()
            .on_add(|_e: Entity, p: &mut Position| {
                unsafe { COUNT += 1 };
                p.x = 10.0;
                p.y = 20.0;
            })
            .on_remove(|_e: Entity, _p: &mut Position| {
                unsafe { COUNT -= 1 };
            });

        world
            .component::<Velocity>()
            .on_set(|_e: Entity, v: &mut Velocity| {
                unsafe { COUNT2 += 1 };
                v.x *= 10.0;
                v.y *= 10.0;
            });

        assert_eq!(unsafe { COUNT }, 0);

        let entity = world.new_entity().add_component::<Position>();

        assert_eq!(unsafe { COUNT }, 1);
        let pos_e1 = entity.get_component::<Position>();
        assert_eq!(unsafe { (*pos_e1).x }, 10.0);
        assert_eq!(unsafe { (*pos_e1).y }, 20.0);

        entity.add_component::<Position>();
        assert_eq!(unsafe { COUNT }, 1);
        let pos_e1 = entity.get_component::<Position>();
        assert_eq!(unsafe { (*pos_e1).x }, 10.0);
        assert_eq!(unsafe { (*pos_e1).y }, 20.0);

        let entity2 = world.new_entity().add_component::<Position>();
        assert_eq!(unsafe { COUNT }, 2);
        let pos_e2 = entity2.get_component::<Position>();
        assert_eq!(unsafe { (*pos_e1).x }, 10.0);
        assert_eq!(unsafe { (*pos_e1).y }, 20.0);
        assert_eq!(unsafe { (*pos_e2).x }, 10.0);
        assert_eq!(unsafe { (*pos_e2).y }, 20.0);

        entity.add_component::<Velocity>();
        assert_eq!(unsafe { COUNT2 }, 0);
        let vel_e1 = entity.get_component::<Velocity>();
        assert_eq!(unsafe { (*vel_e1).x }, 0.0);
        assert_eq!(unsafe { (*vel_e1).y }, 0.0);
        entity.remove_component::<Velocity>();

        entity.remove_component::<Position>();
        assert_eq!(unsafe { COUNT }, 1);

        entity2.set_component(Velocity { x: 3.0, y: 5.0 });
        let vel_e2 = entity2.get_component::<Velocity>();
        assert_eq!(unsafe { (*vel_e2).x }, 30.0);
        assert_eq!(unsafe { (*vel_e2).y }, 50.0);
        assert_eq!(unsafe { COUNT2 }, 1);

        entity.remove_component::<Position>();
        assert_eq!(unsafe { COUNT }, 1);
        entity2.remove_component::<Position>();
        assert_eq!(unsafe { COUNT }, 0);
    }
}
