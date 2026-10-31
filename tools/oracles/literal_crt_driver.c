// SPDX-License-Identifier: AGPL-3.0-or-later
// Replay the source action's CRT calls in a defined locale/rounding environment.
// Inputs are preselected finite/in-range ASCII lexemes, not arbitrary source code.
#include <fenv.h>
#include <float.h>
#include <limits.h>
#include <locale.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

_Static_assert(sizeof(int) == 4 && INT_MAX == 2147483647, "Oracle requires int32");
_Static_assert(sizeof(float) == 4 && FLT_RADIX == 2 && FLT_MANT_DIG == 24 && FLT_MAX_EXP == 128,
               "Oracle requires binary32");

int main(void) {
    if (!setlocale(LC_ALL, "C") || fesetround(FE_TONEAREST)) return 1;
    char kind;
    char text[256];
    while (scanf(" %c %255s", &kind, text) == 2) {
        uint32_t bits;
        if (kind == 'I') {
            int value;
            if (sscanf(text, "%d", &value) != 1) return 2;
            memcpy(&bits, &value, 4);
        } else if (kind == 'F') {
            float value;
            if (sscanf(text, "%f", &value) != 1) return 3;
            memcpy(&bits, &value, 4);
        } else {
            return 4;
        }
        printf("N %c %s %08x\n", kind, text, bits);
    }
    return 0;
}
