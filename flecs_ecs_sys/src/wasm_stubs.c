// Minimal stubs for functions not provided by wasi-libc
// These are weak symbols so they can be overridden if needed

#include <stddef.h>

// execinfo.h functions (backtrace support) - not available in wasi-libc
__attribute__((weak)) int backtrace(void **buffer, int size) { 
    (void)buffer; (void)size; 
    return 0; 
}

__attribute__((weak)) char **backtrace_symbols(void *const *buffer, int size) { 
    (void)buffer; (void)size; 
    return 0; 
}

__attribute__((weak)) void backtrace_symbols_fd(void *const *buffer, int size, int fd) { 
    (void)buffer; (void)size; (void)fd; 
}
