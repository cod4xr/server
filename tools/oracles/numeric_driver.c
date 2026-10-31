/* SPDX-License-Identifier: AGPL-3.0-or-later */
static VariableValue number(int type, uint32_t bits)
{
    if (type != VAR_INTEGER && type != VAR_FLOAT) abort();
    VariableValue value;
    memset(&value, 0, sizeof(value));
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

static char kind(VariableValue value)
{
    if (value.type == VAR_INTEGER) return 'I';
    if (value.type == VAR_FLOAT) return 'F';
    abort();
}

static void sample(int left_type, uint32_t left_bits, int right_type, uint32_t right_bits)
{
    VariableValue left = number(left_type, left_bits);
    VariableValue right = number(right_type, right_bits);
    char lk = kind(left), rk = kind(right);
    Scr_CastWeakerPair(&left, &right);
    printf("P %c %08" PRIx32 " %c %08" PRIx32 " %c %08" PRIx32 " %c %08" PRIx32 "\n",
           lk, left_bits, rk, right_bits, kind(left), raw(left), kind(right), raw(right));
    void (*comparisons[])(VariableValue *, VariableValue *) = {
        Scr_EvalEquality, Scr_EvalInequality, Scr_EvalLess, Scr_EvalGreater,
        Scr_EvalLessEqual, Scr_EvalGreaterEqual
    };
    for (unsigned int operation = 0; operation < sizeof(comparisons) / sizeof(comparisons[0]); ++operation) {
        left = number(left_type, left_bits);
        right = number(right_type, right_bits);
        script_error = 0;
        comparisons[operation](&left, &right);
        if (script_error || left.type != VAR_INTEGER || (left.u.intValue != 0 && left.u.intValue != 1)) abort();
        printf("C %u %c %08" PRIx32 " %c %08" PRIx32 " %d\n",
               operation, lk, left_bits, rk, right_bits, left.u.intValue);
    }
    left = number(left_type, left_bits);
    right = number(right_type, right_bits);
    script_error = 0;
    Scr_EvalDivide(&left, &right);
    if (left.type != VAR_FLOAT) abort();
    printf("D %c %08" PRIx32 " %c %08" PRIx32 " %d ", lk, left_bits, rk, right_bits, script_error);
    if (isnan(left.u.floatValue)) puts("nan");
    else printf("%08" PRIx32 "\n", raw(left));
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
    _Static_assert(CHAR_BIT == 8 && sizeof(int) == 4 && sizeof(float) == 4, "Source scalar domain");
    _Static_assert(FLT_RADIX == 2 && FLT_MANT_DIG == 24 && FLT_MAX_EXP == 128 && FLT_EVAL_METHOD == 0,
                   "Explicit binary32 evaluation profile");
    _mm_setcsr(0x1f80);
    const uint32_t integers[] = {
        0, 1, 0xffffffff, 2, 0xfffffffe, 0x7fffffff, 0x80000000, 0x00ffffff,
        0x01000000, 0x01000001, 0xfeffffff, 0x01000003, 0x0000ffff, 0xffff0001,
        0x40000000, 0xc0000000
    };
    const uint32_t floats[] = {
        0, 0x80000000, 1, 0x80000001, 0x007fffff, 0x00800000, 0x3f800000, 0xbf800000,
        0x3f000000, 0x3fc00000, 0x358637bc, 0x358637bd, 0x358637be, 0x7f7fffff,
        0x7f800000, 0xff800000, 0x7fc00001, 0x7f800001, 0x4b800000, 0x4f000000
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
