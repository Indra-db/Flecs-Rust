#ifndef NULL
#define NULL ((void*)0)
#endif

typedef __SIZE_TYPE__ size_t;

_Noreturn void abort (void);

int atoi (const char *);
long atol (const char *);
long long atoll (const char *);
double atof (const char *);

double strtod (const char *__restrict, char **__restrict);

long strtol (const char *__restrict, char **_restrict, int);
long long strtoll (const char *__restrict, char **__restrict, int);
unsigned long long strtoull (const char *__restrict, char **__restrict, int);

void *malloc (size_t);
void *calloc (size_t, size_t);
void *realloc (void *, size_t);
void free (void *);

#define RAND_MAX (0x7fffffff)
int rand (void);
