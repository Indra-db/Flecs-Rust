mod common;
use common::*;

#[derive(Clone, Component, Debug, Default)]
struct TradesWith;

fn main() {
    let world = World::new();

    // Register TradesWith as symmetric relationship. Symmetric relationships
    // go both ways, adding (R, B) to A will also add (R, A) to B.
    world.component::<TradesWith>().add_id(ECS_SYMMETRIC);

    // Create two players
    let player_1 = world.new_entity();
    let player_2 = world.new_entity();

    // Add (TradesWith, player_2) to player_1. This also adds
    // (TradesWith, player_1) to player_2.
    player_1.add_pair_first::<TradesWith>(player_2);

    // Log platoon of unit
    println!(
        "Player 1 trades with Player 2: {}",
        player_1.has_pair_first::<TradesWith>(player_2)
    ); // true
    println!(
        "Player 2 trades with Player 1: {}",
        player_2.has_pair_first::<TradesWith>(player_1)
    ); // true

    // Output:
    //  Player 1 trades with Player 2: true
    //  Player 2 trades with Player 1: true
}
