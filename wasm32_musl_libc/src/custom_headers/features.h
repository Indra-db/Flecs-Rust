#ifndef _WASM_FEATURES_H
#define _WASM_FEATURES_H

// Include the standard features.h first
#include_next <features.h>

// Then include the internal musl features.h for definitions like weak_alias
#include "../libc-top-half/musl/src/include/features.h"

#endif
