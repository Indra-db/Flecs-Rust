# Flecs WASM Test Project

This project demonstrates how to use **Flecs ECS** with **wasm-bindgen** and **wasi-libc** using a single-project approach with conditional compilation.

## Quick Start

```bash
# Run the complete demo
cargo make demo-bindgen

# Or step by step:
cargo make build-bindgen    # Build with conditional compilation
cargo make test-bindgen     # Verify the build
cargo make serve-bindgen    # Test in browser
```

## Approach

This project uses a **single-project conditional compilation** approach to solve the wasm-bindgen + wasi-libc incompatibility:

1. **Stage 1**: Build with `bindgen_only` feature → Clean WASM (no WASI functions)
2. **Stage 2**: Generate JS bindings with wasm-bindgen → Success (no WASI conflicts)  
3. **Stage 3**: Build full runtime → WASM with Flecs + WASI support
4. **Stage 4**: Replace bindgen WASM with runtime WASM → Full functionality

## Key Features

- ✅ **Single project** - no separate `bindgen_minimal` needed
- ✅ **Conditional compilation** - same code, different builds
- ✅ **Clean separation** - bindgen vs runtime builds
- ✅ **Full functionality** - complete Flecs ECS + WASI support

## Documentation

- **[SINGLE_PROJECT_GUIDE.md](./SINGLE_PROJECT_GUIDE.md)** - Complete guide (recommended)
- **[WHY_TWO_STAGE.md](./WHY_TWO_STAGE.md)** - Historical context and evolution

## Project Structure

```
src/
├── lib.rs          # Main library with conditional compilation
├── bindgen.rs      # Flecs integration (when flecs_ecs feature enabled)
└── lib_simple.rs   # Alternative simple implementation

build.rs            # Conditional linking logic
Cargo.toml          # Conditional dependencies and features
Makefile.toml       # Build automation tasks
```

## Build Modes

### Bindgen-Only Mode
```bash
cargo build --target wasm32-unknown-unknown --no-default-features --features bindgen_only
```
- No flecs_ecs dependency
- No libc.a linking  
- Clean WASM for wasm-bindgen

### Full Runtime Mode
```bash
cargo build --target wasm32-unknown-unknown
```
- Full flecs_ecs dependency
- Complete libc.a linking
- WASI functions available

## Browser Testing

The generated `pkg/` directory contains:
- `flecs_test_wasm.js` - JS bindings (from clean WASM)
- `flecs_test_wasm_bg.wasm` - Runtime WASM (full Flecs + WASI)

Test at: `http://localhost:8000/web/bindgen_test.html`

## Available Functions

```javascript
import init, { add, greet, run_flecs_bindgen_test } from './pkg/flecs_test_wasm.js';

await init();

// Basic functions (always available)
add(2, 3);                    // Returns 5
greet("WASM");               // Returns "Hello, WASM!"

// Flecs ECS functionality (runtime only)  
run_flecs_bindgen_test();    // Full ECS system with entities, components, systems
```

## Technical Achievement

This project demonstrates the **first successful integration** of:
- Flecs ECS (C library)
- wasi-libc (POSIX environment)  
- wasm-bindgen (JS integration)
- Single Rust project

Previous attempts required separate projects or complex workarounds. The conditional compilation + build.rs approach provides a clean, maintainable solution.
