/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Fixed terminated byte labels; all object/vector runtime branches abort if used. */
typedef struct { union VariableUnion u; int type; } VariableValue;
typedef struct {
    union { unsigned int type; unsigned int status; } w;
    struct { struct { struct { unsigned int size; } u; } o; } u;
} VariableValueInternal;
static struct { VariableValueInternal variableList[1]; } gScrVarGlob;
static struct { int error_index; } gScrVarPub;
static const char *var_typename[VAR_ENDON_LIST + 1];
static unsigned int script_error, removals, conversions, creations, removed_type;
static unsigned char large_string[70001];
static const unsigned char *const texts[] = {
    (const unsigned char *)"", (const unsigned char *)"Rust",
    (const unsigned char *)"\xff\x80" "A", (const unsigned char *)"\xc3\xa9",
    (const unsigned char *)"before\0after", (const unsigned char *)"\0tail",
    (const unsigned char *)"abc\0", large_string
};
static const char *SL_ConvertToString(unsigned int label)
{
    assert(label >= 1 && label <= sizeof texts / sizeof texts[0]);
    ++conversions;
    return (const char *)texts[label - 1];
}
static unsigned int SL_GetStringOfSize(const char *bytes, unsigned int user,
                                        unsigned int size, int type)
{
    assert(user == 0 && size == 2 && type == 15 && bytes[0] != 0 && bytes[1] == 0);
    ++creations;
    return 0x100u + (unsigned char)bytes[0];
}
static void SL_RemoveRefToString(unsigned int label)
{
    assert(label >= 1 && label <= sizeof texts / sizeof texts[0]);
    ++removals;
}
static char *va(const char *format, ...)
{
    if (strcmp(format, "string index %d out of range") == 0) script_error = 1;
    else if (strcmp(format, "%s is not a string index") == 0) script_error = 2;
    else if (strcmp(format, "size cannot be applied to %s") == 0) script_error = 3;
    else abort();
    return "checked fixed helper error";
}
static void Scr_Error(const char *message)
{
    assert(script_error != 0 && strcmp(message, "checked fixed helper error") == 0);
}
static void RemoveRefToValue(int type, union VariableUnion value)
{
    (void)value;
    assert(type != VAR_POINTER && type != VAR_STRING && type != VAR_STACK);
    removed_type = (unsigned int)type;
    ++removals;
}
static unsigned int Scr_FindArrayIndex(unsigned int parent, VariableValue *index)
{ (void)parent; (void)index; abort(); }
static VariableValue Scr_EvalVariable(unsigned int id) { (void)id; abort(); }
static void RemoveRefToObject(unsigned int id) { (void)id; abort(); }
static void RemoveRefToVector(const float *value) { (void)value; abort(); }

static void reset_services(void)
{
    script_error = removals = conversions = creations = removed_type = 0;
    gScrVarPub.error_index = 0;
}
