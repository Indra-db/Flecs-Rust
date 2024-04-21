mod common;
use common::*;

fn iterate_components(entity: EntityView, snap: &mut Snap) {
    // 1. The easiest way to print the components is to use archetype
    fprintln!(snap, "[{:?}]", entity.archetype());
    fprintln!(snap);
    // 2. To get individual component ids, use for_each
    let mut count_components = 0;
    entity.each_component(|id| {
        fprintln!(snap, "{}: {}", count_components, id.to_str());
        count_components += 1;
    });
    fprintln!(snap);

    // 3. we can also inspect and print the ids in our own way. This is a
    // bit more complicated as we need to handle the edge cases of what can be
    // encoded in an id, but provides the most flexibility.
    count_components = 0;

    entity.each_component(|id| {
        snap.str
            .last_mut()
            .unwrap()
            .push_str(format!("{}: ", count_components).as_str());

        count_components += 1;
        if id.is_pair() {
            // If id is a pair, extract & print both parts of the pair
            let rel = id.first_id();
            let target = id.second_id();
            snap.str
                .last_mut()
                .unwrap()
                .push_str(format!("rel: {}, target: {}", rel.name(), target.name()).as_str());
        } else {
            // Id contains a regular entity. Strip role before printing.
            let comp = id.entity_view();
            snap.str
                .last_mut()
                .unwrap()
                .push_str(format!("entity: {}", comp.name()).as_str());
        }

        println!("{}", snap.str.last().unwrap());
        snap.push(String::new());
    });
}
fn main() {
    //ignore snap in example, it's for snapshot testing
    let mut snap = Snap::setup_snapshot_test();

    let world = World::new();

    let bob = world
        .entity_named(c"Bob")
        .set(Position { x: 10.0, y: 20.0 })
        .set(Velocity { x: 1.0, y: 1.0 })
        .add::<Human>()
        .add::<(Eats, Apples)>();

    fprintln!(snap, "Bob's components:");
    iterate_components(bob, &mut snap);

    fprintln!(snap);

    // We can use the same function to iterate the components of a component
    fprintln!(snap, "Position's components:");
    iterate_components(world.component::<Position>().entity(), &mut snap);

    snap.test();

    // Output:
    //  Bob's components:
    //  [Position, Velocity, Human, (Identifier,Name), (Eats,Apples)]
    //
    //  0: Position
    //  1: Velocity
    //  2: Human
    //  3: (Identifier,Name)
    //  4: (Eats,Apples)
    //
    //  0: entity: Position
    //  1: entity: Velocity
    //  2: entity: Human
    //  3: rel: Identifier, target: Name
    //  4: rel: Eats, target: Apples
    //
    //  Position's components:
    //  [Component, (Identifier,Name), (Identifier,Symbol)
    //
    //  0: Component
    //  1: (Identifier,Name)
    //  2: (Identifier,Symbol)
    //  3: (ChildOf,entity_iterate_components.common)

    //  0: entity: Component
    //  1: rel: Identifier, target: Name
    //  2: rel: Identifier, target: Symbol
    //  3: rel: ChildOf, target: common
}
