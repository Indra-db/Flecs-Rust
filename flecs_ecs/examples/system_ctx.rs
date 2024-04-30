include!("common");

use std::{
    ffi::c_void,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Component)]
struct Radius {
    value: f32,
}

fn sqr(value: f32) -> f32 {
    value * value
}

fn distance_sqr(p1: &Position, p2: &Position) -> f32 {
    sqr(p2.x - p1.x) + sqr(p2.y - p1.y)
}

fn rand(max: u64) -> f32 {
    let start = SystemTime::now();

    //this rand method isn't great, but it's good enough for this example
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    // Use some aspect of the current time to generate a number
    let random_number = since_the_epoch.as_secs() ^ since_the_epoch.subsec_nanos() as u64;

    (random_number % max) as f32
}

#[allow(dead_code)]
pub fn main() -> Result<Snap, String> {
    let world = World::new();

    //ignore snap in example, it's for snapshot testing
    world.import::<Snap>();

    // Applications can pass context data to a system. A common use case where this
    // comes in handy is when a system needs to iterate more than one query. The
    // following example shows how to pass a custom query into a system for a simple
    // collision detection example.

    let mut query_collide = world.new_query::<(&Position, &Radius)>();

    let sys = world
        .system::<(&Position, &Radius)>()
        .set_context(&mut query_collide as *mut Query<(&Position, &Radius)> as *mut c_void)
        .each_iter(|it, index, (p1, r1)| {
            let query = unsafe { it.context::<Query<(&Position, &Radius)>>() };
            let e1 = it.entity(index);

            query.each_entity(|e2, (p2, r2)| {
                if e1 == *e2 {
                    // don't collide with self
                    return;
                }

                if e1 > *e2 {
                    // Simple trick to prevent collisions from being detected
                    // twice with the entities reversed.
                    return;
                }

                // Check for collision
                let d_sqr = distance_sqr(p1, p2);
                let r_sqr = sqr(r1.value + r2.value);
                if r_sqr > d_sqr {
                    fprintln!(it, "{} and {} collided!", e1, e2);
                }
            });
        });

    // Create a few test entities
    for _ in 0..10 {
        world
            .entity()
            .set(Position {
                x: rand(100),
                y: rand(100),
            })
            .set(Radius {
                value: rand(10) + 1.0,
            });
    }

    // Run the system
    sys.run();

    Ok(Snap::from(&world))

    // Output:
    //  532 and 539 collided!
    //  532 and 540 collided!
    //  534 and 538 collided!
    //  536 and 537 collided!
    //  536 and 540 collided!
    //  537 and 540 collided!
}
