#include <stddef.h>
#include <stdarg.h>
#include <string.h>
#include <stdio.h>
#include <errno.h>
#include <execinfo.h>
#include <time.h>

/* Provide a dummy errno */
int errno;

/* backtrace stubs */
int backtrace(void **buffer, int size) { (void)buffer; (void)size; return 0; }
char **backtrace_symbols(void *const *buffer, int size) { (void)buffer; (void)size; return 0; }
void backtrace_symbols_fd(void *const *buffer, int size, int fd) { (void)buffer; (void)size; (void)fd; }
void abort(void) { __builtin_trap(); }
int printf(const char *fmt, ...) { (void)fmt; return 0; }
int fprintf(FILE *stream, const char *fmt, ...) { (void)stream; (void)fmt; return 0; }
int fputs(const char *s, FILE *stream) { (void)s; (void)stream; return 0; }
int putchar(int c) { return c; }
int vsnprintf(char *str, size_t size, const char *fmt, va_list ap) { (void)str;(void)size;(void)fmt;(void)ap; return 0; }
int snprintf(char *str, size_t size, const char *fmt, ...) { (void)str;(void)size;(void)fmt; return 0; }
int fclose(FILE *stream) { (void)stream; return 0; }
size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream) { (void)ptr;(void)size;(void)nmemb;(void)stream; return 0; }
int fseek(FILE *stream, long offset, int whence) { (void)stream;(void)offset;(void)whence; return 0; }
long ftell(FILE *stream) { (void)stream; return 0; }
int fopen_s(FILE **result, const char *filename, const char *mode) { (void)filename;(void)mode; if(result) *result=NULL; errno=EINVAL; return errno; }
int __clock_gettime(clockid_t clk, struct timespec *ts) { return -1; }