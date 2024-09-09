#![allow(dead_code)]
use crate::common_test::*;

#[test]
fn component_lifecycle_count_in_add_hook() {
    let world = World::new();

    world.set(Count(0));

    world.component::<Position>().on_add(|e, _| {
        e.world().get::<&mut Count>(|count| {
            count.0 += 1;
        });
    });

    world.entity().set(Position { x: 1, y: 2 });

    assert_eq!(world.cloned::<&Count>().0, 1);

    world.new_query::<&Position>().each_entity(|e, _| {
        e.world().get::<&mut Count>(|count| {
            count.0 += 1;
        });
    });

    assert_eq!(world.cloned::<&Count>().0, 2);
}

#[test]
fn component_lifecycle_count_in_remove_hook() {
    let world = World::new();

    world.set(Count(0));

    world.component::<Position>().on_remove(|e, _| {
        e.world().set(Count(e.world().count::<Position>()));
    });

    let entity = world.entity().set(Position { x: 1, y: 2 });
    assert_eq!(world.cloned::<&Count>().0, 0);

    entity.destruct();
    assert_eq!(world.cloned::<&Count>().0, 1);

    world.set(Count(0));

    world.new_query::<&Position>().each_entity(|e, _| {
        e.world().get::<&mut Count>(|count| {
            count.0 += 1;
        });
    });

    assert_eq!(world.cloned::<&Count>().0, 0);
}
