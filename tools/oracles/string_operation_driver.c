/* SPDX-License-Identifier: AGPL-3.0-or-later */
static uint64_t append_hash(uint64_t hash, const char *bytes, unsigned int length)
{
    for (unsigned int i = 0; i < length; ++i) {
        hash ^= (unsigned char)bytes[i];
        hash *= UINT64_C(1099511628211);
    }
    return hash;
}

static void sample(unsigned int seed, unsigned int left, unsigned int right)
{
    const unsigned char lp[] = {'A', 'b', '_', '1', 0xff, 0x80};
    const unsigned char rp[] = {'x', 'Y', '-', '7', 0xe9};
    for (unsigned int i = 0; i < left; ++i) left_bytes[i] = (char)lp[(i + seed) % sizeof(lp)];
    for (unsigned int i = 0; i < right; ++i) right_bytes[i] = (char)rp[(i + seed) % sizeof(rp)];
    left_bytes[left] = right_bytes[right] = 0;
    left_length = left;
    right_length = right;
    removed[0] = removed[1] = 0;
    script_error = 0;
    result_length = 0;
    VariableValue a = {0}, b = {0};
    a.type = b.type = VAR_STRING;
    a.u.stringValue = 1;
    b.u.stringValue = 2;
    Scr_EvalPlus(&a, &b);
    if (removed[0] != 1 || removed[1] != 1) abort();
    uint64_t complete = append_hash(UINT64_C(14695981039346656037), left_bytes, left);
    complete = append_hash(complete, right_bytes, right);
    printf("%u %u %u %d ", seed, left, right, script_error);
    if (script_error) {
        if (a.type != VAR_UNDEFINED || b.type != VAR_UNDEFINED) abort();
        printf("~ ");
    } else {
        if (a.type != VAR_STRING || a.u.stringValue != 3 || result_length != left + right) abort();
        uint64_t native = append_hash(UINT64_C(14695981039346656037), result_bytes, result_length);
        if (native != complete) abort();
        printf("%016" PRIx64 " ", native);
    }
    printf("%016" PRIx64 "\n", complete);
}

int main(void)
{
    _Static_assert(CHAR_BIT == 8 && sizeof(unsigned int) == 4 && sizeof(int) == 4, "Source string domain");
    const unsigned int sizes[][2] = {
        {0, 0}, {1, 0}, {0, 1}, {1, 1}, {2, 7}, {127, 255}, {4095, 4095},
        {4095, 4096}, {4096, 4095}, {8190, 0}, {8191, 0}, {0, 8190}, {0, 8191},
        {8190, 1}, {8191, 1}, {10000, 50000}, {100000, 0}, {100000, 100000}
    };
    for (unsigned int seed = 0; seed < 6; ++seed) {
        for (unsigned int i = 0; i < sizeof(sizes) / sizeof(sizes[0]); ++i) {
            sample(seed, sizes[i][0], sizes[i][1]);
        }
    }
    return 0;
}
