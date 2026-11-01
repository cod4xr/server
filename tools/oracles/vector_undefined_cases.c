/* SPDX-License-Identifier: AGPL-3.0-or-later */
static VariableValue undefined_operand(Operand input)
{
    if (input.kind == 'U') return (VariableValue){.type = VAR_UNDEFINED};
    return operand(input);
}

static void sample_undefined(unsigned int operation, Operand a, Operand b)
{
    static void (*const functions[])(VariableValue *, VariableValue *) = {
        Scr_EvalEquality, Scr_EvalInequality, Scr_EvalLess, Scr_EvalGreater,
        Scr_EvalLessEqual, Scr_EvalGreaterEqual
    };
    vector_used = 0;
    VariableValue left = undefined_operand(a), right = undefined_operand(b);
    printf("U %u", operation); print_operand(a); print_operand(b);
    script_error = 0;
    if (setjmp(error_target) == 0) functions[operation](&left, &right);
    if (script_error) printf(" E%d\n", script_error);
    else {
        if (left.type != VAR_INTEGER || (left.u.intValue != 0 && left.u.intValue != 1)) abort();
        printf(" ok %d\n", left.u.intValue);
    }
}

int main(void)
{
    _Static_assert(CHAR_BIT == 8 && sizeof(int) == 4 && sizeof(float) == 4, "Source domains");
    _mm_setcsr(0x1f80);
    const uint32_t integers[] = {0, 1, 2, 0xffffffff, 0x7fffffff, 0x80000000, 0x01000001};
    const uint32_t floats[] = {0, 0x80000000, 1, 0x007fffff, 0x00800000, 0x3f800000,
                             0xbf800000, 0x3f800001, 0x7f7fffff, 0x7f800000,
                             0xff800000, 0x7fc00001, 0x7f800001};
    Operand undefined = {'U', {0, 0, 0}};
    for (unsigned int op = 0; op < 6; ++op) {
        sample_undefined(op, undefined, undefined);
        for (unsigned int i = 0; i < sizeof(integers)/sizeof(integers[0]); ++i) {
            Operand value = {'I', {integers[i], 0, 0}};
            sample_undefined(op, undefined, value); sample_undefined(op, value, undefined);
        }
        const unsigned int count = sizeof(floats)/sizeof(floats[0]);
        for (unsigned int i = 0; i < count; ++i) {
            Operand value = {'F', {floats[i], 0, 0}};
            sample_undefined(op, undefined, value); sample_undefined(op, value, undefined);
            value = (Operand){'V', {floats[i], floats[(i+1)%count], floats[(i+5)%count]}};
            sample_undefined(op, undefined, value); sample_undefined(op, value, undefined);
        }
    }
    return 0;
}
