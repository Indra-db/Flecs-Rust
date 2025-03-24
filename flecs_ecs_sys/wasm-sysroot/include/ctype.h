#ifndef	_CTYPE_H
#define	_CTYPE_H

#ifdef __cplusplus
extern "C" {
#endif

static __inline int __isspace(int _c)
{
        return _c == ' ' || (unsigned)_c-'\t' < 5;
}

#define isalpha(a) ((((unsigned)(a)|32) - 'a') < 26)
#define isdigit(a) (((unsigned)(a) - '0') < 10)
#define islower(a) (((unsigned)(a) - 'a') < 26)
#define isupper(a) (((unsigned)(a) - 'A') < 26)
#define isprint(a) (((unsigned)(a) - 0x20) < 0x5f)
#define isgraph(a) (((unsigned)(a) - 0x21) < 0x5e)
#define isspace(a) __isspace(a)

int   tolower(int);
int   toupper(int);

#ifdef __cplusplus
}
#endif

#endif
