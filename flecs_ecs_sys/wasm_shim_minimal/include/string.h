#ifndef FLECS_WASM_STRING_EXTENSIONS_H
#define FLECS_WASM_STRING_EXTENSIONS_H

#include_next <string.h>
#include <errno.h>

/* Provide Microsoft-style safe string functions for WASM */
#ifdef __cplusplus
extern "C" {
#endif

static inline int strcpy_s(char *dest, size_t destsz, const char *src) {
    if (!dest || !src || destsz == 0) {
        if (dest && destsz > 0) dest[0] = '\0';
        errno = EINVAL;
        return EINVAL;
    }
    
    size_t src_len = strlen(src);
    if (src_len >= destsz) {
        dest[0] = '\0';
        errno = ERANGE;
        return ERANGE;
    }
    
    strcpy(dest, src);
    return 0;
}

static inline int strcat_s(char *dest, size_t destsz, const char *src) {
    if (!dest || !src || destsz == 0) {
        errno = EINVAL;
        return EINVAL;
    }
    
    size_t dest_len = strlen(dest);
    size_t src_len = strlen(src);
    
    if (dest_len + src_len >= destsz) {
        errno = ERANGE;
        return ERANGE;
    }
    
    strcat(dest, src);
    return 0;
}

static inline int strncpy_s(char *dest, size_t destsz, const char *src, size_t count) {
    if (!dest || !src || destsz == 0) {
        if (dest && destsz > 0) dest[0] = '\0';
        errno = EINVAL;
        return EINVAL;
    }
    
    size_t copy_len = (count < destsz - 1) ? count : destsz - 1;
    strncpy(dest, src, copy_len);
    dest[copy_len] = '\0';
    return 0;
}

static inline int fopen_s(FILE **result, const char *filename, const char *mode) {
    if (!result) {
        errno = EINVAL;
        return EINVAL;
    }
    
    *result = fopen(filename, mode);
    if (!*result) {
        return errno;
    }
    return 0;
}

#ifdef __cplusplus
}
#endif

#endif
