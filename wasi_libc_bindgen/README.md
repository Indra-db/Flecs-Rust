# wasi-libc-bindgen

Rust bindings for wasi-libc generated via bindgen.

## Overview

This crate provides clean Rust bindings to WASI libc functions, solving the compatibility issue between wasm-bindgen and wasi-libc by separating the concerns:

- **Bindgen stage**: Generate Rust bindings from C headers (no linking)
- **Runtime stage**: Link with compiled `libc.a` for actual implementation

## Usage

### Basic Usage

```rust
use wasi_libc_bindgen::*;

unsafe {
    let ptr = malloc(1024);
    memset(ptr, 0, 1024);
    free(ptr);
}
```

### With wasm-bindgen

Enable the `wasm-bindgen` feature for additional safe wrappers:

```toml
[dependencies]
wasi-libc-bindgen = { version = "0.1", features = ["wasm-bindgen"] }
```

```rust
use wasi_libc_bindgen::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn allocate_memory(size: usize) -> *mut u8 {
    safe_malloc(size)
}
```

## Build Requirements

1. **Headers**: Standard C headers must be available during build
2. **libc.a**: For WASM targets, place a compiled `libc.a` in the crate root
   - This provides the actual WASI implementation
   - Can be generated from wasi-sdk or similar tools

## Project Structure

```
wasi_libc_bindgen/
├── Cargo.toml          # Crate configuration
├── build.rs            # Bindgen configuration
├── libc.a              # Compiled WASI libc (place here)
└── src/
    ├── lib.rs          # Main library code
    └── wrapper.h       # C headers to bind
```

## Features

- `wasm-bindgen`: Optional wasm-bindgen compatibility layer

## Benefits

✅ **Clean separation**: Bindings generated independently of runtime  
✅ **Reusable**: Can be used by multiple projects  
✅ **wasm-bindgen compatible**: No WASI conflicts during JS binding generation  
✅ **Full functionality**: Complete WASI implementation available at runtime  

## How It Solves the Problem

1. **Bindgen stage**: Generates clean Rust bindings from headers (no WASI runtime)
2. **wasm-bindgen stage**: Processes clean WASM without WASI conflicts  
3. **Runtime stage**: Links with full WASI implementation via `libc.a`

This approach eliminates the need for complex conditional compilation while maintaining full functionality.
