// WASI libc header wrapper for bindgen - using musl libc top half only
// This wrapper includes only the musl headers to generate proper bindings

// Include musl headers that define the types we need through __NEED_* macros
#include "libc-top-half/musl/include/wchar.h"    // Defines __NEED_wchar_t, __NEED_size_t, etc.

// Core musl headers for the functions we want to bind (using musl paths only)
#include "libc-top-half/musl/include/stdlib.h"
#include "libc-top-half/musl/include/string.h" 
#include "libc-top-half/musl/include/stdio.h"
#include "libc-top-half/musl/include/limits.h"
#include "libc-top-half/musl/include/float.h"
