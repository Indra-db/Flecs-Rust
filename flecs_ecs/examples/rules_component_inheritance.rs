mod common;
use common::*;

// This example shows how rules can be used to match simple inheritance trees.

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
struct Builder;

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
    world.component::<Builder>().is_a::<Unit>();

    // Create a few units
    world.new_entity_named(c"warrior_1").add::<Warrior>();
    world.new_entity_named(c"warrior_2").add::<Warrior>();

    world.new_entity_named(c"marksman_1").add::<Marksman>();
    world.new_entity_named(c"marksman_2").add::<Marksman>();

    world.new_entity_named(c"wizard_1").add::<Wizard>();
    world.new_entity_named(c"wizard_2").add::<Wizard>();

    world.new_entity_named(c"builder_1").add::<Builder>();
    world.new_entity_named(c"builder_2").add::<Builder>();

    // Create a rule to find all ranged units
    let r = world.new_rule::<&RangedUnit>();

    // Iterate the rule
    r.each_entity(|e, _| {
        println!("Unit {} found", e.name());
    });

    // Output:
    //  Unit wizard_1 found
    //  Unit wizard_2 found
    //  Unit marksman_1 found
    //  Unit marksman_2 found
}
