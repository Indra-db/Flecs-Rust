# WebAssembly Support for Flecs-Rust

Flecs-Rust supports compilation to WebAssembly (`wasm32-unknown-unknown`) and requires wasi-libc headers for compilation.

## Requirements

**wasi-libc is required** for WASM compilation. The build system will automatically detect wasi-libc installations, but you must install it first.

## Installation

### macOS (Homebrew)
```bash
brew install wasi-libc
```

### Ubuntu/Debian
```bash
sudo apt install wasi-libc-dev
```

### Arch Linux
```bash
sudo pacman -S wasi-libc
```

### Manual Installation
Download wasi-sdk from [WebAssembly/wasi-sdk releases](https://github.com/WebAssembly/wasi-sdk/releases)

### Custom Path
Set the `WASI_SYSROOT_INCLUDE` environment variable:
```bash
export WASI_SYSROOT_INCLUDE=/path/to/wasi-libc/include/wasm32-wasi
cargo build --target wasm32-unknown-unknown
```

## Quick Start

After installing wasi-libc:

```bash
cargo build --target wasm32-unknown-unknown
```

## Detection Methods

The build system automatically searches for wasi-libc headers in this order:

1. **`WASI_SYSROOT_INCLUDE` environment variable** - User override
2. **Homebrew** - `brew --prefix wasi-libc` (macOS)  
3. **pkg-config** - `pkg-config --variable=includedir wasi-libc` (Linux)
4. **Common paths** - Standard installation locations
5. **wasi-sdk detection** - Find via `wasi-clang` binary

## Runtime Requirements

Your JavaScript runtime needs to provide WASI function stubs:

```javascript
const imports = {
    wasi_snapshot_preview1: {
        clock_time_get: (clock_id, precision, time_ptr) => {
            const timeNs = BigInt(Math.floor(performance.now() * 1000000));
            const memory = new DataView(wasmInstance.exports.memory.buffer);
            memory.setBigUint64(time_ptr, timeNs, true);
            return 0;
        },
        proc_exit: () => 0,
        environ_get: () => 0,
        environ_sizes_get: () => 0,
        // ... other WASI functions as needed
    }
};
```

## Cross-Platform Compatibility

The build system automatically detects wasi-libc on:
- **macOS**: Homebrew, MacPorts
- **Linux**: apt, pacman, pkg-config  
- **Windows**: wasi-sdk installations
- **Any platform**: Environment variable override

## Troubleshooting

### Build Errors

If compilation fails with "wasi-libc headers not found":

1. **Install wasi-libc** using your package manager (see Installation section)
2. **Set custom path**: `export WASI_SYSROOT_INCLUDE=/path/to/headers` 
3. **Verify installation**: Check that the headers exist in the expected location

### Runtime Errors

Common runtime issues:
- **Missing WASI functions**: Add stubs to your JavaScript imports
- **Memory access errors**: Ensure proper WASM memory handling

### Supported Targets

- ✅ `wasm32-unknown-unknown` (recommended)
- ⚠️ `wasm32-wasi` (experimental)

## Architecture

```
┌─────────────────┐    ┌──────────────┐    ┌─────────────────┐
│   Your Code     │ -> │ Flecs-Rust   │ -> │ WASM Module     │
└─────────────────┘    └──────────────┘    └─────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │                   │
            ┌───────v────────┐ ┌────────v────────┐
            │ wasi-libc      │ │ wasi-libc.a     │
            │ (headers)      │ │ (runtime)       │
            └────────────────┘ └─────────────────┘
```

The build system uses wasi-libc headers for compilation and links against wasi-libc.a for runtime implementation.
