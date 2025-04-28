#![allow(dead_code)]
use crate::common_test::*;

#[test]
fn table_eq_test() {
    let world = World::new();

    let ent1 = world.entity().set(Position { x: 10, y: 20 });

    let x = ent1.fetch::<&Position>();
    println!("position: {x:?}");
}
