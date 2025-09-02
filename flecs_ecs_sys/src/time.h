/* Minimal fallback time.h for WASM when wasi-libc is not available */
#ifndef _TIME_H
#define _TIME_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef long time_t;
typedef long clock_t;

struct tm {
    int tm_sec;    /* seconds (0-60) */
    int tm_min;    /* minutes (0-59) */
    int tm_hour;   /* hours (0-23) */
    int tm_mday;   /* day of month (1-31) */
    int tm_mon;    /* month (0-11) */
    int tm_year;   /* year - 1900 */
    int tm_wday;   /* day of week (0-6, Sunday = 0) */
    int tm_yday;   /* day of year (0-365) */
    int tm_isdst;  /* daylight saving time */
};

/* Time functions (stubbed for WASM) */
time_t time(time_t *tloc);
clock_t clock(void);
struct tm *localtime(const time_t *timep);
struct tm *gmtime(const time_t *timep);
char *asctime(const struct tm *tm);
char *ctime(const time_t *timep);
size_t strftime(char *s, size_t max, const char *format, const struct tm *tm);

#define CLOCKS_PER_SEC 1000000L

#ifdef __cplusplus
}
#endif

#endif /* _TIME_H */
