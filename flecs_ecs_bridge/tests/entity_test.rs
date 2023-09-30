use flecs_ecs_bridge::core::world::World;

struct Parent {
    entity_type: EntityType,
}

struct EntityType;

#[test]
fn entity_new() {
    let world = World::default();
    let entity = world.new_entity();
    assert_eq!(entity.get_is_valid(), true);
}

#[test]
fn entity_new_named() {
    let world = World::default();
    let entity = world.new_entity_named("test");
    assert_eq!(entity.get_is_valid(), true);
    assert_eq!(entity.get_name(), "test");
}

#[test]
fn entity_new_named_from_scope() {
    let world = World::default();
    let entity = world.new_entity_named("Foo");
    assert_eq!(entity.get_is_valid(), true);

    let prev = world.set_scope_id(entity.raw_id);
    let child = world.new_entity_named("Bar");
    assert_eq!(child.get_is_valid(), true);

    world.set_scope_id(prev.raw_id);

    assert_eq!(child.get_name(), "Bar");
    assert_eq!(child.get_hierachy_path_default().unwrap(), "::Foo::Bar");
}
