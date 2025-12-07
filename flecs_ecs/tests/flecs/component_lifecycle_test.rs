#![allow(dead_code)]
use std::{
    collections::HashSet,
    sync::{LazyLock, Mutex},
};

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
        e.world().set(Count(e.world().count(Position::id())));
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

static INITIALIZED_BOXES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

#[derive(Component, Clone)]
struct BoxedNumber {
    number: Box<u32>,
    id: String,
    dropped: bool,
}

impl BoxedNumber {
    fn new(id: &str) -> Self {
        INITIALIZED_BOXES.lock().unwrap().insert(id.to_string());
        Self {
            number: Box::new(10),
            id: id.to_string(),
            dropped: false,
        }
    }
}

impl Drop for BoxedNumber {
    fn drop(&mut self) {
        assert!(!self.dropped, "double-free detected");
        self.dropped = true;
        INITIALIZED_BOXES.lock().unwrap().remove(&self.id);
    }
}

#[test]
fn component_lifecycle_drop_on_remove() {
    let world = World::new();

    world
        .entity_named("object 1")
        .set(BoxedNumber::new("Object 1"));
    world
        .entity_named("object 2")
        .set(BoxedNumber::new("Object 2"));
    world
        .entity_named("object 3")
        .set(BoxedNumber::new("Object 3"));

    world.defer_begin();
    world.query::<&BoxedNumber>().build().each_entity(|ent, _| {
        ent.each_component(|e| {
            ent.remove(e);
        });
    });
    world.defer_end();

    let init_boxes = INITIALIZED_BOXES.lock().unwrap();
    assert!(
        init_boxes.is_empty(),
        "Leaked memory, objects not properly deleted: {:?}",
        init_boxes
    );
}

#[test]
fn component_lifecycle_set_singleton() {
    {
        let world = World::new();

        world
            .component::<BoxedNumber>()
            .add_trait::<flecs::Singleton>();

        world.system::<()>().run(|it| {
            let world = it.world();
            world.set(BoxedNumber::new("Object 1"));
        });

        world.progress();
    }
}

#[test]
fn component_lifecycle_drop_on_world_delete() {
    {
        let world = World::new();

        world
            .entity_named("object 1")
            .set(BoxedNumber::new("Object 1"));
        world
            .entity_named("object 2")
            .set(BoxedNumber::new("Object 2"));
        world
            .entity_named("object 3")
            .set(BoxedNumber::new("Object 3"));

        world.quit();

        world.progress();
    }

    let init_boxes = INITIALIZED_BOXES.lock().unwrap();
    assert!(
        init_boxes.is_empty(),
        "Leaked memory, objects not properly deleted: {:?}",
        init_boxes
    );
}

#[test]
fn component_lifecycle_set_multiple_times() {
    let world = World::new();

    let ent = world
        .entity_named("object 1")
        .set(BoxedNumber::new("Object 1"));
    ent.set(BoxedNumber::new("Object 2"));
    ent.set(BoxedNumber::new("Object 3"));
    ent.destruct();

    let init_boxes = INITIALIZED_BOXES.lock().unwrap();
    assert!(
        init_boxes.is_empty(),
        "Leaked memory, objects not properly deleted: {:?}",
        init_boxes
    );
}
