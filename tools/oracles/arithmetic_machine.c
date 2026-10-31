/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Isolated instruction semantics, not execution of original full assembly routines. */
#if !defined(__i386__) && !defined(__x86_64__)
#error Integer machine profile requires x86
#endif

static uint32_t machine(unsigned int operation, uint32_t left, uint32_t right)
{
    switch (operation) {
        case 0: __asm__ volatile ("addl %1, %0" : "+r"(left) : "r"(right) : "cc"); break;
        case 1: __asm__ volatile ("subl %1, %0" : "+r"(left) : "r"(right) : "cc"); break;
        case 2: __asm__ volatile ("imull %1, %0" : "+r"(left) : "r"(right) : "cc"); break;
        case 7: __asm__ volatile ("shll %%cl, %0" : "+r"(left) : "c"(right) : "cc"); break;
        case 8: __asm__ volatile ("sarl %%cl, %0" : "+r"(left) : "c"(right) : "cc"); break;
        default: abort();
    }
    return left;
}

static int c_defined(unsigned int operation, int left, int right)
{
    int64_t wide;
    switch (operation) {
        case 0: wide = (int64_t)left + right; break;
        case 1: wide = (int64_t)left - right; break;
        case 2: wide = (int64_t)left * right; break;
        case 7: return left >= 0 && right >= 0 && right < 32 &&
                       ((uint64_t)left << right) <= INT_MAX;
        case 8: return left >= 0 && right >= 0 && right < 32;
        default: return 1;
    }
    return wide >= INT_MIN && wide <= INT_MAX;
}
