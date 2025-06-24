#include <string.h>
#include <stdint.h>

void *memcpy(void *restrict dest, const void *restrict src, size_t n)
{
#if defined(__wasm_bulk_memory__)
	if (n > BULK_MEMORY_THRESHOLD)
	  return __builtin_memcpy(dest, src, n);
#endif
	unsigned char *d = dest;
	const unsigned char *s = src;

	for (; n; n--) *d++ = *s++;
	return dest;
}
