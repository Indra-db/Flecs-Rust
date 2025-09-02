# Custom Headers

This directory contains custom header files that are needed for the WASI libc build but should be preserved when updating the upstream musl sources in `libc-top-half/`.

## Files

- `__macro_PAGESIZE.h`: Defines the PAGESIZE macro for WASM32 (64KB page size). This is included by `libc-top-half/musl/arch/wasm32/bits/limits.h`.

## Why These Are Separate

These headers are kept outside of the `libc-top-half/` directory so they won't be overwritten when you update the upstream musl libc sources. The build script includes this directory in the include path before the musl headers, so these custom definitions take precedence.
