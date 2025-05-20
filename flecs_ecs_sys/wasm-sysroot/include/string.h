#include <stddef.h>

void *memchr(const void *s, int c, size_t n);
void *memcpy (void *restrict, const void *restrict, size_t);
void *memmove (void *, const void *, size_t);
void *memset (void *, int, size_t);
int memcmp (const void *, const void *, size_t);

char *strcat (char *restrict, const char *restrict);

char *strcpy (char *restrict, const char *restrict);
char *strncpy (char *__restrict, const char *__restrict, size_t);

char *strchr (const char *, int);
char *strrchr (const char *, int);

int strcmp (const char *, const char *);
int strncmp (const char *, const char *, size_t);

char *strstr (const char *, const char *);

extern size_t strlen(const char *s);

char *strerror (int);
