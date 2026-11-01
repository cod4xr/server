/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Bounded modeled lookup, references and removal. No native allocation or freeing. */
#define CLASS_NUM_COUNT 1u
#define SCR_GET_ENTITY_FROM_ENTCLIENT(value) (value)
static struct { VariableValueInternal variableList[VARIABLELIST_CHILD_BEGIN + 4]; } gScrVarGlob;
static struct { unsigned int entId, entFieldName; } gScrVarPub;
static struct { unsigned int id; } gScrClassMap[CLASS_NUM_COUNT];
static const char *var_typename[VAR_ENDON_LIST + 1];
static unsigned int present, add_calls, remove_calls, lookup_calls;

static unsigned int FindVariableIndexInternal(unsigned int parent, unsigned int name)
{
    assert(parent == 1 && (name == 101 || name == 102));
    ++lookup_calls;
    return name == 101 && present ? 1 : 0;
}
static unsigned int FindVariable(unsigned int parent, unsigned int name)
{ return FindVariableIndexInternal(parent, name); }
static unsigned int GetVariableIndexInternal(unsigned int parent, unsigned int name)
{
    assert(parent == 1 && name == 101);
    present = 1;
    return 1;
}
static void AddRefToValue(int type, union VariableUnion value)
{
    assert(type != VAR_VECTOR);
    (void)value;
    ++add_calls;
}
static void RemoveRefToValue(int type, union VariableUnion value)
{
    assert(type != VAR_VECTOR);
    (void)value;
    ++remove_calls;
}
static void RemoveVariable(unsigned int parent, unsigned int name)
{
    assert(parent == 1 && name == 101 && present);
    present = 0;
    /* Deliberately modeled: native unlink/free paths are not exercised here. */
}
static VariableValue Scr_EvalVariableEntityField(unsigned int parent, unsigned int name)
{ (void)parent; (void)name; abort(); }
static unsigned int FindArrayVariable(unsigned int parent, unsigned int name)
{ (void)parent; (void)name; abort(); }
static void SetEntityFieldValue(unsigned int classnum, unsigned int entity, int field, VariableValue *value)
{ (void)classnum; (void)entity; (void)field; (void)value; abort(); }
static char *va(const char *format, ...) { (void)format; abort(); }
static void Scr_Error(const char *message) { (void)message; abort(); }
