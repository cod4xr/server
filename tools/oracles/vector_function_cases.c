/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Synthetic 32-bit identity labels, never native code allocation/invocation. */
static VariableValue function_operand(Operand input)
{
    if (input.kind == 'U') return (VariableValue){.type = VAR_UNDEFINED};
    if (input.kind != 'R') return operand(input);
    VariableValue result = {.type = VAR_FUNCTION};
    memcpy(&result.u, &input.bits[0], sizeof(uint32_t));
    return result;
}

static void sample_function(unsigned int operation, Operand a, Operand b)
{
    static void (*const functions[])(VariableValue *, VariableValue *) = {
        Scr_EvalEquality, Scr_EvalInequality, Scr_EvalLess, Scr_EvalGreater,
        Scr_EvalLessEqual, Scr_EvalGreaterEqual
    };
    vector_used = 0;
    VariableValue left = function_operand(a), right = function_operand(b);
    printf("R %u", operation); print_operand(a); print_operand(b);
    script_error = 0;
    if (setjmp(error_target) == 0) functions[operation](&left, &right);
    if (script_error) printf(" E%d\n", script_error);
    else {
        if (left.type != VAR_INTEGER || (left.u.intValue != 0 && left.u.intValue != 1)) abort();
        printf(" ok %d\n", left.u.intValue);
    }
}

static void both(unsigned int op, Operand reference, Operand other)
{
    sample_function(op, reference, other);
    sample_function(op, other, reference);
}

int main(void)
{
    _Static_assert(CHAR_BIT == 8 && sizeof(int) == 4 && sizeof(float) == 4, "Source domains");
    _mm_setcsr(0x1f80);
    const uint32_t references[] = {0, 1, 2, 0x7fffffff, 0x80000000, 0xffffffff};
    const uint32_t integers[] = {0, 1, 2, 0xffffffff, 0x7fffffff, 0x80000000, 0x01000001};
    const uint32_t floats[] = {0, 0x80000000, 1, 0x007fffff, 0x00800000, 0x3f800000,
                             0xbf800000, 0x3f800001, 0x7f7fffff, 0x7f800000,
                             0xff800000, 0x7fc00001, 0x7f800001};
    for (unsigned int op = 0; op < 6; ++op) {
        for (unsigned int i = 0; i < sizeof(references)/sizeof(references[0]); ++i) {
            Operand reference = {'R', {references[i], 0, 0}};
            for (unsigned int j = 0; j < sizeof(references)/sizeof(references[0]); ++j)
                sample_function(op, reference, (Operand){'R', {references[j], 0, 0}});
            both(op, reference, (Operand){'U', {0, 0, 0}});
            for (unsigned int j = 0; j < sizeof(integers)/sizeof(integers[0]); ++j)
                both(op, reference, (Operand){'I', {integers[j], 0, 0}});
            const unsigned int count = sizeof(floats)/sizeof(floats[0]);
            for (unsigned int j = 0; j < count; ++j) {
                both(op, reference, (Operand){'F', {floats[j], 0, 0}});
                both(op, reference, (Operand){'V', {floats[j], floats[(j+1)%count], floats[(j+5)%count]}});
            }
        }
    }
    return 0;
}
