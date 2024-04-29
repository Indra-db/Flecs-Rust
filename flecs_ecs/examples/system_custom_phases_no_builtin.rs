include!("common");

// This application demonstrates how to use custom phases for systems. The
// default pipeline will automatically run systems for custom phases as long as
// they have the flecs::Phase tag.

// Dummy system
fn sys(it: &mut Iter) {
    fprintln!(it, "system {}", it.system().name());
}

#[allow(dead_code)]
pub fn main() -> Result<World, String> {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Create three custom phases. Note that the phases have the Phase tag,
    // which is necessary for the builtin pipeline to discover which systems it
    // should run.

    let update = world.entity().add::<flecs::pipeline::Phase>();

    let physics = world
        .entity()
        .add::<flecs::pipeline::Phase>()
        .depends_on_id(update);

    let collisions = world
        .entity()
        .add::<flecs::pipeline::Phase>()
        .depends_on_id(physics);

    // Create 3 dummy systems.
    world
        .system_named::<()>(c"CollisionSystem")
        .kind_id(collisions)
        .iter_only(sys);

    world
        .system_named::<()>(c"PhysicsSystem")
        .kind_id(physics)
        .iter_only(sys);

    world
        .system_named::<()>(c"GameSystem")
        .kind_id(update)
        .iter_only(sys);

    // Run pipeline
    world.progress();

    Ok(world)

    // Output:
    //   system GameSystem
    //   system PhysicsSystem
    //   system CollisionSystem
}
