/* SPDX-License-Identifier: AGPL-3.0-or-later */
static VariableValue number(int type, uint32_t bits)
{
    VariableValue value = {0};
    value.type = type;
    memcpy(&value.u, &bits, 4);
    return value;
}

static uint32_t raw(VariableValue value)
{
    uint32_t bits;
    memcpy(&bits, &value.u, 4);
    return bits;
}

static void sample(int lt, uint32_t lb, int rt, uint32_t rb)
{
    void (*operators[])(VariableValue *, VariableValue *) = {
        Scr_EvalPlus, Scr_EvalMinus, Scr_EvalMultiply, Scr_EvalOr, Scr_EvalAnd,
        Scr_EvalExOr, Scr_EvalMod, Scr_EvalShiftLeft, Scr_EvalShiftRight
    };
    for (unsigned int op = 0; op < sizeof(operators) / sizeof(operators[0]); ++op) {
        VariableValue left = number(lt, lb), right = number(rt, rb);
        if (op == 6 && lt == VAR_INTEGER && rt == VAR_INTEGER &&
            left.u.intValue == INT_MIN && right.u.intValue == -1) continue;
        int original = lt != VAR_INTEGER || rt != VAR_INTEGER ||
                       c_defined(op, left.u.intValue, right.u.intValue);
        script_error = 0;
        if (original) operators[op](&left, &right);
        else {
            uint32_t bits = machine(op, lb, rb);
            memcpy(&left.u, &bits, 4);
        }
        printf("%c %u %c %08" PRIx32 " %c %08" PRIx32 " %d ",
               original ? 'C' : 'M', op, lt == VAR_INTEGER ? 'I' : 'F', lb,
               rt == VAR_INTEGER ? 'I' : 'F', rb, script_error);
        if (script_error) puts("~ ~");
        else {
            if (left.type != VAR_INTEGER && left.type != VAR_FLOAT) abort();
            printf("%c ", left.type == VAR_INTEGER ? 'I' : 'F');
            if (left.type == VAR_FLOAT && isnan(left.u.floatValue)) puts("nan");
            else printf("%08" PRIx32 "\n", raw(left));
        }
    }
}

static uint32_t next(uint32_t *state)
{
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    return *state;
}

int main(void)
{
    _Static_assert(CHAR_BIT == 8 && sizeof(int) == 4 && sizeof(float) == 4, "Scalar shapes");
    _Static_assert(FLT_RADIX == 2 && FLT_MANT_DIG == 24 && FLT_MAX_EXP == 128 && FLT_EVAL_METHOD == 0,
                   "Binary32 evaluation");
    _mm_setcsr(0x1f80);
    const uint32_t integers[] = {
        0, 1, 0xffffffff, 2, 0xfffffffe, 0x7fffffff, 0x80000000, 0x00ffffff,
        0x01000000, 0x01000001, 31, 32, 33, 0xffffffe0, 0x40000000, 0xc0000000
    };
    const uint32_t floats[] = {
        0, 0x80000000, 1, 0x80000001, 0x007fffff, 0x00800000, 0x3f800000, 0xbf800000,
        0x3f000000, 0x3fc00000, 0x7f7fffff, 0x7f800000, 0xff800000, 0x7fc00001, 0x7f800001, 0x4f000000
    };
    const size_t ni = sizeof(integers) / sizeof(integers[0]);
    const size_t nf = sizeof(floats) / sizeof(floats[0]);
    for (size_t i = 0; i < ni + nf; ++i) {
        for (size_t j = 0; j < ni + nf; ++j) {
            sample(i < ni ? VAR_INTEGER : VAR_FLOAT, i < ni ? integers[i] : floats[i - ni],
                   j < ni ? VAR_INTEGER : VAR_FLOAT, j < ni ? integers[j] : floats[j - ni]);
        }
    }
    uint32_t state = 0x95f24a1;
    for (unsigned int i = 0; i < 256; ++i) {
        uint32_t left = next(&state), right = next(&state);
        sample(i & 1 ? VAR_INTEGER : VAR_FLOAT, left, i & 2 ? VAR_INTEGER : VAR_FLOAT, right);
    }
    return 0;
}
