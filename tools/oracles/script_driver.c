// SPDX-License-Identifier: AGPL-3.0-or-later
static VariableValue number(int type, uint32_t bits) {
    VariableValue value;
    memset(&value, 0, sizeof(value));
    value.type = type;
    memcpy(&value.u, &bits, 4);
    return value;
}

static void numeric(int type, uint32_t bits) {
    VariableValue value = number(type, bits);
    Scr_CastBool(&value);
    int truth = value.u.intValue;
    value = number(type, bits);
    Scr_EvalBoolNot(&value);
    int negated = value.u.intValue;
    value = number(type, bits);
    script_error = 0;
    Scr_EvalBoolComplement(&value);
    printf("N %d %08x %d %d %d %08x\n", type, bits, truth, negated,
           script_error, script_error ? 0 : (uint32_t)value.u.intValue);
}

static void wait(int fps, int type, uint32_t bits) {
    VariableValue value = number(type, bits);
    if (type == VAR_FLOAT) {
        float product = (float)fps * value.u.floatValue;
        if (!isfinite(value.u.floatValue) ||
            (value.u.floatValue >= 0 && (!isfinite(product) || product >= 2147483648.0f))) return;
    } else if (value.u.intValue >= 0 && (int64_t)value.u.intValue * fps > INT_MAX) {
        return; // Do not exercise undefined signed multiplication in the original.
    }
    level.framerate = fps;
    script_error = 0;
    int ticks = VM_CalcWaitTime(&value);
    printf("W %d %d %08x %d %d\n", fps, type, bits, script_error, ticks);
}

int main(void) {
    // The source helper obeys MXCSR. Explicitly set the profile this oracle tests;
    // do not rely on a shell/OS default or claim other engine environments match.
    _mm_setcsr(0x1f80);
    const uint32_t bits[] = {
        0, 0x80000000, 1, 0x007fffff, 0x00800000, 0x3b03126f,
        0x3ccccccc, 0x3ccccccd, 0x3cccccce, 0x3d999999, 0x3d99999a,
        0x3e000000, 0x3e333333, 0x3e800000, 0x3f000000, 0x3f400000,
        0x3f800000, 0x3fc00000, 0x40200000, 0xbf800000,
        0x7fffffff, 0xffffffff, 0x7f800000, 0xff800000, 0x7fc00001,
        0x7f800001, 0x4effffff, 0x4f000000, 0x4b7fffff, 0x4b800001,
    };
    const int rates[] = {1, 10, 20, 30, 60, 120, 1000, 16777217};
    for (unsigned index = 0; index < sizeof(bits) / sizeof(bits[0]); ++index) {
        numeric(VAR_INTEGER, bits[index]);
        numeric(VAR_FLOAT, bits[index]);
        for (unsigned rate = 0; rate < sizeof(rates) / sizeof(rates[0]); ++rate) {
            wait(rates[rate], VAR_INTEGER, bits[index]);
            wait(rates[rate], VAR_FLOAT, bits[index]);
        }
    }
    uint32_t state = 0x136a89f7;
    for (unsigned index = 0; index < 256; ++index) {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        numeric(VAR_INTEGER, state);
        numeric(VAR_FLOAT, state);
    }
    return 0;
}
