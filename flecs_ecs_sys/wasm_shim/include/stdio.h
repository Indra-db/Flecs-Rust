#ifndef FLECS_WASM_STDIO_H
#define FLECS_WASM_STDIO_H
#include <stdarg.h>
#include <stddef.h>

typedef struct FILE FILE; /* opaque */

extern FILE *stdout;
extern FILE *stderr;

int printf(const char *fmt, ...);
int fprintf(FILE *stream, const char *fmt, ...);
int snprintf(char *str, size_t size, const char *fmt, ...);
int vsnprintf(char *str, size_t size, const char *fmt, va_list ap);
int fputs(const char *s, FILE *stream);
int putchar(int c);
/* File operations (stubbed) */
int fclose(FILE *stream);
size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream);
int fseek(FILE *stream, long offset, int whence);
long ftell(FILE *stream);
#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2
/* Map fopen_s macro used by flecs */
int fopen_s(FILE **result, const char *filename, const char *mode);
#endif
