#ifndef _UNISTD_H
#define _UNISTD_H
/* Minimal unistd.h stub for wasm32-unknown-unknown freestanding build */
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif

typedef int pid_t;
typedef long ssize_t;
#ifndef SEEK_SET
#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2
#endif

static inline unsigned sleep(unsigned seconds) { (void)seconds; return 0; }
static inline int usleep(unsigned int usec) { (void)usec; return 0; }
static inline pid_t getpid(void) { return 1; }
static inline int close(int fd) { (void)fd; return 0; }
static inline ssize_t read(int fd, void *buf, size_t count) { (void)fd; (void)buf; (void)count; return 0; }
static inline ssize_t write(int fd, const void *buf, size_t count) { (void)fd; (void)buf; return (ssize_t)count; }
static inline int isatty(int fd) { (void)fd; return 0; }
static inline int unlink(const char *path) { (void)path; return 0; }
static inline void _exit(int status) { (void)status; for(;;) { __builtin_trap(); } }
static inline int access(const char *path, int mode) { (void)path; (void)mode; return 0; }

#ifdef __cplusplus
}
#endif
#endif /* _UNISTD_H */
