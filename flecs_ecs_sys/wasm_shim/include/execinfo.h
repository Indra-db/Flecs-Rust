#ifndef FLECS_WASM_EXECINFO_H
#define FLECS_WASM_EXECINFO_H
int backtrace(void **buffer, int size);
char **backtrace_symbols(void *const *buffer, int size);
void backtrace_symbols_fd(void *const *buffer, int size, int fd);
#endif
