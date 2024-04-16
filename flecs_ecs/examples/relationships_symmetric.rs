mod common;
use common::*;

#[derive(Component)]
struct TradesWith;

fn main() {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    // Register TradesWith as symmetric relationship. Symmetric relationships
    // go both ways, adding (R, B) to A will also add (R, A) to B.
    world.component::<TradesWith>().add::<flecs::Symmetric>();

    // Create two players
    let player_1 = world.entity();
    let player_2 = world.entity();

    // Add (TradesWith, player_2) to player_1. This also adds
    // (TradesWith, player_1) to player_2.
    player_1.add_pair_first::<TradesWith>(player_2);

    // Log platoon of unit
    fprintln!(
        snap,
        "Player 1 trades with Player 2: {}",
        player_1.has_pair_first::<TradesWith>(player_2)
    ); // true
    fprintln!(
        snap,
        "Player 2 trades with Player 1: {}",
        player_2.has_pair_first::<TradesWith>(player_1)
    ); // true

    snap.test();

    // Output:
    //  Player 1 trades with Player 2: true
    //  Player 2 trades with Player 1: true
}
