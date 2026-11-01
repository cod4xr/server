/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Actual selected declarations; modeled records and reference callback only. */
static struct { VariableValueInternal variableList[VARIABLELIST_PARENT_BEGIN + 2]; } gScrVarGlob;
static const char *var_typename[VAR_ENDON_LIST + 1];
static unsigned int removals;

static void RemoveRefToObject(unsigned int id)
{
    assert(id == 1 && removals == 0);
    ++removals;
}
static const char *SL_ConvertToString(unsigned int label) { (void)label; abort(); }
static void SL_RemoveRefToString(unsigned int label) { (void)label; abort(); }
static void RemoveRefToValue(int type, union VariableUnion value)
{ (void)type; (void)value; abort(); }
static char *va(const char *format, ...) { (void)format; abort(); }
static void Scr_Error(const char *message) { (void)message; abort(); }
