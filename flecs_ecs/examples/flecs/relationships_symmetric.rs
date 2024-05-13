use crate::z_snapshot_test::*;
snapshot_test!();
use flecs_ecs::prelude::*;
#[derive(Component)]
struct TradesWith;

#[test]
fn main() {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Register TradesWith as symmetric relationship. Symmetric relationships
    // go both ways, adding (R, B) to A will also add (R, A) to B.
    world.component::<TradesWith>().add::<flecs::Symmetric>();

    // Create two players
    let player_1 = world.entity();
    let player_2 = world.entity();

    // Add (TradesWith, player_2) to player_1. This also adds
    // (TradesWith, player_1) to player_2.
    player_1.add_first::<TradesWith>(player_2);

    // Log platoon of unit
    fprintln!(
        &world,
        "Player 1 trades with Player 2: {}",
        player_1.has_first::<TradesWith>(player_2)
    ); // true
    fprintln!(
        &world,
        "Player 2 trades with Player 1: {}",
        player_2.has_first::<TradesWith>(player_1)
    ); // true

    world.get::<&Snap>(|snap| snap.test("relationships_symmetric".to_string()));

    // Output:
    //  Player 1 trades with Player 2: true
    //  Player 2 trades with Player 1: true
}
