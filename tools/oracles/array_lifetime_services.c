/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Actual selected clear/free/ref helpers; unavailable services abort. */
#define CLASS_NUM_COUNT 1u
#define UNK_ENTNUM_MASK 0x3fffu
static struct { unsigned int id, entArrayId; } gScrClassMap[CLASS_NUM_COUNT];
static const char *var_typename[VAR_COUNT];
void RemoveRefToObject(unsigned int id);
static void RemoveRefToVector(const float *value) { (void)value; abort(); }
static void SL_RemoveRefToString(unsigned int id) { (void)id; abort(); }
static void RemoveArrayVariable(unsigned int parent, unsigned int key)
{ (void)parent; (void)key; abort(); }
static unsigned int FindArrayVariable(unsigned int parent, unsigned int key)
{ (void)parent; (void)key; abort(); }
static VariableValue GetEntityFieldValue(unsigned int classnum, unsigned int entnum, unsigned int offset)
{ (void)classnum; (void)entnum; (void)offset; abort(); }
static unsigned int GetNewVariable(unsigned int parent, unsigned int key)
{ (void)parent; (void)key; abort(); }
static const char *va(const char *format, ...) { (void)format; abort(); }
static void Scr_Error(const char *message) { (void)message; abort(); }
