// SPDX-License-Identifier: AGPL-3.0-or-later
// The original function narrows an address to DWORD. Keep every computed pointer
// inside a real allocation, then record unsigned relative offsets modulo 2^32.
// No address values or implementation copies are used as product fixtures.
static byte storage[9][4096];

int main(void) {
    const uint32_t offsets[] = {0, 1, 3, 4, 15, 16, 255, 256, 1023, 4095};
    for (unsigned block = 0; block < 9; ++block) {
        g_streamBlocks[block].data = storage[block];
        g_streamBlocks[block].size = sizeof(storage[block]);
    }
    for (unsigned block = 0; block < 9; ++block) {
        for (unsigned index = 0; index < sizeof(offsets) / sizeof(offsets[0]); ++index) {
            uint32_t encoded = ((uint32_t)block << 28) + offsets[index] + 1;
            uint32_t result = encoded;
            DB_ConvertOffsetToPointer(&result);
            uint32_t base = (uint32_t)(uintptr_t)g_streamBlocks[block].data;
            printf("%u %u %u\n", encoded, block, (uint32_t)(result - base));
        }
    }
    return 0;
}
