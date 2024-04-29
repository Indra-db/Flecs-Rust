include!("common");

extern "C" fn compare_position(
    _e1: EntityT,
    p1: *const Position,
    _e2: EntityT,
    p2: *const Position,
) -> std::ffi::c_int {
    let p1 = unsafe { &*p1 };
    let p2 = unsafe { &*p2 };

    (p1.x > p2.x) as i32 - (p1.x < p2.x) as i32
}

fn print_query(query: &Query<&Position>) {
    query.each_entity(|entity, pos| fprintln!(entity, "{:?}", pos));
}

#[allow(dead_code)]
pub fn main() -> Result<Snap, String> {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Create entities, set position in random order
    let entity = world.entity().set(Position { x: 1.0, y: 0.0 });
    world.entity().set(Position { x: 6.0, y: 0.0 });
    world.entity().set(Position { x: 2.0, y: 0.0 });
    world.entity().set(Position { x: 5.0, y: 0.0 });
    world.entity().set(Position { x: 4.0, y: 0.0 });

    // Create a sorted query
    let query = world
        .query::<&Position>()
        .order_by(compare_position)
        .build();

    // Create a sorted system
    let sys = world
        .system::<&Position>()
        .order_by(compare_position)
        .each_entity(|entity, pos| {
            fprintln!(entity, "{:?}", pos);
        });

    fprintln!(&world);
    fprintln!(&world, "--- First iteration ---");
    print_query(&query);

    // Change the value of one entity, invalidating the order
    entity.set(Position { x: 7.0, y: 0.0 });

    // Iterate query again, printed values are still ordered
    fprintln!(&world);
    fprintln!(&world, "--- Second iteration ---");
    print_query(&query);

    // Create new entity to show that data is also sorted for new entities
    world.entity().set(Position { x: 3.0, y: 0.0 });

    // Run system, printed values are ordered
    fprintln!(&world);
    fprintln!(&world, "--- System iteration ---");
    sys.run();

    Ok(Snap::from(&world))

    // Output:
    //
    //  --- First iteration ---
    //  Position { x: 1.0, y: 0.0 }
    //  Position { x: 2.0, y: 0.0 }
    //  Position { x: 4.0, y: 0.0 }
    //  Position { x: 5.0, y: 0.0 }
    //  Position { x: 6.0, y: 0.0 }
    //
    //  --- Second iteration ---
    //  Position { x: 2.0, y: 0.0 }
    //  Position { x: 4.0, y: 0.0 }
    //  Position { x: 5.0, y: 0.0 }
    //  Position { x: 6.0, y: 0.0 }
    //  Position { x: 7.0, y: 0.0 }
    //
    //  --- System iteration ---
    //  Position { x: 2.0, y: 0.0 }
    //  Position { x: 3.0, y: 0.0 }
    //  Position { x: 4.0, y: 0.0 }
    //  Position { x: 5.0, y: 0.0 }
    //  Position { x: 6.0, y: 0.0 }
    //  Position { x: 7.0, y: 0.0 }
}
