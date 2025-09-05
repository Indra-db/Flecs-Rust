# wasi-musl-libc-top-half

Rust FFI bindings and compiled library for the **top half** of musl libc targeting WASI (WebAssembly System Interface).

## Overview

This crate provides the **top half** of musl libc for WASM applications - the standard C library interface and implementation without platform-specific system calls. The "bottom half" (system calls) is provided by the WASI runtime.

### What is the Top Half?

- **Top Half** (this crate): Standard C library functions like `malloc`, `memcpy`, `strlen`, `printf`, etc.
- **Bottom Half**: Platform-specific system calls like `read`, `write`, `open` - provided by WASI

This separation allows Rust programs using C libraries to compile to `wasm32-unknown-unknown` while having access to essential libc functions.

## Features

- 🔗 **Multi-mode build system**: bindgen-only, build-from-source, or link-precompiled
- 📁 **Generated headers**: Automatic `alltypes.h` generation from musl templates  
- 🛠 **Distribution ready**: Pre-compiled `libc.a` for easy integration
- 🎯 **WASM optimized**: Specifically targeted for `wasm32-unknown-unknown`

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
wasi-musl-libc-top-half = { version = "0.1", features = ["link-libc"] }
```

Use in your Rust code:

```rust
use wasi_musl_libc_top_half::*;

unsafe {
    let ptr = malloc(1024);
    memset(ptr, 0, 1024);
    free(ptr);
}
```

## Build Features

### `bindgen` 
Generate Rust FFI bindings from musl C headers.

```bash
cargo make build-bindgen
```

### `build-libc`
Compile `libc.a` from musl source code with automatic header generation.

```bash
cargo make build-libc
```

### `link-libc` 
Link with pre-compiled `libc.a` (for end users).

```bash
cargo make check-link
```

## Build System

The crate uses a sophisticated three-stage build system:

1. **Template Processing**: Generates `alltypes.h` from musl templates
2. **C Compilation**: Compiles musl source files to `libc.a`
3. **Rust Bindings**: Creates FFI bindings via bindgen

### Generated Files

Generated files are placed in `src/generated_headers/` and automatically excluded from version control:

```
src/generated_headers/
└── bits/
    └── alltypes.h    # Generated from musl templates
```

## Project Structure

```
wasi-musl-libc-top-half/
├── Cargo.toml                    # Package configuration
├── Makefile.toml                 # Cargo-make build tasks
├── build.rs                      # Build script with header generation
├── lib/                          # Pre-compiled libraries
│   └── musl_libc_top_half.a           # Distribution library
└── src/
    ├── lib.rs                    # Main library code
    ├── wrapper.h                 # C headers for bindgen
    ├── custom_headers/           # Custom header overrides
    ├── generated_headers/        # Auto-generated headers (gitignored)
    └── libc-top-half/           # Musl source code
        └── musl/
            ├── include/          # Musl headers
            ├── src/             # Musl source files  
            ├── arch/wasm32/     # WASM32-specific code
            └── tools/           # Build tools (mkalltypes.sed)
```

## Development

### Using cargo-make

Install cargo-make for convenient development:

```bash
cargo install cargo-make
```

Available tasks:

```bash
cargo make                    # Run all checks
cargo make build-libc        # Build libc.a from source
cargo make build-bindgen     # Generate Rust bindings
cargo make check-all         # Check all feature combinations
cargo make clean-generated   # Clean and rebuild everything
cargo make dist              # Create distribution package
```

### Manual Building

```bash
# Generate bindings only
cargo check --target wasm32-unknown-unknown --features bindgen

# Build library from source  
cargo check --target wasm32-unknown-unknown --features build-libc

# Use pre-compiled library
cargo check --target wasm32-unknown-unknown --features link-libc
```

## How It Works

This crate solves the fundamental problem of using C libraries in WASM:

1. **Header Generation**: Processes musl template files to create proper type definitions
2. **Source Compilation**: Compiles essential musl functions for WASM target
3. **FFI Bindings**: Generates safe Rust interfaces to C functions
4. **Distribution**: Provides pre-compiled library for easy integration

### Benefits

✅ **No WASI conflicts**: Clean separation between bindgen and runtime  
✅ **Musl compatibility**: Uses official musl libc source code  
✅ **Build flexibility**: Multiple build modes for different use cases  
✅ **WASM optimized**: Specifically targeted for WebAssembly  
✅ **Distribution ready**: Pre-compiled libraries for end users

## License

MIT License - see LICENSE file for details.
