#ifndef FLECS_WASM_MATH_H
#define FLECS_WASM_MATH_H
#define NAN (__builtin_nanf(""))
#define INFINITY (__builtin_inff())
static inline int isnan_f(float x){ return __builtin_isnan(x); }
static inline int isnan_d(double x){ return __builtin_isnan(x); }
static inline int isinf_f(float x){ return __builtin_isinf(x); }
static inline int isinf_d(double x){ return __builtin_isinf(x); }
#define isnan(x) __builtin_isnan(x)
#define isinf(x) __builtin_isinf(x)
#endif
