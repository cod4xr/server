/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Explicit bounded allocator/globals. Unavailable value/engine paths fail closed. */
unsigned char scrCompileGlob[0x200], gScrCompilePub[0x21100], gScrVarPub[0x20000];
const char *var_typename[32];
static unsigned char arena[1024 * 1024] __attribute__((aligned(16)));
static Word used, allocations;
static void *blocks[128];
static Word sizes[128];

void *memcpy(void *destination, const void *source, Word size)
{
    volatile unsigned char *out = destination;
    const volatile unsigned char *in = source;
    for (Word i = 0; i < size; ++i) out[i] = in[i];
    return destination;
}
void *Hunk_AllocateTempMemoryHigh(Word size)
{
    if (size > sizeof(arena) - 16 || allocations == 128) unexpected("scope allocation bound");
    Word length = (size + 15) & ~(Word)15;
    if (used > sizeof(arena) - length) unexpected("scope arena bound");
    void *result = arena + used;
    used += length;
    blocks[allocations] = result;
    sizes[allocations++] = size;
    return result;
}
#define UNAVAILABLE(name) void name(void) { unexpected(#name); }
UNAVAILABLE(CompileError)
UNAVAILABLE(Com_Error)
UNAVAILABLE(Scr_AllocVector)
UNAVAILABLE(AddRefToValue)
UNAVAILABLE(Scr_EvalBinaryOperator)
#if !defined(NATIVE_SCOPE_CONSTANTS)
UNAVAILABLE(RemoveRefToValue)
UNAVAILABLE(Scr_CastBool)
#endif
#undef UNAVAILABLE

__attribute__((regparm(2))) void Scr_CalcLocalVarsVariableExpressionRef(Word *, Word *);
__attribute__((regparm(2))) void Scr_CalcLocalVarsStatement(Word *, Word *);

static void reset(void)
{
    used = allocations = 0;
    for (Word i = 0; i < sizeof(arena); ++i) arena[i] = 0;
    for (Word i = 0; i < sizeof(scrCompileGlob); ++i) scrCompileGlob[i] = 0;
    for (Word i = 0; i < sizeof(gScrCompilePub); ++i) gScrCompilePub[i] = 0;
    for (Word i = 0; i < sizeof(gScrVarPub); ++i) gScrVarPub[i] = 0;
}
static void show(const char *label, const Word *block)
{
    if (block[3] > 64) unexpected("scope trace width");
    text(label);
    for (Word i = 0; i < 6; ++i) { text(" "); decimal(block[i]); }
    for (Word i = 0; i < block[3]; ++i) { text(" n:"); decimal(block[6 + i]); }
    text("\n");
}
