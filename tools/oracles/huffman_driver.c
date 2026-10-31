// SPDX-License-Identifier: AGPL-3.0-or-later
static void print_bytes(const byte *bytes, int length) {
    if (!length) putchar('~');
    for (int i = 0; i < length; ++i) printf("%02x", bytes[i]);
}

int main(void) {
    Huffman_InitMain();
    for (int symbol = 0; symbol <= 256; ++symbol) {
        byte bytes[16] = {0};
        int offset = 0;
        Huff_offsetTransmit(&msgHuff, symbol, bytes, &offset);
        printf("code %d %d ", symbol, offset);
        print_bytes(bytes, (offset + 7) / 8);
        putchar('\n');
    }
    const int lengths[] = {0, 1, 2, 7, 31, 128, 256, 1024};
    for (int pattern = 0; pattern < 4; ++pattern) {
        for (int sample = 0; sample < 8; ++sample) {
            int length = lengths[sample];
            byte plain[1024], encoded[4096] = {0}, decoded[16384] = {0};
            for (int i = 0; i < length; ++i) {
                switch (pattern) {
                    case 0: plain[i] = 0; break;
                    case 1: plain[i] = i & 0xff; break;
                    case 2: plain[i] = (i * 73 + 11) & 0xff; break;
                    case 3: plain[i] = "getstatus snapshot entity command"[i % 33]; break;
                }
            }
            int encoded_length = MSG_WriteBitsCompress(plain, encoded, length);
            // Extra bytes make the original decoder's final-code overread
            // deterministic and memory-safe within this disposable oracle.
            int decoded_length = MSG_ReadBitsCompress(encoded, encoded_length,
                                                      decoded, sizeof(decoded));
            printf("sample %d %d ", pattern, length);
            print_bytes(plain, length);
            putchar(' ');
            print_bytes(encoded, encoded_length);
            putchar(' ');
            print_bytes(decoded, decoded_length);
            putchar('\n');
        }
    }
    return 0;
}
