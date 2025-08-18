#ifndef FLECS_WASM_ALLOCA_H
#define FLECS_WASM_ALLOCA_H
#include <stddef.h>
#define alloca(size) __builtin_alloca(size)
#endif
