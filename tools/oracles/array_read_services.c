/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Bounded nonfreeing refs/error services; not native interning, allocator or VM cleanup. */
static const char *var_typename[VAR_COUNT];
static unsigned int parent_removals, read_errors, read_category;
static unsigned int active_parent, active_aliases, active_key_type, active_key_word;
static _Noreturn void validate_read_error(void);

static void RemoveRefToObject(unsigned int id)
{
    assert(id > 0 && id <= object_count);
    VariableValueInternal *entry = &variable_pool[id + VARIABLELIST_PARENT_BEGIN];
    assert(IsObject(entry) && entry->u.o.refCount > 0 && gScrVarPub.totalObjectRefCount > 0);
    --entry->u.o.refCount;
    --gScrVarPub.totalObjectRefCount;
    ++parent_removals;
}

static void SL_RemoveRefToString(unsigned int label)
{
    assert(label > 0 && label < SL_MAX_STRING_INDEX);
    ++string_removals;
}

static char *va(const char *format, ...)
{
    /* Fixed format identity only; no native formatting or error propagation is modeled. */
    return (char *)format;
}

static _Noreturn void Scr_Error(const char *message)
{
    ++read_errors;
    if (strcmp(message, "array index %d out of range") == 0) read_category = 1;
    else if (strcmp(message, "%s is not an array index") == 0) read_category = 2;
    else if (strcmp(message, "%s is not an array") == 0) read_category = 3;
    else abort();
    validate_read_error();
}

static const char *SL_ConvertToString(unsigned int label) { (void)label; abort(); }
static unsigned int SL_GetStringOfSize(const char *bytes, unsigned int user,
                                     unsigned int size, unsigned int type)
{ (void)bytes; (void)user; (void)size; (void)type; abort(); }
static void RemoveRefToVector(const float *vector) { (void)vector; abort(); }
static void RemoveRefToValue(int type, VariableUnion value) { (void)type; (void)value; abort(); }
