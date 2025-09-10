use flecs_ecs::prelude::*;
use wasm_bindgen::prelude::*;

mod wasm_os_api;
use wasm_os_api::setup_wasm_os_api;

#[wasm_bindgen]
#[derive(Debug, Component, Clone, Copy)]
pub struct Position {
    x: i32,
    y: i32,
}

#[wasm_bindgen]
impl Position {
    #[wasm_bindgen(constructor)]
    pub fn new(x: i32, y: i32) -> Position {
        Position { x, y }
    }

    #[wasm_bindgen(getter)]
    pub fn x(&self) -> i32 {
        self.x
    }

    #[wasm_bindgen(getter)]
    pub fn y(&self) -> i32 {
        self.y
    }

    #[wasm_bindgen(setter)]
    pub fn set_x(&mut self, x: i32) {
        self.x = x;
    }

    #[wasm_bindgen(setter)]
    pub fn set_y(&mut self, y: i32) {
        self.y = y;
    }
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
        let entity = world.entity().set(Position { x: 10, y: 10 });
        let entity_id = entity.id();

        // Set up a simple system that increments position
        world.system::<&mut Position>().each(|pos| {
            pos.x += 1;
            pos.y += 1;
        });

        WorldState { world, entity_id }
    }

    /// Progress the world simulation by one frame
    #[wasm_bindgen]
    pub fn progress(&mut self) {
        self.world.progress();
    }

    /// Get the current X position of the entity
    #[wasm_bindgen]
    pub fn get_position_x(&self) -> i32 {
        let entity = self.world.entity_from_id(self.entity_id);
        let pos = entity.cloned::<&Position>();
        pos.x
    }

    /// Get the current Y position of the entity
    #[wasm_bindgen]
    pub fn get_position_y(&self) -> i32 {
        let entity = self.world.entity_from_id(self.entity_id);
        let pos = entity.cloned::<&Position>();
        pos.y
    }

    /// Get the current position as a Position struct
    #[wasm_bindgen]
    pub fn get_position(&self) -> Position {
        let entity = self.world.entity_from_id(self.entity_id);
        entity.cloned::<&Position>()
    }
}
