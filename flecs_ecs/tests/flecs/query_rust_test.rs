#![allow(dead_code)]
use flecs_ecs::core::*;

use crate::common_test::*;

#[test]
fn query_rust_pass_query_to_system() {
    let world = World::new();

    world.set(Position { x: 1, y: 2 });

    let query = world
        .query::<(&Position, &Velocity)>()
        .term_at(0)
        .singleton()
        .set_cached()
        .build();

    world.entity().set(Velocity { x: 590, y: 20 });

    let query_entity = query.entity().id();

    let sys = world.system::<()>().run(move |it| {
        let world = it.world();
        let query = world.query_from(query_entity);
        query.run(|mut it| {
            let mut count = 0;
            while it.next() {
                let pos = &it.field::<&Position>(0).unwrap()[0]; //singleton
                let vel = it.field::<&Velocity>(1).unwrap();
                for i in it.iter() {
                    count += 1;
                    assert_eq!(pos.x, 1);
                    assert_eq!(pos.y, 2);
                    assert_eq!(vel[i].x, 590);
                    assert_eq!(vel[i].y, 20);
                }
            }
            assert_eq!(count, 1);
        });
    });

    sys.run();
}

#[test]
fn test_trait_query() {
    pub trait Shapes {
        fn calculate(&self) -> u64;
    }

    // Define a ShapesTrait component with the necessary functionality
    ecs_rust_trait!(Shapes);

    #[derive(Component)]
    pub struct Circle {
        radius: f32,
    }

    //implement the Shapes trait as you want
    impl Shapes for Circle {
        fn calculate(&self) -> u64 {
            1
        }
    }

    #[derive(Component)]
    pub struct Square {
        side: f32,
    }

    impl Shapes for Square {
        fn calculate(&self) -> u64 {
            2
        }
    }

    #[derive(Component)]
    pub struct Triangle {
        side: f32,
    }

    impl Shapes for Triangle {
        fn calculate(&self) -> u64 {
            3
        }
    }

    let world = World::new();

    // Register a vtable per component that implements the trait through the ShapesTrait component
    ShapesTrait::register_vtable::<Circle>(&world);
    ShapesTrait::register_vtable::<Square>(&world);
    ShapesTrait::register_vtable::<Triangle>(&world);

    world.entity_named("circle").set(Circle { radius: 5.0 });
    world.entity_named("square").set(Square { side: 5.0 });
    world.entity_named("triangle").set(Triangle { side: 5.0 });

    // query for all entities with components that implement the Shapes trait
    let query = world.query::<&ShapesTrait>().build();

    let mut count = 0;

    query.run(|mut it| {
        while it.next() {
            for i in it.iter() {
                let e = it.entity(i).unwrap();
                let id = it.id(0);
                // cast the component to the Shapes trait
                let shape: &dyn Shapes = ShapesTrait::cast(e, id);
                // call the method on the trait
                let calc = shape.calculate();
                count += calc;
            }
        }
    });

    assert_eq!(count, 6);
}
