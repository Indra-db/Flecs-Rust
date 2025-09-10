#include <stdio.h>
#include <stdlib.h>

// Minimal WASM-compatible implementations for assert.c dependencies

// Dummy stderr definition - just use a static void pointer cast to FILE*
// static int __dummy_stderr;
// FILE *const stderr = (FILE*)&__dummy_stderr;

// // Minimal fprintf implementation that does nothing but satisfies the linker
// int fprintf(FILE *stream, const char *format, ...) {
//     // In WASM, we can't really print to stderr, so just return 0
//     (void)stream;
//     (void)format;
//     return 0;
// }

// // WASM-compatible abort implementation
void abort(void) {
    // wasm doesn't support signals, so just trap to halt the program.
    __builtin_trap();
}
