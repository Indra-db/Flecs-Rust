#include <bits/alltypes.h>
#include <features.h>

typedef struct FILE FILE;

extern FILE *const stdout;

int printf(const char *__restrict, ...);
int snprintf(char *__restrict, size_t, const char *__restrict, ...);
int vfprintf(FILE *__restrict, const char *__restrict, va_list);
int vsprintf(char *restrict, const char *restrict, va_list);
int vsnprintf(char *restrict, size_t, const char *restrict, va_list);
