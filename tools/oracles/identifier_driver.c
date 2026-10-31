/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Drives the original conversion function; native string/ref services are stubs. */
static void emit(const char *kind, unsigned int label, unsigned int length)
{
    removed = 0;
    unsigned int result = SL_ConvertToLowercase(17, 1, 14);
    const char *canonical = result == 17 ? input_bytes : converted;
    if (result != 17 && result != 23) abort();
    if (removed != (result == 23 ? 1U : 0U)) abort();
    uint64_t hash = UINT64_C(14695981039346656037);
    for (unsigned int i = 0; i < length; ++i) {
        hash ^= (unsigned char)canonical[i];
        hash *= UINT64_C(1099511628211);
    }
    printf("%s %u %u %016" PRIx64 " %u\n", kind, label, length, hash,
           result == 23 ? 1U : 0U);
}

int main(void)
{
    _Static_assert(CHAR_BIT == 8 && sizeof(unsigned int) == 4, "Source integer domain");
    if (!setlocale(LC_ALL, "C")) return 2;
    for (unsigned int value = 0; value < 128; ++value) {
        if ((value >= 'A' && value <= 'Z') ||
            (value >= 'a' && value <= 'z') || value == '_') {
            input_bytes[0] = (char)value;
            input_bytes[1] = 0;
            emit("S", value, 1);
        }
    }
    const unsigned int lengths[] = {1, 2, 7, 127, 8190, 8191, 8192, 8193, 100000};
    const char pattern[] = "AaZz_19";
    for (unsigned int seed = 0; seed < 8; ++seed) {
        for (unsigned int sample = 0; sample < sizeof(lengths) / sizeof(lengths[0]); ++sample) {
            unsigned int length = lengths[sample];
            input_bytes[0] = 'N';
            for (unsigned int i = 1; i < length; ++i) {
                input_bytes[i] = pattern[(i + seed) % 7];
            }
            input_bytes[length] = 0;
            emit("P", seed, length);
        }
    }
    return 0;
}
