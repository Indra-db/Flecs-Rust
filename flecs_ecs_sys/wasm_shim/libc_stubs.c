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
__attribute__((weak)) int backtrace(void **buffer, int size) { (void)buffer; (void)size; return 0; }
__attribute__((weak)) char **backtrace_symbols(void *const *buffer, int size) { (void)buffer; (void)size; return 0; }
__attribute__((weak)) void backtrace_symbols_fd(void *const *buffer, int size, int fd) { (void)buffer; (void)size; (void)fd; }

__attribute__((weak)) void abort(void) { __builtin_trap(); }
__attribute__((weak)) long long atoll(const char *s) {
    long long v = 0; int neg = 0; if (!s) return 0; if (*s=='-'){neg=1; s++;}
    while (*s>='0' && *s<='9') { v = v*10 + (*s - '0'); s++; }
    return neg? -v : v;
}
__attribute__((weak)) int strncmp(const char *a, const char *b, size_t n) {
    for (; n; a++, b++, n--) {
        unsigned char ac = (unsigned char)*a, bc = (unsigned char)*b;
        if (ac != bc || !ac || !bc) return ac - bc;
    }
    return 0;
}
__attribute__((weak)) char *strchr(const char *s, int c) {
    char ch = (char)c;
    while (*s) { if (*s == ch) return (char*)s; s++; }
    return c==0 ? (char*)s : NULL;
}
__attribute__((weak)) char *strrchr(const char *s, int c) {
    const char *last = NULL; char ch = (char)c; while(*s){ if(*s==ch) last=s; s++; } return (char*)(c==0? s : last);
}
__attribute__((weak)) char *strstr(const char *h, const char *n) {
    if(!*n) return (char*)h; size_t nl=strlen(n); for(; *h; h++){ if(*h==*n && !strncmp(h,n,nl)) return (char*)h; } return 0;
}
/* rewritten to avoid empty-body warning */
__attribute__((weak)) char *strcpy(char *d, const char *s) {
    char *r = d;
    while (*s) { *d++ = *s++; }
    *d = '\0';
    return r;
}
__attribute__((weak)) char *strcat(char *d, const char *s) {
    char *r = d;
    while (*d) d++;
    while (*s) { *d++ = *s++; }
    *d = '\0';
    return r;
}
__attribute__((weak)) void *memcpy(void *d, const void *s, size_t n) {
    unsigned char *dd=d; const unsigned char *ss=s; while(n--) *dd++=*ss++; return d;
}
__attribute__((weak)) void *memmove(void *d, const void *s, size_t n) {
    unsigned char *dd=d; const unsigned char *ss=s; if (dd<ss) { while(n--) *dd++=*ss++; } else { dd+=n; ss+=n; while(n--) *--dd=*--ss; } return d;
}
__attribute__((weak)) void *memset(void *d, int c, size_t n) { unsigned char *p=d; while(n--) *p++=(unsigned char)c; return d; }
__attribute__((weak)) int memcmp(const void *a, const void *b, size_t n) { const unsigned char *aa=a,*bb=b; while(n--) { if(*aa!=*bb) return *aa-*bb; aa++; bb++; } return 0; }
__attribute__((weak)) size_t strlen(const char *s) { const char *p=s; while(*p) p++; return (size_t)(p-s); }
__attribute__((weak)) int strcmp(const char *a, const char *b) { while(*a && *a==*b){a++;b++;} return (unsigned char)*a - (unsigned char)*b; }
__attribute__((weak)) char *strerror(int errnum) { (void)errnum; return (char*)""; }

/* time stub */
__attribute__((weak)) time_t time(time_t *t) { static time_t fake=1; fake++; if(t) *t=fake; return fake; }

/* stdio stubs */
struct FILE { int _unused; };
static struct FILE dummy_stdout, dummy_stderr;
FILE *stdout = &dummy_stdout;
FILE *stderr = &dummy_stderr;
__attribute__((weak)) int printf(const char *fmt, ...) { (void)fmt; return 0; }
__attribute__((weak)) int fprintf(FILE *stream, const char *fmt, ...) { (void)stream; (void)fmt; return 0; }
__attribute__((weak)) int fputs(const char *s, FILE *stream) { (void)s; (void)stream; return 0; }
__attribute__((weak)) int putchar(int c) { return c; }
__attribute__((weak)) int vsnprintf(char *str, size_t size, const char *fmt, va_list ap) { (void)str;(void)size;(void)fmt;(void)ap; return 0; }
__attribute__((weak)) int snprintf(char *str, size_t size, const char *fmt, ...) { (void)str;(void)size;(void)fmt; return 0; }
__attribute__((weak)) int fclose(FILE *stream) { (void)stream; return 0; }
__attribute__((weak)) size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream) { (void)ptr;(void)size;(void)nmemb;(void)stream; return 0; }
__attribute__((weak)) int fseek(FILE *stream, long offset, int whence) { (void)stream;(void)offset;(void)whence; return 0; }
__attribute__((weak)) long ftell(FILE *stream) { (void)stream; return 0; }
__attribute__((weak)) int fopen_s(FILE **result, const char *filename, const char *mode) { (void)filename;(void)mode; if(result) *result=NULL; errno=EINVAL; return errno; }

/* simple pseudo-random generator */
static unsigned long __flecs_wasm_rand_state = 2463534242UL;
__attribute__((weak)) int rand(void) {
    /* xorshift32 */
    unsigned long x = __flecs_wasm_rand_state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    __flecs_wasm_rand_state = x;
    return (int)(x & 0x7fffffff);
}
