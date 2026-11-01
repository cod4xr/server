/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Bounded harness services, not the native pool/refcount/string/error runtime. */
typedef struct { union VariableUnion u; int type; } VariableValue;
static struct { struct { struct { unsigned int type; } w; } variableList[1]; } gScrVarGlob;
static struct { int evaluate; int error_index; } gScrVarPub;
static float vector_pool[32][3];
static unsigned int vector_used;
static int script_error;
static jmp_buf error_target;
/* Invalid constructor types are outside the numeric-only construction corpus. */
static const char *const var_typename[32] = {0};

static float *Scr_AllocVectorInternal(void)
{
    if (vector_used >= 32) abort();
    return vector_pool[vector_used++];
}

static float *Scr_AllocVector(const float *value)
{
    float *result = Scr_AllocVectorInternal();
    memcpy(result, value, 3 * sizeof(float));
    return result;
}

static void RemoveRefToVector(const float *value)
{
    for (unsigned int i = 0; i < vector_used; ++i) {
        if (value == vector_pool[i]) return;
    }
    abort();
}

static void Scr_ClearVector(VariableValue *value) { (void)value; abort(); }
static _Noreturn void Scr_UnmatchingTypesError(VariableValue *left, VariableValue *right)
{
    (void)left; (void)right;
    script_error = 1;
    longjmp(error_target, 1);
}

static _Noreturn void Scr_Error(const char *message)
{
    (void)message;
    script_error = 2;
    longjmp(error_target, 1);
}

static unsigned int SL_GetStringForVector(const float *v) { (void)v; abort(); }
static unsigned int SL_GetStringForFloat(float v) { (void)v; abort(); }
static unsigned int SL_GetStringForInt(int v) { (void)v; abort(); }
static const char *SL_ConvertToString(unsigned int v) { (void)v; abort(); }
static unsigned int SL_GetStringLen(unsigned int v) { (void)v; abort(); }
static unsigned int SL_GetStringOfSize(const char *v, unsigned int u, unsigned int n, int t)
{ (void)v; (void)u; (void)n; (void)t; abort(); }
static void SL_RemoveRefToString(unsigned int v) { (void)v; abort(); }
static void RemoveRefToObject(unsigned int v) { (void)v; abort(); }
static const char *va(const char *format, ...) { return format; }
