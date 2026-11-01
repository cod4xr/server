/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Isolated archival x86 value/parent comparisons after source-role verification.
 * Parameter access below is modeled; NOT the whole original function/VM/ABI. */
static unsigned int value_defined(unsigned int type)
{
    __asm__ volatile (
        "test %%eax, %%eax\n\t"
        "setnz %%al\n\t"
        "movzx %%al, %%eax"
        : "+a" (type) : : "cc"
    );
    return type;
}

static unsigned int pointer_defined(unsigned int type)
{
    __asm__ volatile (
        "cmp $0x15, %%eax\n\t"
        "jg 1f\n\t"
        "cmp $0x13, %%eax\n\t"
        "jz 1f\n\t"
        "mov $0x1, %%eax\n\t"
        "jmp 2f\n"
        "1: xor %%eax, %%eax\n"
        "2:"
        : "+a" (type) : : "cc"
    );
    return type;
}

static void sample(unsigned int type, unsigned int pointer_type, uint32_t count)
{
    /* Modeled Scr_GetType(0) admission, grounded in its separately checked block. */
    printf("D %u %08" PRIx32 " %" PRIu32, type, pointer_type, count);
    if (count == 0) puts(" missing");
    else printf(" ok %u\n", type == VAR_POINTER ? pointer_defined(pointer_type) : value_defined(type));
}

int main(void)
{
    _Static_assert(sizeof(unsigned int) == 4, "Archival EAX word shape");
    static const uint32_t counts[] = {0, 1, 2, 3, 32, 255, 256, 65536, UINT32_MAX};
    for (unsigned int i = 0; i < sizeof counts / sizeof counts[0]; ++i) {
        for (unsigned int type = 0; type <= VAR_ENDON_LIST; ++type) {
            if (type != VAR_POINTER) sample(type, 0, counts[i]);
        }
        for (unsigned int parent = VAR_THREAD; parent <= VAR_ENDON_LIST; ++parent)
            sample(VAR_POINTER, parent, counts[i]);
        /* Raw comparator words only, not valid allocated native parent types. */
        sample(VAR_POINTER, 0x80000000u, counts[i]);
        sample(VAR_POINTER, 0xffffffffu, counts[i]);
    }
    return 0;
}
