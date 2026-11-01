/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Modeled checked ordinary parents/independent child markers and reference callbacks only. */
static struct { VariableValueInternal variableList[VARIABLELIST_CHILD_BEGIN + 4]; } gScrVarGlob;
static struct { int evaluate; } gScrVarPub;
static unsigned int removals, removed_ids;

static void RemoveRefToObject(unsigned int id)
{
    assert(id >= 1 && id <= 3);
    ++removals;
    removed_ids |= 1u << id;
}
static float *Scr_AllocVectorInternal(void) { abort(); }
static int VectorCompare(const float *left, const float *right)
{ (void)left; (void)right; abort(); }
static void RemoveRefToVector(const float *value) { (void)value; abort(); }
static void SL_RemoveRefToString(unsigned int id) { (void)id; abort(); }
static void Scr_UnmatchingTypesError(VariableValue *left, VariableValue *right)
{ (void)left; (void)right; abort(); }
