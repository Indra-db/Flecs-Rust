#ifndef FLECS_WASM_STRING_H
#define FLECS_WASM_STRING_H
#include <stddef.h>
#include <limits.h>
void *memcpy(void *restrict, const void *restrict, size_t);
void *memmove(void *, const void *, size_t);
void *memset(void *, int, size_t);
int memcmp(const void *, const void *, size_t);
size_t strlen(const char *);
int strcmp(const char *, const char *);
int strncmp(const char *, const char *, size_t);
char *strchr(const char *, int);
char *strrchr(const char *, int);
char *strstr(const char *, const char *);
char *strcpy(char *restrict, const char *restrict);
char *strncpy(char *restrict, const char *restrict, size_t);
char *strcat(char *restrict, const char *restrict);
char *strerror(int);
#define strcpy_s(dst, cap, src) (strcpy((dst),(src)),0)
#define strcat_s(dst, cap, src) (strcat((dst),(src)),0)
#define sprintf_s(buf, cap, ...) (snprintf((buf),(cap), __VA_ARGS__))
/* Map strncpy_s used by flecs to a best-effort implementation.
	Note: this ignores the destination capacity parameter and returns 0 on success,
	which is sufficient for our use in wasm shim. */
#define strncpy_s(dst, cap, src, n) ( (void)(cap), strncpy((dst),(src),(n)), 0 )
#endif
