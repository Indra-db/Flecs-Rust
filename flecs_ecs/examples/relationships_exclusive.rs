include!("common");

// Type for Platoon relationship
#[derive(Component)]
struct Platoon;

#[allow(dead_code)]
pub fn main() -> Result<World, String> {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Register Platoon as exclusive relationship. This ensures that an entity
    // can only belong to a single Platoon.
    world.component::<Platoon>().add::<flecs::Exclusive>();

    // Create two platoons
    let platoon_1 = world.entity();
    let platoon_2 = world.entity();

    // Create a unit
    let unit = world.entity();

    // Add unit to platoon 1
    unit.add_first::<Platoon>(platoon_1);

    // Log platoon of unit
    fprintln!(
        &world,
        "Unit in platoon 1: {}",
        unit.has_first::<Platoon>(platoon_1)
    ); // true
    fprintln!(
        &world,
        "Unit in platoon 2: {}",
        unit.has_first::<Platoon>(platoon_2)
    ); // false

    fprintln!(&world);

    // Add unit to platoon 2. Because Platoon is an exclusive relationship, this
    // both removes (Platoon, platoon_1) and adds (Platoon, platoon_2) in a
    // single operation.
    unit.add_first::<Platoon>(platoon_2);

    // Log platoon of unit
    fprintln!(
        &world,
        "Unit in platoon 1: {}",
        unit.has_first::<Platoon>(platoon_1)
    ); // false
    fprintln!(
        &world,
        "Unit in platoon 2: {}",
        unit.has_first::<Platoon>(platoon_2)
    ); // true

    Ok(world)

    // Output:
    //  Unit in platoon 1: true
    //  Unit in platoon 2: false
    //
    //  Unit in platoon 1: false
    //  Unit in platoon 2: true
}
