# Single-Project WASM-Bindgen Guide

## Overview

This guide demonstrates how to use wasm-bindgen with wasi-libc using a **single-project approach** with conditional compilation. This eliminates the need for separate `bindgen_minimal` projects while solving the fundamental incompatibility between wasm-bindgen and WASI functions.

## The Problem

wasm-bindgen cannot interpret WASI functions present in WASM built for `wasm32-unknown-unknown` when using wasi-libc:

```
Error: __wasilibc_initialize_environ: unknown instruction
```

## The Solution: Single-Project Conditional Compilation

### Key Components

1. **Conditional Dependencies** in `Cargo.toml`
2. **Conditional Compilation** in source code  
3. **Conditional Linking** in `build.rs`

### Implementation

#### 1. Cargo.toml Configuration

```toml
[dependencies]
# Make flecs_ecs optional - only included in full builds
flecs_ecs = { path = "../../flecs_ecs", default-features = false, optional = true }

# Always available dependencies
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["console"] }

[features]
# Two distinct build modes:
bindgen_only = []  # Minimal build for JS binding generation
default = ["flecs_ecs", "flecs_base"]  # Full build with Flecs ECS

# Forward features to flecs_ecs when available
flecs_base = ["flecs_ecs?/flecs_base"]
flecs_use_os_alloc = ["flecs_ecs?/flecs_use_os_alloc"]
```

#### 2. Conditional Compilation in Source

```rust
// Always available functions
//#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 { a + b }

//#[wasm_bindgen] 
pub fn greet(name: &str) -> String { format!("Hello, {}!", name) }

// Flecs functionality only when feature is enabled
#[cfg(feature = "flecs_ecs")]
mod flecs_impl {
    use flecs_ecs::prelude::*;
    
    //#[wasm_bindgen]
    pub fn run_flecs_bindgen_test() {
        // Full Flecs ECS functionality here
    }
}

// Re-export when available
#[cfg(feature = "flecs_ecs")]
pub use flecs_impl::*;
```

#### 3. Conditional Linking in build.rs

```rust
fn main() {
    if std::env::var("TARGET").unwrap_or_default() == "wasm32-unknown-unknown" {
        let is_bindgen_only = std::env::var("CARGO_FEATURE_BINDGEN_ONLY").is_ok();
        
        if !is_bindgen_only {
            // Only link libc.a for full builds
            println!("cargo:rustc-link-search=native={}", current_dir.display());
            println!("cargo:rustc-link-arg=--whole-archive");
            println!("cargo:rustc-link-arg={}/libc.a", current_dir.display());
            println!("cargo:rustc-link-arg=--no-whole-archive");
            println!("cargo:rustc-link-arg=--allow-undefined");
            println!("cargo:rustc-link-arg=--no-entry");
        }
    }
}
```

## Build Process

### Stage 1: Generate Clean JS Bindings

```bash
# Build without flecs_ecs dependency and without libc.a linking
cargo build --target wasm32-unknown-unknown --no-default-features --features bindgen_only

# Generate JS bindings from clean WASM (no WASI functions)
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/debug/flecs_test_wasm.wasm
```

**Result**: Clean WASM with only wasm-bindgen imports, no WASI functions.

### Stage 2: Build Full Runtime

```bash
# Build with full Flecs functionality and WASI support
cargo build --target wasm32-unknown-unknown

# Replace the bindgen WASM with the full runtime version
cp target/wasm32-unknown-unknown/debug/flecs_test_wasm.wasm pkg/flecs_test_wasm_bg.wasm
```

**Result**: Full Flecs ECS + WASI runtime capabilities.

## Verification

### Check WASM Imports

**Bindgen-only build** (clean):
```bash
wasm-objdump -x pkg/flecs_test_wasm_bg.wasm | grep Import
# Shows only wasm-bindgen functions, no WASI
```

**Full build** (runtime):
```bash  
wasm-objdump -x target/wasm32-unknown-unknown/debug/flecs_test_wasm.wasm | grep wasi
# Shows WASI functions for full runtime
```

### Test in Browser

```javascript
import init, { add, greet, run_flecs_bindgen_test } from './pkg/flecs_test_wasm.js';

await init();

// Basic functions work in both modes
console.log(add(2, 3));        // 5
console.log(greet("WASM"));    // "Hello, WASM!"

// Flecs functionality only in full runtime
run_flecs_bindgen_test();      // Full ECS system
```

## Advantages of Single-Project Approach

✅ **Simplified Project Structure**: No separate `bindgen_minimal` project needed  
✅ **DRY Principle**: Single source of truth for all functionality  
✅ **Easier Maintenance**: One codebase to maintain  
✅ **Flexible Features**: Easy to add/remove functionality via features  
✅ **Clear Separation**: Conditional compilation makes intent explicit  

## Makefile Tasks

```bash
# Complete demo
cargo make demo-bindgen

# Build only  
cargo make build-bindgen

# Test build
cargo make test-bindgen

# Serve in browser
cargo make serve-bindgen
```

## Troubleshooting

### "unknown instruction" Error

If you see WASI-related errors, ensure:
1. `bindgen_only` feature excludes `flecs_ecs` dependency
2. `build.rs` skips libc.a linking for `bindgen_only` builds
3. Using `--no-default-features` for bindgen stage

### Missing Exports

If functions are missing in JS:
1. Check `//#[wasm_bindgen]` annotations
2. Verify conditional compilation with `#[cfg(feature = "...")]`
3. Ensure re-exports for conditional modules

### Build Failures

Common issues:
1. Missing `optional = true` for flecs_ecs in Cargo.toml
2. Incorrect feature names in dependencies
3. Missing environment variable checks in build.rs

## Comparison with Two-Project Approach

| Aspect | Single-Project | Two-Project |
|--------|---------------|-------------|
| **Projects** | 1 | 2 |
| **Complexity** | Lower | Higher |
| **Maintenance** | Easier | Harder |
| **Code Duplication** | None | Some |
| **Build Steps** | 3 | 4+ |
| **Feature Flexibility** | High | Medium |

The single-project approach is now the **recommended method** for integrating wasm-bindgen with wasi-libc in Flecs Rust projects.
