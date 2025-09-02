// WASI libc header wrapper for bindgen - simplified approach
// Using system headers instead of complex musl setup

// Use system C headers for basic functionality
#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <limits.h>
#include <float.h>
#include <math.h>
#include <time.h>
#include <errno.h>
#include <ctype.h>
#include <assert.h>
#include <setjmp.h>
#include <stdarg.h>

// Define NULL if not already defined
#ifndef NULL
#define NULL ((void*)0)
#endif

// Define EOF if not already defined  
#ifndef EOF
#define EOF (-1)
#endif

// File seek constants
#ifndef SEEK_SET
#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2
#endif
