/* SPDX-License-Identifier: AGPL-3.0-or-later */
static uint32_t next_bits(uint32_t *state)
{
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    return *state;
}

int main(void)
{
    _Static_assert(CHAR_BIT == 8 && sizeof(int) == 4 && sizeof(float) == 4, "Source domains");
    _Static_assert(FLT_RADIX == 2 && FLT_MANT_DIG == 24 && FLT_EVAL_METHOD == 0,
                   "Explicit binary32 evaluation profile");
    _mm_setcsr(0x1f80);
    const uint32_t floats[] = {
        0, 0x80000000, 1, 0x80000001, 0x007fffff, 0x00800000,
        0x3f800000, 0xbf800000, 0x3f000000, 0x3fc00000, 0x3f800001,
        0x358637bd, 0x7f7fffff, 0x7f800000, 0xff800000, 0x7fc00001, 0x7f800001
    };
    const uint32_t integers[] = {
        0, 1, 0xffffffff, 0x7fffffff, 0x80000000, 0x00ffffff,
        0x01000000, 0x01000001, 0x01000003, 0xfeffffff
    };
    Operand numbers[32];
    unsigned int count = 0;
    for (unsigned int i = 0; i < sizeof(integers)/sizeof(integers[0]); ++i)
        numbers[count++] = (Operand){'I', {integers[i], 0, 0}};
    for (unsigned int i = 0; i < sizeof(floats)/sizeof(floats[0]); ++i)
        numbers[count++] = (Operand){'F', {floats[i], 0, 0}};
    Operand vectors[24];
    unsigned int vector_count = 0;
    for (unsigned int i = 0; i < sizeof(floats)/sizeof(floats[0]); ++i)
        vectors[vector_count++] = (Operand){'V', {floats[i], floats[i], floats[i]}};
    vectors[vector_count++] = (Operand){'V', {0x3f800000, 0, 0x40000000}};
    vectors[vector_count++] = (Operand){'V', {0x3f800000, 0x40000000, 0x80000000}};
    vectors[vector_count++] = (Operand){'V', {0x3f800000, 0x3f800001, 0x7f800000}};
    for (unsigned int i = 0; i < count; ++i) {
        sample_cast(numbers[i], numbers[i], numbers[i]);
        for (unsigned int j = 0; j < count; ++j)
            sample_cast(numbers[i], numbers[j], numbers[(i*7+j*11)%count]);
    }
    for (unsigned int i = 0; i < vector_count; ++i) {
        for (unsigned int j = 0; j < vector_count; ++j) sample(vectors[i], vectors[j]);
        for (unsigned int j = 0; j < count; ++j) {
            sample(vectors[i], numbers[j]);
            sample(numbers[j], vectors[i]);
        }
    }
    uint32_t state = 0x43ab1029;
    for (unsigned int i = 0; i < 256; ++i) {
        Operand left = {'V', {0}}, right = {'V', {0}};
        for (unsigned int coordinate = 0; coordinate < 3; ++coordinate) {
            left.bits[coordinate] = next_bits(&state);
            right.bits[coordinate] = next_bits(&state);
        }
        sample(left, right);
    }
    return 0;
}
