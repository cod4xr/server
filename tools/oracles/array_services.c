/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Test services: bounded modeled pool/lookup/ref accounting, NOT native allocation/GC. */
enum { RECORD_LIMIT = 1024, OBJECT_LIMIT = 256, CAPTURE_LIMIT = 128 };
static VariableValueInternal variable_pool[VARIABLELIST_CHILD_BEGIN + VARIABLELIST_CHILD_SIZE];
static struct { VariableValueInternal *variableList; } gScrVarGlob = { variable_pool };
static struct {
    const char *varUsagePos;
    unsigned int totalObjectRefCount, numScriptValues, entId, entFieldName;
    int error_index;
} gScrVarPub;
static struct {
    unsigned int leakCount[VARIABLELIST_CHILD_BEGIN + VARIABLELIST_CHILD_SIZE];
    const char *varUsage[VARIABLELIST_CHILD_BEGIN + VARIABLELIST_CHILD_SIZE];
}
    *gScrVarDebugPub;
static struct {
    unsigned int parent, name, index, initial_hash;
} records[RECORD_LIMIT];
static unsigned int record_count, object_count, string_refs, vector_refs;
static unsigned int string_removals, peak_array_size;
static VariableValue captured_keys[CAPTURE_LIMIT];
static unsigned int captured_count, captured_pending;

unsigned int AllocVariable(void);
unsigned int GetNewVariableIndexInternal3(unsigned int parent, unsigned int name, unsigned int hash);
unsigned int FindVariableIndexInternal(unsigned int parent, unsigned int name);
unsigned int FindVariableIndexInternal2(unsigned int name, unsigned int hash);
void AddRefToValue(int type, VariableUnion value);
static void SL_AddRefToString(unsigned int label)
{
    assert(label > 0 && label < SL_MAX_STRING_INDEX);
    ++string_refs;
}
static void AddRefToVector(const float *value)
{
    assert(value != NULL);
    ++vector_refs;
}
static unsigned int Scr_GetObjectType(unsigned int id)
{
    assert(id > 0 && id <= object_count);
    return VAR_TYPE((&variable_pool[id + VARIABLELIST_PARENT_BEGIN]));
}
static void Scr_MakeArray(void)
{
    captured_count = 0;
    captured_pending = 0;
}
static void Scr_AddConstString(unsigned int label)
{
    assert(!captured_pending && captured_count < CAPTURE_LIMIT);
    assert(label > 0 && label < SL_MAX_STRING_INDEX);
    captured_keys[captured_count].type = VAR_STRING;
    captured_keys[captured_count].u.stringValue = label;
    captured_pending = 1;
}
static void Scr_AddInt(int value)
{
    assert(!captured_pending && captured_count < CAPTURE_LIMIT);
    captured_keys[captured_count].type = VAR_INTEGER;
    captured_keys[captured_count].u.intValue = value;
    captured_pending = 1;
}
static void Scr_AddArray(void)
{
    assert(captured_pending);
    ++captured_count;
    captured_pending = 0;
}
