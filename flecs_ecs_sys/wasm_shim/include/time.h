#ifndef FLECS_WASM_TIME_H
#define FLECS_WASM_TIME_H
#include <stdint.h>
#include <stddef.h>
struct timespec { long tv_sec; long tv_nsec; };
typedef long time_t;
 time_t time(time_t *t);
#endif
