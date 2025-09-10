# Flecs ECS + WebAssembly Demo

A clean, minimal example of using [Flecs ECS](https://github.com/SanderMertens/flecs) with Rust and WebAssembly, specifically targeting `wasm32-unknown-unknown` with `wasm-bindgen`.

## 🎯 Learning Goals

This project demonstrates:
- How to use Flecs ECS in Rust with WebAssembly
- Setting up a custom OS API for WASM environments
- Using `wasm-bindgen` for clean JavaScript/Rust interop
- Managing ECS worlds, entities, components, and systems in WASM

## 📁 Project Structure

```
├── src/
│   ├── lib.rs           # Main library with Flecs integration
│   └── wasm_os_api.rs   # WASM-compatible OS API implementations
├── web/
│   ├── demo.html        # Simple, clean demo page
│   ├── app.js           # Core JavaScript functionality
│   ├── styles.css       # Clean styling
│   ├── index.html       # Full-featured demo (legacy)
│   └── pkg/             # Generated wasm-bindgen output
├── Cargo.toml
└── Makefile.toml        # Build tasks
```

## 🚀 Quick Start

1. **Build and serve the demo in one command:**
   ```bash
   cargo make serve-web
   ```
   This will build with wasm-bindgen and start a server on port 8002.

2. **Or build and serve separately:**
   ```bash
   cargo make build-bindgen
   python3 -m http.server 8002
   ```

3. **Open the demo:**
   - Clean learning demo: `http://localhost:8002/web/demo.html`
   - Full-featured demo: `http://localhost:8002/web/index.html`

## 🧩 Core Components

### Rust Side (`src/lib.rs`)

```rust
// Simple Position component
#[wasm_bindgen]
#[derive(Debug, Component, Clone, Copy)]
pub struct Position {
    pub x: i32,  // wasm-bindgen auto-generates getters/setters
    pub y: i32,
}

// World wrapper with ECS logic
#[wasm_bindgen]
pub struct WorldState {
    world: World,
    entity_id: Entity,
}

#[wasm_bindgen]
impl WorldState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WorldState { /* ... */ }
    
    #[wasm_bindgen]
    pub fn progress(&mut self) { /* ... */ }
    
    #[wasm_bindgen]
    pub fn get_position(&self) -> Position { /* ... */ }
}
```

### JavaScript Side (`web/app.js`)

```javascript
// Load and initialize WASM
const { default: init, WorldState, Position } = await import('./pkg/flecs_test_wasm.js');
await init();

// Use the classes
const world = new WorldState();
world.progress();
const pos = world.get_position();
console.log(`Position: (${pos.x}, ${pos.y})`);
```

## 🔧 Key Features

### WASM OS API (`src/wasm_os_api.rs`)
- Custom memory management (malloc, free, realloc, calloc)
- Time functions for ECS frame timing
- Threading stubs (WASM is single-threaded)
- Mutex/condition variable no-ops

### Clean wasm-bindgen Integration
- Automatic getter/setter generation for public struct fields
- Proper memory management with RAII
- Type-safe JavaScript bindings

### ECS Demonstration
- Entity creation with Position component
- Simple system that increments position each frame
- World progression and state querying

## 📚 Learning Path

1. **Start with `demo.html`** - Clean, minimal interface
2. **Examine `src/lib.rs`** - Core Flecs integration
3. **Check `src/wasm_os_api.rs`** - WASM environment setup
4. **Study `app.js`** - JavaScript/WASM interaction

## 🛠️ Build Commands

```bash
# Build and serve (recommended for development)
cargo make serve-web

# Just the clean learning demo
cargo make demo

# Build with wasm-bindgen only
cargo make build-bindgen

# Build regular WASM (manual loading)
cargo make build-wasm
```

## 💡 Key Insights

### Why wasm32-unknown-unknown?
- No assumptions about the host environment
- Requires custom OS API implementations
- Better control over WASM module size and dependencies

### WASM OS API Challenges
- No native threading (stubs required)
- No native file system
- Custom memory management
- Time functions need manual implementation

### wasm-bindgen Benefits
- Automatic JavaScript binding generation
- Type-safe interop
- Proper memory management
- ES6 module support

## 📖 Further Reading

- [Flecs Documentation](https://www.flecs.dev/flecs/)
- [wasm-bindgen Book](https://rustwasm.github.io/wasm-bindgen/)
- [WebAssembly Concepts](https://developer.mozilla.org/en-US/docs/WebAssembly/Concepts)
- [Rust and WebAssembly](https://rustwasm.github.io/docs/book/)

## 🐛 Common Issues

1. **"env" module import errors**: Fixed by providing custom console functions in the generated JS
2. **Memory leaks**: Use `.free()` on wasm-bindgen objects when done
3. **Threading errors**: All operations must be single-threaded in WASM

## 🎮 Demo Features

- **Create World**: Initialize a new ECS world with an entity
- **Progress**: Advance the simulation by one frame
- **Get Position**: Query the current entity position
- **Destroy**: Clean up the world and free resources

The entity starts at position (10, 10) and moves by (1, 2) each frame thanks to the simple system defined in Rust.
