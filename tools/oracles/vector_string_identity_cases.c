/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Canonical synthetic string labels; no native interner, storage or ref cleanup. */
static VariableValue string_operand(Operand input)
{
    if (input.kind == 'U') return (VariableValue){.type = VAR_UNDEFINED};
    if (input.kind == 'R') return (VariableValue){.type = VAR_FUNCTION, .u.intValue = 1};
    if (input.kind != 'S' && input.kind != 'L') return operand(input);
    if (input.bits[0] < 1 || input.bits[0] > 6 || input.bits[1] || input.bits[2]) abort();
    VariableValue result = {.type = input.kind == 'S' ? VAR_STRING : VAR_ISTRING};
    result.u.stringValue = input.bits[0];
    return result;
}

static void sample_string(unsigned int operation, Operand a, Operand b)
{
    /* Plain/mixed numeric formatting is outside this identity-only service profile. */
    if (operation >= 6 && a.kind != 'L' && b.kind != 'L') return;
    static void (*const functions[])(VariableValue *, VariableValue *) = {
        Scr_EvalEquality, Scr_EvalInequality, Scr_EvalLess, Scr_EvalGreater,
        Scr_EvalLessEqual, Scr_EvalGreaterEqual, Scr_EvalPlus, Scr_EvalMinus,
        Scr_EvalMultiply, Scr_EvalDivide
    };
    vector_used = string_reference_removals = 0;
    VariableValue left = string_operand(a), right = string_operand(b);
    printf("S %u", operation); print_operand(a); print_operand(b);
    script_error = 0;
    if (setjmp(error_target) == 0) functions[operation](&left, &right);
    if (script_error) {
        if (string_reference_removals != 0) abort();
        printf(" E%d\n", script_error);
    } else {
        if (operation > 1 || a.kind != b.kind || string_reference_removals != 2) abort();
        if (removed_string_labels[0] != a.bits[0] || removed_string_labels[1] != b.bits[0]) abort();
        if (left.type != VAR_INTEGER || (left.u.intValue != 0 && left.u.intValue != 1)) abort();
        printf(" ok %d\n", left.u.intValue);
    }
}

int main(void)
{
    _Static_assert(CHAR_BIT == 8 && sizeof(int) == 4 && sizeof(float) == 4, "Source domains");
    _mm_setcsr(0x1f80);
    const Operand others[] = {
        {'U', {0, 0, 0}}, {'I', {0, 0, 0}}, {'I', {1, 0, 0}},
        {'F', {0, 0, 0}}, {'F', {0x3f800000, 0, 0}}, {'F', {0x7fc00001, 0, 0}},
        {'V', {0, 0, 0}}, {'V', {0x3f800000, 0x40000000, 0x40400000}}, {'R', {1, 0, 0}}
    };
    for (unsigned int operation = 0; operation < 10; ++operation) {
        for (unsigned int kind = 0; kind < 2; ++kind) {
            for (unsigned int label = 1; label <= 6; ++label) {
                Operand left = {kind ? 'L' : 'S', {label, 0, 0}};
                for (unsigned int other_kind = 0; other_kind < 2; ++other_kind) {
                    for (unsigned int other_label = 1; other_label <= 6; ++other_label) {
                        sample_string(operation, left, (Operand){other_kind ? 'L' : 'S', {other_label, 0, 0}});
                    }
                }
                for (unsigned int index = 0; index < sizeof(others)/sizeof(others[0]); ++index) {
                    sample_string(operation, left, others[index]);
                    sample_string(operation, others[index], left);
                }
            }
        }
    }
    return 0;
}
