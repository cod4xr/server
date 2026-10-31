// SPDX-License-Identifier: AGPL-3.0-or-later
// Synthetic container construction; all lookup/validation/checksum is original C.
static void word(byte *bytes, unsigned offset, uint32_t value) {
    for (unsigned i = 0; i < 4; ++i) bytes[offset + i] = value >> (i * 8);
}

static void hex(const byte *bytes, unsigned length) {
    if (!length) putchar('~');
    for (unsigned i = 0; i < length; ++i) printf("%02x", bytes[i]);
}

static unsigned build(byte *bytes, unsigned version) {
    memset(bytes, 0, 4096);
    memcpy(bytes, "IBSP", 4);
    word(bytes, 4, version);
    if (version <= 18) {
        unsigned end = 8 + Com_GetBspLumpCountForVersion(version) * 8;
        word(bytes, 8, 3); word(bytes, 12, end);
        memcpy(bytes + end, "mat", 3);
        bytes[end + 3] = 0x5b;
        word(bytes, 8 + 4 * 8, 8); word(bytes, 12 + 4 * 8, end + 4);
        for (unsigned i = 0; i < 8; ++i) bytes[end + 4 + i] = i + 7;
        // Retain checksum-sensitive trailing data outside all payload spans.
        bytes[end + 12] = 0xfa; bytes[end + 13] = 0x5e;
        return end + 14;
    }
    const unsigned kinds[] = {0, 4, 39, 4, 77, 1};
    const unsigned lengths[] = {3, 8, 0, 2, 5, 0};
    word(bytes, 8, 6);
    unsigned cursor = 60, end = cursor;
    for (unsigned i = 0; i < 6; ++i) {
        word(bytes, 12 + i * 8, kinds[i]);
        word(bytes, 16 + i * 8, lengths[i]);
        for (unsigned j = 0; j < lengths[i]; ++j) bytes[cursor + j] = 17 + i * 11 + j;
        if (lengths[i]) end = cursor + lengths[i];
        unsigned next = cursor + ((lengths[i] + 3) & ~3u);
        for (unsigned j = cursor + lengths[i]; j < next; ++j) bytes[j] = 0xa5;
        cursor = next;
    }
    // No final padding is required by the original lookup path for empty entries.
    return end;
}

int main(void) {
    _Static_assert(sizeof(enum LumpType) == 4, "Original 32-bit lump kind");
    union { max_align_t alignment; byte data[4096]; } storage;
    const unsigned queries[] = {0, 1, 4, 39, 77, 100, 4};
    for (unsigned version = 6; version <= 45; ++version) {
        unsigned length = build(storage.data, version);
        comBspGlob.header = (BspHeader *)storage.data;
        comBspGlob.fileSize = length;
        assert(!Com_BspError());
        for (unsigned query = 0; query < 7; ++query) {
            unsigned element_size = query == 6 ? 4 : 1;
            unsigned count = 0;
            byte *result = Com_GetBspLump(queries[query], element_size, &count);
            printf("bsp %u ", Com_GetBspVersion());
            hex(storage.data, length);
            printf(" %u %u %u ", queries[query], element_size, count);
            if (result) printf("%u", (unsigned)(result - storage.data));
            else putchar('~');
            putchar(' '); hex(result, count * element_size);
            printf(" %u %u\n", (unsigned)Com_BspHasLump(queries[query]),
                Com_BlockChecksumKey32(storage.data, length, 0));
        }
    }
    return 0;
}
