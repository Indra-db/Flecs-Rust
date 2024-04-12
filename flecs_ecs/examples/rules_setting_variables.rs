mod common;
use common::*;

// This example extends the component_inheritance example, and shows how
// we can use a single rule to match units from different players and platoons
// by setting query variables before we iterate.
//
// The units in this example belong to a platoon, with the platoons belonging
// to a player.

// unit datamodel
#[derive(Component)]
struct Unit;

#[derive(Component)]
struct CombatUnit;

#[derive(Component)]
struct MeleeUnit;

#[derive(Component)]
struct RangedUnit;

#[derive(Component)]
struct Warrior;

#[derive(Component)]
struct Wizard;

#[derive(Component)]
struct Marksman;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Platoon;

const PLAYER_COUNT: usize = 100;
const PLATOONS_PER_PLAYER: usize = 3;

fn main() {
    let world = World::new();

    // Make the ECS aware of the inheritance relationships. Note that IsA
    // relationship used here is the same as in the prefab example.
    world.component::<CombatUnit>().is_a::<Unit>();
    world.component::<MeleeUnit>().is_a::<CombatUnit>();
    world.component::<RangedUnit>().is_a::<CombatUnit>();

    world.component::<Warrior>().is_a::<MeleeUnit>();
    world.component::<Wizard>().is_a::<RangedUnit>();
    world.component::<Marksman>().is_a::<RangedUnit>();

    // Populate store with players and platoons
    for p in 0..PLAYER_COUNT {
        let player = if p == 0 {
            // Give first player a name so we can look it up later
            world.new_entity_named(c"MyPlayer")
        } else {
            world.new_entity()
        };

        // Add player tag so we can query for all players if we want to
        player.add::<Player>();

        for _ in 0..PLATOONS_PER_PLAYER {
            let platoon = world
                .new_entity()
                .add_pair_first::<Player>(player)
                // Add platoon tag so we can query for all platoons if we want to
                .add::<Platoon>();

            // Add warriors, wizards and marksmen to the platoon
            world
                .new_entity()
                .add::<Warrior>()
                .add_pair_first::<Platoon>(platoon);
            world
                .new_entity()
                .add::<Marksman>()
                .add_pair_first::<Platoon>(platoon);
            world
                .new_entity()
                .add::<Wizard>()
                .add_pair_first::<Platoon>(platoon);
        }
    }

    // Create a rule to find all RangedUnits for a platoon/player. The
    // equivalent query in the query DSL would look like this:
    //   (Platoon, $Platoon), Player($Platoon, $Player)
    //
    // The way to read how this query is evaluated is:
    // - find all entities with (Platoon, *), store * in _Platoon
    // - check if _Platoon has (Player, *), store * in _Player
    let rule = world
        .rule::<&RangedUnit>()
        .with_type::<&Platoon>()
        .select_second_name(c"$Platoon")
        .with_pair_name::<&Player>(c"$Player")
        .select_src_name(c"$Platoon")
        .build();

    // If we would iterate this rule it would return all ranged units for all
    // platoons & for all players. We can limit the results to just a single
    // platoon or a single player setting a variable beforehand. In this example
    // we'll just find all platoons & ranged units for a single player.

    let player_var = rule.find_var(c"Player");
    let platoon_var = rule.find_var(c"Platoon");

    // Iterate rule, limit the results to units of MyPlayer
    rule.iterable()
        .set_var(player_var, world.lookup_name(c"MyPlayer", true))
        .each_iter(|it, index, _| {
            let unit = it.entity(index);
            println!(
                "Unit {} of class {} in platoon {} for player {}",
                unit,
                it.id(1).to_str(),
                it.get_var(platoon_var),
                it.get_var(player_var)
            );
        });

    // Output:
    //  Unit 529 of class Wizard in platoon 526 for player MyPlayer
    //  Unit 533 of class Wizard in platoon 530 for player MyPlayer
    //  Unit 537 of class Wizard in platoon 534 for player MyPlayer
    //  Unit 528 of class Marksman in platoon 526 for player MyPlayer
    //  Unit 532 of class Marksman in platoon 530 for player MyPlayer
    //  Unit 536 of class Marksman in platoon 534 for player MyPlayer

    // Try removing the set_var call, this will cause the iterator to return
    // all units in all platoons for all players.
}
