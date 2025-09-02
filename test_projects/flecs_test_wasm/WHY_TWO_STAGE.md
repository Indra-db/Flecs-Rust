# Historical: Why Two-Stage Build Was Necessary

> **⚠️ DEPRECATED**: This document describes the old two-project approach. 
> 
> **✅ CURRENT**: See `SINGLE_PROJECT_GUIDE.md` for the recommended single-project approach using conditional compilation.

## Background

This document explains the technical reasoning behind the original two-stage build approach, which has been superseded by a more elegant single-project solution.

## The Original Problem

The issue with wasm-bindgen + wasi-libc is **not** at the Rust dependency level - it's at the WebAssembly target level.

### Proof: WASI Functions in "Clean" WASM

Even when building with `--no-default-features --features bindgen_only` (no flecs_ecs dependency), the resulting WASM contains WASI imports:

```bash
$ wasm-objdump -x flecs_test_wasm.wasm | grep wasi_snapshot_preview1
 - func[4] <__imported_wasi_snapshot_preview1_environ_get>
 - func[5] <__imported_wasi_snapshot_preview1_environ_sizes_get>
 - func[6] <__imported_wasi_snapshot_preview1_fd_close>
 - func[7] <__imported_wasi_snapshot_preview1_fd_fdstat_get>
 # ... 15 total WASI imports
```

### Root Cause: wasm32-unknown-unknown Target

The `wasm32-unknown-unknown` target automatically includes WASI runtime functions, regardless of Rust dependencies. This is by design - it provides a POSIX-like environment for C code.

## Why Single-Project Approach Failed (Originally)

### Attempted Solution: Conditional Compilation
```rust
#[cfg(feature = "bindgen_only")]  // ❌ Still contained WASI (in original attempt)
fn minimal_build() { /* ... */ }

#[cfg(not(feature = "bindgen_only"))]  // ✅ Full Flecs + WASI
fn full_build() { /* ... */ }
```

### The Original Reality
- **bindgen_only build**: Still had WASI imports (from target and linking)
- **wasm-bindgen**: Cannot interpret WASI functions
- **Result**: `__wasilibc_initialize_environ: unknown instruction`

## Why Two-Stage Build Worked (Historically)

### Stage 1: Clean WASM Generation
- Use a separate project with **no C dependencies**
- Compile to wasm32-unknown-unknown 
- Result: WASM without WASI conflicts
- wasm-bindgen can process this successfully

### Stage 2: Runtime WASM with Full Features
- Build main project with Flecs + wasi-libc
- Gets all WASI functions for runtime

## Evolution to Single-Project Solution

The breakthrough came from realizing that **build.rs conditional linking** was the missing piece:

```rust
// The key insight: conditionally skip libc.a linking
if !std::env::var("CARGO_FEATURE_BINDGEN_ONLY").is_ok() {
    // Only link libc.a for full builds, not bindgen_only
    println!("cargo:rustc-link-arg={}/libc.a", current_dir.display());
}
```

This allows the same project to produce:
1. **Clean WASM** (bindgen_only feature) - no WASI functions
2. **Full WASM** (default features) - with WASI functions

## Lessons Learned

1. **Feature flags alone** weren't sufficient - linking behavior needed control
2. **build.rs** is crucial for conditional system-level dependencies  
3. **Single project** is achievable with proper conditional compilation + linking
4. **Optional dependencies** + conditional features provide the flexibility needed

## Migration Path

**From two-project approach:**
```bash
# Old way
cd bindgen_minimal && cargo build --target wasm32-unknown-unknown
cd ../main_project && cargo build --target wasm32-unknown-unknown
```

**To single-project approach:**
```bash
# New way  
cargo build --target wasm32-unknown-unknown --no-default-features --features bindgen_only
cargo build --target wasm32-unknown-unknown  # full build
```

The single-project approach in `SINGLE_PROJECT_GUIDE.md` represents the current best practice.
- Replace binding WASM with full runtime WASM
- Browser uses full implementation at runtime

## Conclusion

The two-project approach is not just a workaround - it's the **correct architectural solution** to a fundamental incompatibility between:

1. **wasm-bindgen's interpreter** (cannot handle WASI)
2. **wasm32-unknown-unknown target** (automatically includes WASI)

The single-project approach cannot solve this because the problem exists at the WebAssembly target level, not the Rust dependency level.
