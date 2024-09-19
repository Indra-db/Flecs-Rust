#![allow(dead_code)]

use crate::common_test::*;

#[test]
fn bulk_entity_builder_simple_add() {
    let world = World::new();

    let entities = world
        .entity_bulk(10)
        .add::<Position>()
        .add::<Velocity>()
        .build();

    assert_eq!(entities.len(), 10);

    for entity in entities {
        let entity = world.entity_from_id(entity);
        assert!(entity.has::<Position>());
        assert!(entity.has::<Velocity>());
    }
}

#[test]
fn bulk_entity_builder_simple_set() {
    let world = World::new();
    let positions: [Position; 10] = core::array::from_fn(|i| Position {
        x: i as i32,
        y: i as i32,
    });
    let velocities: [Velocity; 10] = core::array::from_fn(|i| Velocity {
        x: (i as i32) * 2,
        y: (i as i32) * 2,
    });

    let entities = world
        .entity_bulk(10)
        .set(&positions)
        .set(&velocities)
        .build();

    assert_eq!(entities.len(), 10);
    for (index, entity) in entities.into_iter().enumerate() {
        let entity = world.entity_from_id(entity);
        assert!(entity.has::<Position>());
        assert!(entity.has::<Velocity>());
        let position = entity.cloned::<&Position>();
        let velocity = entity.cloned::<&Velocity>();
        assert_eq!(position.x, velocity.x / 2);
        assert_eq!(position.y, velocity.y / 2);
        assert_eq!(position.x, index as i32);
        assert_eq!(position.y, index as i32);
    }
}

#[test]
fn bulk_entity_builder_table() {
    let world = World::new();

    let ent_id = world.entity();

    let ent = world
        .entity()
        .add::<Position>()
        .add::<Velocity>()
        .add_id(ent_id);

    let mut table = ent.table().unwrap();

    let positions: [Position; 10] = core::array::from_fn(|i| Position {
        x: i as i32,
        y: i as i32,
    });
    let velocities: [Velocity; 10] = core::array::from_fn(|i| Velocity {
        x: (i * 2) as i32,
        y: (i * 2) as i32,
    });

    let random_ent_id = world.entity();

    let entities = world
        .entity_bulk(10)
        .add::<Mass>()
        .set(&velocities)
        .add_id(random_ent_id)
        .set(&positions)
        .build_to_table(&mut table);

    assert_eq!(entities.len(), 10);

    for (index, entity) in entities.into_iter().enumerate() {
        let entity = world.entity_from_id(entity);
        assert!(entity.has::<Position>());
        assert!(entity.has::<Velocity>());
        assert!(entity.has_id(ent_id));

        assert!(!entity.has::<Mass>());
        assert!(!entity.has_id(random_ent_id));

        let position = entity.cloned::<&Position>();
        let velocity = entity.cloned::<&Velocity>();
        assert_eq!(position.x, velocity.x / 2);
        assert_eq!(position.y, velocity.y / 2);
        assert_eq!(position.x, index as i32);
        assert_eq!(position.y, index as i32);
    }
}

// panics in C
// #[test]
// fn bulk_entity_builder_zero_entities() {
//     let world = World::new();

//     let result = std::panic::catch_unwind(|| {
//         world.entity_bulk(0).add::<Position>().build();
//     });

//     assert!(result.is_err());
// }

#[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
mod debug_only {
    use super::*;

    #[test]
    fn bulk_entity_builder_max_entities() {
        let world = World::new();

        let max_count = i32::MAX as u32;
        let result = std::panic::catch_unwind(|| {
            world.entity_bulk(max_count + 1);
        });

        assert!(result.is_err());
    }

    #[test]
    fn bulk_entity_builder_invalid_component() {
        let world = World::new();

        #[derive(Component)]
        struct NonDefaultComponent {
            value: i32,
        }

        let id = world.component_id::<NonDefaultComponent>();

        let result = std::panic::catch_unwind(|| {
            world.entity_bulk(10).add_id(id).build();
        });

        assert!(result.is_err());
    }

    #[test]
    fn bulk_entity_builder_set_mismatched_length() {
        let world = World::new();
        let positions: [Position; 5] = [Position { x: 0, y: 0 }; 5];

        let result = std::panic::catch_unwind(|| {
            world.entity_bulk(10).set(&positions).build();
        });

        assert!(result.is_err());
    }
}

#[test]
fn bulk_entity_builder_with_entity_ids() {
    let world = World::new();
    let entities: Vec<Entity> = (0..10).map(|_| world.entity().id()).collect();

    let new_entities = world
        .entity_bulk_w_entity_ids(&entities)
        .add::<Position>()
        .build();

    assert_eq!(new_entities.len(), 10);

    for (index, entity) in new_entities.into_iter().enumerate() {
        let entity = world.entity_from_id(entity);
        assert!(entity.has::<Position>());

        assert_eq!(entity.id(), entities[index]);
    }
}

#[test]
fn bulk_entity_builder_add_and_set() {
    let world = World::new();
    let positions: [Position; 10] = core::array::from_fn(|i| Position {
        x: i as i32,
        y: i as i32,
    });

    let entities = world
        .entity_bulk(10)
        .add::<Velocity>()
        .set(&positions)
        .build();

    assert_eq!(entities.len(), 10);

    for entity in entities {
        let entity = world.entity_from_id(entity);
        assert!(entity.has::<Position>());
        assert!(entity.has::<Velocity>());
        let position = entity.cloned::<&Position>();
        assert_eq!(position.x, position.y);
    }
}

#[test]
#[should_panic]
fn bulk_entity_builder_build_to_table_missing_default() {
    let world = World::new();

    #[derive(Component)]
    struct NonDefaultComponent {
        value: i32,
    }

    let ent = world
        .entity()
        .add::<Position>()
        .set(NonDefaultComponent { value: 5 });

    let mut table = ent.table().unwrap();

    let positions: [Position; 10] = [Position { x: 0, y: 0 }; 10];

    world
        .entity_bulk(10)
        .set(&positions)
        .build_to_table(&mut table);
}

#[test]
fn bulk_entity_builder_duplicate_add() {
    let world = World::new();

    let entities = world
        .entity_bulk(10)
        .add::<Position>()
        .add::<Position>()
        .build();

    assert_eq!(entities.len(), 10);

    for entity in entities {
        let entity = world.entity_from_id(entity);
        assert!(entity.has::<Position>());
    }
}

#[test]
fn bulk_entity_builder_set_after_add() {
    let world = World::new();
    let positions: [Position; 10] = core::array::from_fn(|i| Position {
        x: i as i32,
        y: i as i32,
    });

    let entities = world
        .entity_bulk(10)
        .add::<Position>()
        .set(&positions)
        .build();

    assert_eq!(entities.len(), 10);

    for entity in entities {
        let entity = world.entity_from_id(entity);
        assert!(entity.has::<Position>());
        let position = entity.cloned::<&Position>();
        assert_eq!(position.x, position.y);
    }
}

#[test]
fn bulk_entity_builder_no_components() {
    let world = World::new();

    let entities = world.entity_bulk(10).build();

    assert_eq!(entities.len(), 10);

    for entity in entities {
        let entity = world.entity_from_id(entity);
        assert!(!entity.has::<Position>());
    }
}

#[test]
fn bulk_entity_builder_set_same_component_multiple_times() {
    let world = World::new();
    let positions1: [Position; 10] = [Position { x: 1, y: 1 }; 10];
    let positions2: [Position; 10] = [Position { x: 2, y: 2 }; 10];

    let entities = world
        .entity_bulk(10)
        .set(&positions1)
        .set(&positions2)
        .build();

    assert_eq!(entities.len(), 10);

    for entity in entities {
        let entity = world.entity_from_id(entity);
        let position = entity.cloned::<&Position>();
        assert_eq!(position.x, 2);
        assert_eq!(position.y, 2);
    }
}
