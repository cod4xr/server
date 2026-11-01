/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Native VM_Resume only. Variable lists, archives and bytecode bodies are modeled. */
Word gScrVmGlob[0x201c / 4], gScrVmPub[0x4320 / 4], gScrVarPub[32], gFs[5], thread_count;
static Word local_variables[32];
static Word queue[32], queued, allocated, current, scenario, executions;
static Word clocks, bucket_refs, clears, frees, decrements, value_drops, object_drops;
static Word archives[32][16], pointers[32], keys[32], restored[32], freed[32];

static void event(const char *name, Word value)
{
    text(name); text(" "); decimal(value); text("\n");
}
static void require(int condition, const char *name)
{
    if (!condition) unexpected(name);
}
static Word slot(Word id)
{
    require(id >= 100 && id < 100 + allocated, "resume queue node");
    return id - 100;
}
static void enqueue(Word key, Word front, Word values, Word stamp)
{
    require(allocated < 32 && queued < 32 && values <= 3, "resume queue capacity");
    Word index = allocated++;
    keys[index] = key;
    Word *buffer = archives[index];
    buffer[0] = 1000 + index;
    ((unsigned short *)buffer)[2] = (unsigned short)values;
    ((unsigned short *)buffer)[3] = (unsigned short)(11 + 5 * values);
    ((unsigned short *)buffer)[4] = (unsigned short)key;
    ((unsigned char *)buffer)[10] = (unsigned char)stamp;
    for (Word item = 0; item < values; ++item) {
        unsigned char *value = (unsigned char *)buffer + 11 + 5 * item;
        value[0] = (unsigned char)(item == 2 ? 0 : 6);
        Word bits = 40 + item;
        for (Word byte = 0; byte < 4; ++byte) value[1 + byte] = (unsigned char)(bits >> (8 * byte));
    }
    pointers[index] = (Word)buffer;
    if (front) {
        for (Word item = queued; item; --item) queue[item] = queue[item - 1];
        queue[0] = 100 + index;
    } else queue[queued] = 100 + index;
    ++queued;
}
Word Sys_Milliseconds(void) { return ++clocks; }
void AddRefToObject(Word id)
{
    require(id == 77 && !bucket_refs, "resume bucket retain"); ++bucket_refs;
}
void RemoveRefToObject(Word id)
{
    if (id == 77) { require(bucket_refs == 1, "resume bucket release"); --bucket_refs; }
    else { require(id == 900 + executions, "resume result object release"); ++object_drops; }
}
Word FindFirstSibling(Word id)
{
    require(id == 77 && bucket_refs == 1, "resume bucket traversal");
    return queued ? queue[0] : 0;
}
Word GetVariableKeyObject(Word id) { return keys[slot(id)]; }
Word *GetVariableValueAddress(Word id) { return &pointers[slot(id)]; }
void RemoveObjectVariable(Word id, Word key)
{
    require(id == 77 && queued && keys[slot(queue[0])] == key, "resume removes head before body");
    current = slot(queue[0]);
    for (Word item = 1; item < queued; ++item) queue[item - 1] = queue[item];
    --queued;
    event("P", key);
}
void Scr_ClearWaitTime(Word key)
{
    require(key == keys[current] && !restored[current], "resume clears wait once");
    restored[current] = 1; ++clears; event("W", key);
}
Word FindLastSibling(Word id)
{
    require(id == keys[current], "resume local inventory"); return 0;
}
void MT_Free(void *buffer, Word bytes)
{
    require(buffer == archives[current] && bytes == ((unsigned short *)buffer)[3]
            && restored[current] && !freed[current], "resume archive free");
    freed[current] = 1; ++frees; event("F", keys[current]);
}
void Scr_DecNumScriptThreads(void) { ++decrements; }
void RemoveRefToValue(Word type, Word value)
{
    require(type == 6 && value == executions, "resume result value release"); ++value_drops;
}
void ClearVariableValue(Word id)
{
    require(id == 88 && !queued && !bucket_refs, "resume final bucket clear");
    ++gScrVarPub[31]; event("C", id);
}
#define UNAVAILABLE(name) void name(void) { unexpected(#name); }
UNAVAILABLE(FindPrevSibling)
UNAVAILABLE(GetParentLocalId)
UNAVAILABLE(Scr_GetVarId)
#undef UNAVAILABLE
__attribute__((regparm(1))) void VM_Resume(Word);
