#ifndef FLECS_WASM_STDLIB_H
#define FLECS_WASM_STDLIB_H
#include <stddef.h>
void *malloc(size_t);
void free(void*);
void *realloc(void*, size_t);
void *calloc(size_t, size_t);
int abs(int);
void abort(void);
long long atoll(const char*);
int rand(void);
#ifndef RAND_MAX
#define RAND_MAX 2147483647
#endif
#endif
