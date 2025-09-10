// WASI libc header wrapper for bindgen - including core musl headers
// This wrapper includes the essential musl headers to let the type system work properly

// Include basic type definition headers first
#include "libc-top-half/musl/include/stddef.h"
#include "libc-top-half/musl/include/stdint.h"
#include "libc-top-half/musl/include/stdbool.h"

// Include core headers with the __NEED_* system
#include "libc-top-half/musl/include/wchar.h"    // This sets up __NEED_wchar_t and includes bits/alltypes.h
#include "libc-top-half/musl/include/stdio.h"
#include "libc-top-half/musl/include/stdlib.h"
#include "libc-top-half/musl/include/string.h"
#include "libc-top-half/musl/include/math.h"

// Include limits and float constants
#include "libc-top-half/musl/include/limits.h"
#include "libc-top-half/musl/include/float.h"
