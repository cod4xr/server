/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Native slot/if/transfer helpers execute; opcode output/body dispatch is modeled. */
static unsigned char code[4096];
static Word code_size, event_count, positions;
static Word event_offsets[128];

void *TempMalloc(Word size)
{
    if (size > sizeof(code) - code_size) unexpected("scope code bound");
    void *result = code + code_size;
    code_size += size;
    return result;
}
void *TempMallocAlign(Word size) { return TempMalloc(size); }
__attribute__((regparm(3))) void EmitOpcode(Word opcode, Word change, Word call)
{
    if (event_count == 128 || opcode > 255 || call != 0 ||
        change != (opcode == 0x5e ? (Word)-1 : 0)) unexpected("scope opcode profile");
    event_offsets[event_count++] = code_size;
    *(unsigned char *)TempMalloc(1) = (unsigned char)opcode;
}
void AddOpcodePos(Word source, Word breakpoint)
{
    if (source > 10 || breakpoint > 1) unexpected("scope source position");
    ++positions;
}
void SL_AddRefToString(Word id) { if (id < 1 || id > 5) unexpected("scope string add"); }
void SL_RemoveRefToString(Word id) { if (id < 1 || id > 5) unexpected("scope string drop"); }
Word SL_TransferToCanonicalString(Word id)
{
    if (id < 1 || id > 5) unexpected("scope canonical ID");
    return id;
}
const char *SL_ConvertToString(Word id)
{
    (void)id; unexpected("scope unsupported diagnostic");
}
__attribute__((regparm(3))) Word EmitOrEvalExpression(Word *value, void *result, Word *block)
{
    if (value[0] != 6 || !result || !block) unexpected("scope condition profile");
    return 0;
}
void EmitValue(void) { unexpected("scope constant emission"); }
__attribute__((regparm(3))) Word Scr_FindLocalVarIndex(Word, Word, Word, Word *);
__attribute__((regparm(3))) void EmitIfStatement(Word *, Word *, Word, Word, Word, Word *, Word **);
__attribute__((regparm(3))) void EmitIfElseStatement(
    Word *, Word *, Word *, Word, Word, Word, Word, Word *, Word **, Word **);
__attribute__((regparm(2))) void EmitBreakStatement(Word, Word *);
__attribute__((regparm(2))) void EmitContinueStatement(Word, Word *);

static void emit_assignment(Word *statement, Word *block)
{
    if (statement[0] != 2) unexpected("scope body statement profile");
    Word *target = (Word *)statement[1];
    if (target[0] != 4) unexpected("scope body target profile");
    (void)Scr_FindLocalVarIndex(target[1], 0, 1, block);
}
__attribute__((regparm(3))) void EmitStatement(Word *statement, Word last, Word source, Word *block)
{
    if (last || source > 10) unexpected("scope body flags");
    if (statement[0] == 2) emit_assignment(statement, block);
    else if (statement[0] == 0x2c) {
        Word *handle = (Word *)statement[1], *root = (Word *)handle[0];
        Word *link = (Word *)root[1];
        Word count = 0;
        while (link) {
            if (++count > 2) unexpected("scope body list bound");
            emit_assignment((Word *)link[0], block);
            link = (Word *)link[1];
        }
    } else unexpected("scope unsupported body");
}
static void reset_emission(void)
{
    code_size = event_count = positions = 0;
    for (Word i = 0; i < sizeof(code); ++i) code[i] = 0;
}
static void show_emission(void)
{
    text("O");
    for (Word i = 0; i < event_count; ++i) {
        Word offset = event_offsets[i], opcode = code[offset];
        text(" "); decimal(opcode);
        if (opcode == 0x17) { text(":"); decimal(code[offset + 1]); }
        if (opcode == 0x16) {
            if (offset + 2 >= code_size) unexpected("scope local operand bound");
            text(":"); decimal(code[offset + 1] + 256 * code[offset + 2]);
        }
    }
    text(" p:"); decimal(positions); text("\n");
}
