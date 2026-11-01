/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Local-slot COW only. Unavailable entity/game branches abort instead of inventing results. */
#define CLASS_NUM_COUNT 1u
#define UNK_ENTNUM_MASK 0x3fff
static struct { unsigned int id; } gScrClassMap[CLASS_NUM_COUNT];
static const char *var_typename[VAR_COUNT];
static unsigned int ref_removals, error_calls, error_category;
static unsigned int rejected_type, rejected_slot;

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
    assert(strcmp(format, "%s is not an array") == 0);
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
    else abort();
    assert(rejected_type > 0 && rejected_slot > 0);
    assert(gScrVarPub.error_index == 1 && error_calls == 1 && !ref_removals);
    assert(object_count == 2 && record_count == 1);
    assert(VAR_TYPE((&variable_pool[rejected_slot + VARIABLELIST_CHILD_BEGIN])) == rejected_type);
    printf("E %u %u %u\n", rejected_type, error_category, error_calls);
    exit(EXIT_SUCCESS);
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
    (void)type; (void)value; abort();
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
