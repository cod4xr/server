/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Bounded modeled allocator/output slot; unsupported cleanup branches abort. */
#define RECORD_LIMIT 16u
#define CLASS_NUM_COUNT 1u
#define UNK_ENTNUM_MASK 0x3fffu
static struct { VariableValueInternal variableList[VARIABLELIST_PARENT_BEGIN + RECORD_LIMIT + 1]; } gScrVarGlob;
static struct { unsigned int totalObjectRefCount; } gScrVarPub;
static struct DebugState { unsigned int leakCount[VARIABLELIST_PARENT_BEGIN + RECORD_LIMIT + 1]; }
    debug_state, *gScrVarDebugPub;
static struct { unsigned int entArrayId; } gScrClassMap[CLASS_NUM_COUNT];
static struct { VariableValue *top; } gScrVmPub;
static VariableValue output;
static unsigned int allocated, output_calls;

static unsigned int AllocVariable(void)
{
    assert(allocated < RECORD_LIMIT);
    unsigned int id = ++allocated;
    VariableValueInternal *entry = &gScrVarGlob.variableList[VARIABLELIST_PARENT_BEGIN + id];
    assert(entry->w.status == 0);
    memset(entry, 0, sizeof *entry);
    ++gScrVarPub.totalObjectRefCount;
    if (gScrVarDebugPub) gScrVarDebugPub->leakCount[VARIABLELIST_PARENT_BEGIN + id] = 1;
    return id;
}
static void IncInParam(void)
{
    assert(output_calls == 0);
    ++output_calls;
    gScrVmPub.top = &output;
}
static void RemoveArrayVariable(unsigned int parent, unsigned int name)
{ (void)parent; (void)name; abort(); }
static void ClearObject(unsigned int id) { (void)id; abort(); }
static void FreeVariable(unsigned int id) { (void)id; abort(); }
