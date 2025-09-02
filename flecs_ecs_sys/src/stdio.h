/* Minimal fallback stdio.h for WASM when wasi-libc is not available */
#ifndef _STDIO_H
#define _STDIO_H

#include <stdarg.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FILE FILE;

/* Standard streams (stubbed for WASM) */
extern FILE *stdin;
extern FILE *stdout; 
extern FILE *stderr;

/* File operations (all stubbed for WASM) */
FILE *fopen(const char *filename, const char *mode);
int fclose(FILE *stream);
size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream);
size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream);
int fseek(FILE *stream, long offset, int whence);
long ftell(FILE *stream);
int fflush(FILE *stream);

/* Printf family */
int printf(const char *format, ...);
int fprintf(FILE *stream, const char *format, ...);
int sprintf(char *str, const char *format, ...);
int snprintf(char *str, size_t size, const char *format, ...);
int vprintf(const char *format, va_list ap);
int vfprintf(FILE *stream, const char *format, va_list ap);
int vsprintf(char *str, const char *format, va_list ap);
int vsnprintf(char *str, size_t size, const char *format, va_list ap);

/* Character I/O */
int putchar(int c);
int puts(const char *s);
int fputs(const char *s, FILE *stream);

/* Seek constants */
#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

/* EOF */
#define EOF (-1)

#ifdef __cplusplus
}
#endif

#endif /* _STDIO_H */
