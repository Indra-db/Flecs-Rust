use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;
#[derive(Component)]
struct TradesWith;

fn main() {
    let world = World::new();

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
    println!(
        "Player 1 trades with Player 2: {}",
        player_1.has_first::<TradesWith>(player_2)
    ); // true
    println!(
        "Player 2 trades with Player 1: {}",
        player_2.has_first::<TradesWith>(player_1)
    ); // true

    // Output:
    //  Player 1 trades with Player 2: true
    //  Player 2 trades with Player 1: true
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("relationships_symmetric".to_string());
}
