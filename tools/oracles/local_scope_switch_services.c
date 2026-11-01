/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Sorting/output and encoded integer-key services are explicitly modeled. */
void *TempMallocAlignStrict(Word size) { return TempMalloc(size); }
int IsValidArrayIndex(Word value) { return value < 0x800000; }
Word GetInternalVariableIndex(Word value)
{
    if (!IsValidArrayIndex(value)) unexpected("scope switch integer domain");
    return (value + 0x800000) & 0xffffff;
}
static Word sorted_keys[16], sorted_count, transferred_strings[16], transfer_count;
void SL_TransferRefToUser(Word id, Word user)
{
    if (id < 1 || id > 5 || user != 1 || transfer_count == 16)
        unexpected("scope case string transfer profile");
    transferred_strings[transfer_count++] = id;
}
void va(void) { unexpected("scope switch diagnostic"); }
void qsort(void *base, Word count, Word size, int (*compare)(const void *, const void *))
{
    if (count > 16 || size != 8 || !compare) unexpected("scope switch sorting profile");
    unsigned char *bytes = base;
    for (Word i = 1; i < count; ++i)
        for (Word j = i; j > 0 && compare(bytes + (j - 1) * size, bytes + j * size) > 0; --j)
            for (Word k = 0; k < size; ++k) {
                unsigned char value = bytes[(j - 1) * size + k];
                bytes[(j - 1) * size + k] = bytes[j * size + k];
                bytes[j * size + k] = value;
            }
    sorted_count = count;
    for (Word i = 0; i < count; ++i)
        memcpy(&sorted_keys[i], bytes + i * size, sizeof(Word));
}
