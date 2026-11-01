/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Models only services/field names used by copied C time functions; no native ABI. */
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#ifndef __cdecl
#define __cdecl
#endif
#define CON_CHANNEL_PARSERSCRIPT 0
static struct { unsigned int time, timeArrayId; void *error_message; int error_index; } gScrVarPub;
static struct { int function_count, outparamcount, inparamcount; void *top, *stack; } gScrVmPub;
static struct { int boolean; } enabled, logging;
static __typeof__(enabled) *scrVmEnableScripts = &enabled;
static __typeof__(logging) *logScriptTimes = &logging;
static int gScrExecuteTime, raw_calls, resumes, logs, frees, bucket;

static int Sys_MillisecondsRaw(void)
{
    printf("B %u\n", gScrVarPub.time); return ++raw_calls;
}
static int Sys_Milliseconds(void) { return 777; }
static unsigned int FindVariable(unsigned int array, unsigned int time)
{
    assert(array == 100 && time == gScrVarPub.time);
    printf("L %u\n", time); return bucket ? 200 : 0;
}
static unsigned int FindObject(unsigned int id) { assert(id == 200 && bucket); return 300; }
static void VM_Resume(unsigned int id)
{
    assert(id == 300 && bucket); ++resumes; printf("R %u\n", gScrVarPub.time);
}
static void SafeRemoveVariable(unsigned int array, unsigned int time)
{
    assert(array == 100 && time == gScrVarPub.time && bucket);
    bucket = 0; printf("D %u\n", time);
}
static void Com_Printf(int channel, const char *format, int milliseconds)
{
    assert(channel == CON_CHANNEL_PARSERSCRIPT && format && milliseconds == 777 && logging.boolean);
    ++logs; printf("O %u\n", gScrVarPub.time);
}
static void Scr_FreeEntityList(void) { ++frees; printf("F %u\n", gScrVarPub.time); }
