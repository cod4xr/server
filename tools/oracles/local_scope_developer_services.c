/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Only developer-list and local-slot instructions execute natively. */
unsigned char scrCompileGlob[0x200], gScrCompilePub[0x21100], gScrVarPub[0x20000];
static unsigned char code[256];
static Word code_size, requests[16], request_count, bodies, modes[8], rewinds;
static Word diagnostics, adds, drops, canonical;
static Word dropped_names[4];

static Word *compile_mode(void) { return (Word *)(gScrCompilePub + 0x20024); }
static Word *source_position(void) { return (Word *)(gScrVarPub + 0x38); }
void *TempMalloc(Word size)
{
    if (size > sizeof(code) - code_size) unexpected("developer output bound");
    void *result = code + code_size;
    code_size += size;
    return result;
}
void *TempMallocAlign(Word size) { return TempMalloc(size); }
void TempMemorySetPos(unsigned char *position)
{
    if (position < code || position > code + code_size) unexpected("developer rewind range");
    code_size = (Word)(position - code);
    ++rewinds;
}
__attribute__((regparm(3))) void EmitOpcode(Word opcode, Word change, Word call)
{
    if (request_count == 16 || (opcode != 0x16 && opcode != 0x17) || change || call)
        unexpected("developer opcode profile");
    requests[request_count++] = opcode;
    *(unsigned char *)TempMalloc(1) = (unsigned char)opcode;
}
void CompileError(Word source, const char *message)
{
    if (source != 7 || !message) unexpected("developer diagnostic profile");
    ++diagnostics; /* This selected branch returns directly after the service. */
}
void Com_Error(void) { unexpected("developer fatal diagnostic"); }
static void check_name(Word id)
{
    if (id < 1 || id > 3) unexpected("developer modeled name");
}
void SL_AddRefToString(Word id) { check_name(id); ++adds; }
void SL_RemoveRefToString(Word id)
{
    check_name(id);
    if (drops == 4) unexpected("developer modeled drop bound");
    dropped_names[drops++] = id;
}
Word SL_TransferToCanonicalString(Word id) { check_name(id); ++canonical; return id; }
const char *SL_ConvertToString(Word id) { (void)id; unexpected("developer uninitialized name"); }
/* Function discard mode must return without any of these name/link services. */
#define UNAVAILABLE(name) void name(void) { unexpected(#name); }
UNAVAILABLE(AddOpcodePos)
UNAVAILABLE(FindVariable)
UNAVAILABLE(GetNewVariable)
UNAVAILABLE(GetValueType)
UNAVAILABLE(GetVariable)
UNAVAILABLE(SGetObjectA)
UNAVAILABLE(SL_TransferRefToUser)
UNAVAILABLE(Scr_CreateCanonicalFilename)
UNAVAILABLE(Scr_EvalVariableExtern)
UNAVAILABLE(SetNewVariableValue)
UNAVAILABLE(SetVariableValue)
#undef UNAVAILABLE
__attribute__((regparm(3))) Word Scr_FindLocalVarIndex(Word, Word, Word, Word *);
__attribute__((regparm(3))) void EmitDeveloperStatementList(Word *, Word, Word *, Word **);
__attribute__((regparm(2))) void EmitFunction(Word *, Word);
__attribute__((regparm(3))) void EmitStatement(Word *statement, Word last, Word source, Word *block)
{
    if (bodies == 8 || last || source) unexpected("developer body flags");
    modes[bodies++] = *compile_mode();
    *source_position() = 999;
    if (statement[0] == 2) (void)Scr_FindLocalVarIndex(statement[1], 0, 1, block);
    else if (statement[0] == 0x1b) block[0] = 3; /* Modeled abort, not native return emission. */
    else unexpected("developer body statement");
}
