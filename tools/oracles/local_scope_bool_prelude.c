/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Separate C unit for the actual pinned bool helper; unsupported errors fail. */
#ifndef __cdecl
#define __cdecl
#endif
#define VAR_UNDEFINED 0
#define VAR_FLOAT 5
#define VAR_INTEGER 6
typedef union { int intValue; float floatValue; } VariableUnion;
typedef struct { VariableUnion u; int type; } VariableValue;
_Static_assert(sizeof(VariableUnion) == 4 && sizeof(VariableValue) == 8,
    "Selected native bool helper requires word-sized payload/type");
extern const char *var_typename[32];
extern void RemoveRefToValue(int, VariableUnion);
extern void Scr_Error(const char *);
extern const char *va(const char *, ...);
