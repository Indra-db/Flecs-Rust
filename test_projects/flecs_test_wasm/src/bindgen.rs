use flecs_ecs::prelude::*;
use wasm_bindgen::prelude::*;

// Import the `console.log` function from the browser's console API
//#[wasm_bindgen]
extern "C" {
    //#[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro for easier console logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Define a simple component
#[derive(Component, Debug)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component, Debug)]
struct Velocity {
    x: f32,
    y: f32,
}

// Exported function that can be called from JavaScript
//#[wasm_bindgen]
pub fn run_flecs_bindgen_test() {
    console_log!("Starting Flecs wasm-bindgen test...");

    // Set up the WASM OS API
    crate::setup_wasm_os_api();

    // Create a world
    let world = World::new();
    console_log!("World created");

    // Create an entity with Position and Velocity components
    let entity = world
        .entity()
        .set(Position { x: 0.0, y: 0.0 })
        .set(Velocity { x: 1.0, y: 0.5 });

    console_log!("Entity created with Position and Velocity");

    // Get the initial position
    entity.try_get::<&Position>(|pos| {
        console_log!("Initial position: x={}, y={}", pos.x, pos.y);
    });

    // Create a movement system
    world
        .system::<(&mut Position, &Velocity)>()
        .each(|(pos, vel)| {
            pos.x += vel.x;
            pos.y += vel.y;
        });

    console_log!("Movement system created");

    // Run the system a few times
    for i in 1..=3 {
        world.progress();

        entity.try_get::<&Position>(|pos| {
            console_log!("After step {}: position x={}, y={}", i, pos.x, pos.y);
        });
    }

    console_log!("Flecs wasm-bindgen test completed successfully!");
}

// Export a simple function to test basic wasm-bindgen functionality
//#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Export a function that returns a string
//#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
