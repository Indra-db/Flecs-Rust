typedef struct FILE FILE;

int printf(const char *__restrict, ...);
int vsprintf(char *restrict, const char *restrict, va_list);
int vsnprintf(char *restrict, size_t, const char *restrict, va_list);
