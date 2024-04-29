include!("common");

#[derive(Component)]
struct TradesWith;

#[allow(dead_code)]
pub fn main() -> Result<Snap, String> {
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

    Ok(Snap::from(&world))

    // Output:
    //  Player 1 trades with Player 2: true
    //  Player 2 trades with Player 1: true
}
