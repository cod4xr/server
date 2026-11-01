/* SPDX-License-Identifier: AGPL-3.0-or-later */
static void sample_index(Operand input, Operand index_input)
{
    vector_used = 0;
    VariableValue value = operand(input), index = operand(index_input);
    printf("I"); print_vector(value.u.vectorValue, 0); print_operand(index_input);
    script_error = 0;
    if (setjmp(error_target) == 0) Scr_EvalArray(&value, &index);
    if (script_error) printf(" E%d\n", script_error);
    else {
        uint32_t bits;
        if (index.type != VAR_FLOAT) abort();
        memcpy(&bits, &index.u, sizeof(bits));
        printf(" ok %08" PRIx32 "\n", bits);
    }
}

int main(void)
{
    _Static_assert(CHAR_BIT == 8 && sizeof(int) == 4 && sizeof(float) == 4, "Source domains");
    _mm_setcsr(0x1f80);
    const uint32_t values[] = {0, 0x80000000, 1, 0x007fffff, 0x00800000, 0x3f800000,
                              0xbf800000, 0x3f800001, 0x7f7fffff, 0x7f800000,
                              0xff800000, 0x7fc00001, 0x7f800001};
    const uint32_t indices[] = {0, 1, 2, 3, 4, 0xffffffff, 0xfffffffe, 0x7fffffff, 0x80000000};
    const unsigned int count = sizeof(values)/sizeof(values[0]);
    for (unsigned int i = 0; i < count; ++i) {
        Operand input = {'V', {values[i], values[(i+1)%count], values[(i+5)%count]}};
        for (unsigned int j = 0; j < sizeof(indices)/sizeof(indices[0]); ++j)
            sample_index(input, (Operand){'I', {indices[j], 0, 0}});
        for (unsigned int j = 0; j < sizeof(values)/sizeof(values[0]); ++j)
            sample_index(input, (Operand){'F', {values[j], 0, 0}});
        sample_index(input, (Operand){'V', {0, 0, 0}});
    }
    return 0;
}
