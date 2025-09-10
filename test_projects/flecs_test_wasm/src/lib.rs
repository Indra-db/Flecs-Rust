use flecs_ecs::prelude::*;
use wasm_bindgen::prelude::*;

mod triangle_renderer;
mod wasm_os_api;

pub use triangle_renderer::TriangleRenderer;
use wasm_os_api::setup_wasm_os_api;

#[wasm_bindgen]
#[derive(Debug, Component, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

// Helper struct to hold world and entity together
#[wasm_bindgen]
pub struct WorldState {
    world: World,
    entity_id: Entity,
}

#[wasm_bindgen]
impl WorldState {
    /// Create a new world with a simple position system
    #[wasm_bindgen(constructor)]
    pub fn new() -> WorldState {
        setup_wasm_os_api();

        let world = World::new();
        // Start at bottom left: x = -50 (left side), y = -30 (bottom)
        let entity = world.entity().set(Position { x: -50, y: -30 });
        let entity_id = entity.id();

        // Set up a system that moves the triangle from left to right
        world.system::<&mut Position>().each(|pos| {
            pos.x += 1; // Move right
            pos.y += 1;

            // Reset to left when reaching right side (x = 50)
            if pos.x > 50 {
                pos.x = -50;
                pos.y = -30;
            }
            // Keep y constant at bottom
        });

        WorldState { world, entity_id }
    }

    /// Progress the world simulation by one frame
    #[wasm_bindgen]
    pub fn progress(&mut self) {
        self.world.progress();
    }

    /// Get the current position as a Position struct
    #[wasm_bindgen]
    pub fn get_position(&self) -> Position {
        let entity = self.world.entity_from_id(self.entity_id);
        entity.cloned::<&Position>()
    }
}
