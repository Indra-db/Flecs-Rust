use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Component)]
pub struct TypeWithEntity {
    pub e: Entity,
}

#[test]
fn main() {
    let mut world = World::new();

    // Using Entity directly would resolve to a u64 datatype, so
    // use flecs::meta::Entity instead.
    world
        .component::<TypeWithEntity>()
        .member::<flecs::meta::Entity>("e", 1, offset_of!(TypeWithEntity, e));

    let bar = world.entity_named("bar");

    // Create a new entity with the TypeWithEntity component
    let e = world.entity().set(TypeWithEntity { e: bar.into() });

    // Convert TypeWithEntity component to flecs expression string
    e.get::<&TypeWithEntity>(|p| {
        let expr: String = world.to_expr(p);
        println!("TypeWithEntity: {}", expr);
    });

    // Output:
    //  TypeWithEntity: {e: foo}
}
