/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Local-slot COW only. Unavailable entity/game branches abort instead of inventing results. */
#define CLASS_NUM_COUNT 1u
#define UNK_ENTNUM_MASK 0x3fff
static struct { unsigned int id; } gScrClassMap[CLASS_NUM_COUNT];
static const char *var_typename[VAR_COUNT];
static unsigned int ref_removals, error_calls, error_category;
static unsigned int rejected_type, rejected_slot;
enum ErrorProfile { UNEXPECTED_ERROR, LOCAL_TYPE, CLEAR_UNDEFINED, CLEAR_INDEX };
static enum ErrorProfile error_profile;
static unsigned int rejected_word, rejected_original;
static _Noreturn void validate_error(void);
static void SL_RemoveRefToString(unsigned int label);

static void RemoveRefToObject(unsigned int id)
{
    assert(id > 0 && id <= object_count);
    VariableValueInternal *entry = &variable_pool[id + VARIABLELIST_PARENT_BEGIN];
    assert(IsObject(entry) && entry->u.o.refCount > 0 && gScrVarPub.totalObjectRefCount > 0);
    --entry->u.o.refCount;
    --gScrVarPub.totalObjectRefCount;
    ++ref_removals;
}
static const char *va(const char *format, ...)
{
    assert(strcmp(format, "%s is not an array") == 0
           || strcmp(format, "array index %d out of range") == 0
           || strcmp(format, "%s is not an array index") == 0);
    return format;
}
static _Noreturn void Scr_Error(const char *message)
{
    ++error_calls;
    if (strcmp(message, "%s is not an array") == 0) error_category = 1;
    else if (strcmp(message, "string characters cannot be individually changed") == 0)
        error_category = 2;
    else if (strcmp(message, "vector components cannot be individually changed") == 0)
        error_category = 3;
    else if (strcmp(message, "array index %d out of range") == 0) error_category = 4;
    else if (strcmp(message, "%s is not an array index") == 0) error_category = 5;
    else abort();
    validate_error();
}
static unsigned int FindArrayVariable(unsigned int parent, unsigned int name)
{
    (void)parent; (void)name; abort();
}
static VariableValue GetEntityFieldValue(unsigned int classnum, unsigned int entnum, unsigned int offset)
{
    (void)classnum; (void)entnum; (void)offset; abort();
}
static void RemoveRefToValue(unsigned int type, VariableUnion value)
{
    if (type == VAR_STRING) SL_RemoveRefToString(value.stringValue);
    else assert(type == VAR_UNDEFINED || type == VAR_INTEGER || type == VAR_FLOAT);
}
static unsigned int GetNewVariable(unsigned int parent, unsigned int name)
{
    (void)parent; (void)name; abort();
}

static unsigned int new_local(unsigned int owner, unsigned int name, unsigned int pointer)
{
    set_word(owner, name, VAR_POINTER, pointer);
    unsigned int index = FindVariableIndexInternal(owner, name);
    return variable_pool[index + VARIABLELIST_CHILD_BEGIN].hash.id;
}
