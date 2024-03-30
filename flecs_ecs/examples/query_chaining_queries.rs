mod common;

use common::*;

// this example is to showcase how you can chain queries together where the second query
// uses the results of the first query to filter the results

#[derive(Default, Clone, Component)]
struct Enchanted;

#[derive(Default, Clone, Component)]
struct Location {
    x: f32,
    y: f32,
}

#[derive(Default, Clone, Component)]
struct Ability {
    power: f32,
}

#[derive(Default, Clone, Component)]
struct ArtifactPower {
    _magic_level: f32,
}

fn main() {
    let forest = World::new();

    // Populate the forest with creatures. Some are enchanted.
    for i in 0..10 {
        let creature = forest
            .new_entity()
            .set(Location {
                x: i as f32 * 10.0,
                y: i as f32 * 5.0,
            })
            .set(Ability {
                power: i as f32 * 1.5,
            });

        if i % 2 == 0 {
            creature.add::<Enchanted>();
        }
    }

    // Introduce mystical artifacts into the forest, some of which are also enchanted
    for i in 0..10 {
        let artifact = forest
            .new_entity()
            .set(Location { x: -1.0, y: -1.0 }) //to showcase we don't match this
            .set(ArtifactPower {
                _magic_level: i as f32 * 2.5,
            });

        if i % 2 != 0 {
            // Differentiate enchantment condition to diversify
            artifact.add::<Enchanted>();
        }
    }

    // Query for creatures based on their Location and Ability
    let query_creatures = forest.query::<(&Location, &Ability)>();

    // Query specifically for enchanted things in the world
    let query_enchanted = forest
        .rule_builder::<()>()
        .with_type::<&Enchanted>()
        .build();

    // Iterate over creatures to find the enchanted ones
    query_creatures.iter(|it, (loc, ability)| {

        // Filter for enchanted creatures within the current iteration
        query_enchanted
            .iterable()
            .set_var_as_range(0, it.get_table_range())
            .each_iter( |it, index ,()| {
               let pos = &loc[index];
               let abil_power = ability[index].power;
               let entity = it.get_entity(index);
                println!(
                    "Creature {entity} at location {},{} is enchanted with mystical energy, ability power: {} "
                    , pos.x, pos.y, abil_power

                );
            });
    });

    // Output:
    // Creature 525 at location 0,0 is enchanted with mystical energy, ability power: 0
    // Creature 527 at location 20,10 is enchanted with mystical energy, ability power: 3
    // Creature 529 at location 40,20 is enchanted with mystical energy, ability power: 6
    // Creature 531 at location 60,30 is enchanted with mystical energy, ability power: 9
    // Creature 533 at location 80,40 is enchanted with mystical energy, ability power: 12
}
