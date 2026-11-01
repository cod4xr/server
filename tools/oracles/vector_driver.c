/* SPDX-License-Identifier: AGPL-3.0-or-later */
static void sample_pair(Operand li, Operand ri)
{
    vector_used = 0;
    VariableValue left = operand(li), right = operand(ri);
    Scr_CastWeakerPair(&left, &right);
    if (left.type != VAR_VECTOR || right.type != VAR_VECTOR) abort();
    printf("P"); print_operand(li); print_operand(ri);
    print_vector(left.u.vectorValue, 0); print_vector(right.u.vectorValue, 0); puts("");
}

static void sample_arithmetic(unsigned int operation, Operand li, Operand ri)
{
    void (*functions[])(VariableValue *, VariableValue *) = {
        Scr_EvalPlus, Scr_EvalMinus, Scr_EvalMultiply, Scr_EvalDivide
    };
    if (operation >= 4) abort();
    vector_used = 0;
    VariableValue left = operand(li), right = operand(ri);
    script_error = 0;
    if (setjmp(error_target) == 0) functions[operation](&left, &right);
    printf("A %u", operation); print_operand(li); print_operand(ri);
    if (script_error) printf(" E%d\n", script_error);
    else {
        if (left.type != VAR_VECTOR) abort();
        printf(" ok"); print_vector(left.u.vectorValue, 1); puts("");
    }
}

static void sample_compare(unsigned int operation, Operand li, Operand ri)
{
    void (*functions[])(VariableValue *, VariableValue *) = {
        Scr_EvalEquality, Scr_EvalInequality, Scr_EvalLess, Scr_EvalGreater,
        Scr_EvalLessEqual, Scr_EvalGreaterEqual
    };
    if (operation >= 6) abort();
    vector_used = 0;
    VariableValue left = operand(li), right = operand(ri);
    script_error = 0;
    if (setjmp(error_target) == 0) functions[operation](&left, &right);
    printf("C %u", operation); print_operand(li); print_operand(ri);
    if (script_error) printf(" E%d\n", script_error);
    else {
        if (left.type != VAR_INTEGER || (left.u.intValue != 0 && left.u.intValue != 1)) abort();
        printf(" ok %d\n", left.u.intValue);
    }
}

static void sample(Operand left, Operand right)
{
    sample_pair(left, right);
    for (unsigned int op = 0; op < 4; ++op) sample_arithmetic(op, left, right);
    for (unsigned int op = 0; op < 6; ++op) sample_compare(op, left, right);
}

static void sample_cast(Operand x, Operand y, Operand z)
{
    vector_used = 0;
    /* Coordinate-order fixture arguments; the actual helper consumes reversed slots. */
    VariableValue values[3] = {operand(z), operand(y), operand(x)};
    Scr_CastVector(values);
    if (values[0].type != VAR_VECTOR) abort();
    printf("V"); print_operand(x); print_operand(y); print_operand(z);
    print_vector(values[0].u.vectorValue, 0); puts("");
}
