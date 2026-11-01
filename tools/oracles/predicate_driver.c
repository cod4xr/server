// SPDX-License-Identifier: AGPL-3.0-or-later
// Actual extracted functions; services model supplied count/type and fixed result/error.
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static unsigned int parameter_count;
static unsigned int parameter_type;
static unsigned int type_reads;
static unsigned int result_writes;
static int result_value;
static int failed;

static unsigned int Scr_GetNumParam(void) { return parameter_count; }
static unsigned int Scr_GetType(unsigned int index)
{
    if (index != 0 || parameter_count != 1 || ++type_reads != 1) abort();
    return parameter_type;
}
static void Scr_AddBool(int value)
{
    if (++result_writes != 1 || failed || (value != 0 && value != 1)) abort();
    result_value = value;
}
static void Scr_Error(const char *message)
{
    if (!message || failed || result_writes || type_reads) abort();
    failed = 1;
}

// Extracted original functions are inserted here by the generator.
/* ORIGINAL_FUNCTIONS */

int main(void)
{
    void (*const functions[])(void) = {GScr_IsInt, GScr_IsFloat, GScr_IsVector};
    const unsigned int counts[] = {0, 1, 2, 6, 255, 256, 70000, UINT32_MAX};
    for (unsigned int function = 0; function < 3; ++function) {
        for (unsigned int type = 0; type <= VAR_ENDON_LIST; ++type) {
            for (unsigned int count = 0; count < sizeof(counts)/sizeof(counts[0]); ++count) {
                parameter_count = counts[count];
                parameter_type = type;
                type_reads = result_writes = 0;
                result_value = failed = 0;
                functions[function]();
                if (failed) {
                    if (result_writes || type_reads) abort();
                    printf("P %u %u %u E1\n", function, type, parameter_count);
                } else {
                    if (result_writes != 1 || type_reads != 1) abort();
                    printf("P %u %u %u ok %d\n", function, type, parameter_count, result_value);
                }
            }
        }
    }
    return 0;
}
