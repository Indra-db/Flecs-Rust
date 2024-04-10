mod common;
use common::*;

// When an enumeration constant is added to an entity, it is added as a relationship
// pair where the relationship is the enum type, and the target is the constant. For
// example, this statement:
//   e.add(Color::Red)
//
// adds this relationship:
//   (Color, Color::Red)
//
// Enums are registered as exclusive relationships, which means that adding an
// enum constant will replace the previous constant for that enumeration:
//   e.add(Color::Green)
//
//  will replace Color::Red with Color::Green

//Regular C style enumerations are supported
//   rust style enum variants are not supported *yet* due to limitations in C flecs lib
//   where it expects each enum field to be 4 bytes.
//   I plan on adding support for rust style enums in the future.
#[derive(Component, Debug, PartialEq)]
#[repr(C)]
enum Tile {
    Grass,
    Sand,
    Stone,
}

#[derive(Component)]
#[repr(C)]
enum TileStatus {
    Free,
    Occupied,
}

fn main() {
    let world = World::new();

    // Create an entity with (Tile, Red) and (TileStatus, Free) relationships
    let tile = world
        .new_entity()
        .add_enum(Tile::Stone)
        .add_enum(TileStatus::Free);

    // (Tile, Tile.Stone), (TileStatus, TileStatus.Free)
    println!("{:?}", tile.archetype());

    // Replace (TileStatus, Free) with (TileStatus, Occupied)
    tile.add_enum(TileStatus::Occupied);

    // (Tile, Tile.Stone), (TileStatus, TileStatus.Occupied)
    println!("{:?}", tile.archetype());

    println!();

    // Check if the entity has the Tile relationship and the Tile::Stone pair
    println!("has tile enum: {}", tile.has::<Tile>()); // true
    println!(
        "is the enum from tile stone?: {}",
        tile.has_enum(Tile::Stone)
    ); // true

    // Get the current value of the enum
    let v = tile.get::<Tile>();
    if let Some(tile_value) = v {
        println!("is tile stone: {}", *tile_value == Tile::Stone); // true
    }

    // Create a few more entities that we can query
    world
        .new_entity()
        .add_enum(Tile::Grass)
        .add_enum(TileStatus::Free);

    world
        .new_entity()
        .add_enum(Tile::Sand)
        .add_enum(TileStatus::Occupied);

    println!();

    // Iterate all entities with a Tile relationship
    world
        .filter_builder::<()>()
        .with_enum_wildcard::<Tile>()
        .build()
        .each_iter(|it, _, _| {
            let pair = it.pair(1).unwrap();
            let tile_constant = pair.second();
            println!("{}", tile_constant.path().unwrap());
        });

    // Outputs:
    //  ::Tile::Stone
    //  ::Tile::Grass
    //  ::Tile::Sand

    println!();

    // Iterate only occupied tiles
    world
        .filter_builder::<()>()
        .with_enum_wildcard::<Tile>()
        .with_enum(TileStatus::Occupied)
        .build()
        .each_iter(|it, _, _| {
            let pair = it.pair(1).unwrap();
            let tile_constant = pair.second();
            println!("{}", tile_constant.path().unwrap());
        });

    // Outputs:
    //  ::Tile::Stone
    //  ::Tile::Sand

    println!();

    // Remove any instance of the TileStatus relationship
    tile.remove::<TileStatus>();

    // (Tile, Tile.Stone)
    println!("{:?}", tile.archetype());
}
