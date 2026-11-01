/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Explicit planned tables and synthetic lists, not parser or native local analysis. */
static void clear(volatile unsigned char *bytes, Word size)
{
    for (Word i = 0; i < size; ++i) bytes[i] = 0;
}
static void show_block(const char *label, const Word *block)
{
    if (block[3] > 8) unexpected("developer trace width");
    text(label);
    for (Word i = 0; i < 6; ++i) { text(" "); decimal(block[i]); }
    for (Word i = 0; i < block[3]; ++i) { text(" n:"); decimal(block[6 + i]); }
    text("\n");
}
static void developer_case(Word scenario, Word enabled, Word nesting)
{
    clear(scrCompileGlob, sizeof(scrCompileGlob));
    clear(gScrCompilePub, sizeof(gScrCompilePub));
    clear(gScrVarPub, sizeof(gScrVarPub));
    clear(code, sizeof(code));
    code_size = request_count = bodies = rewinds = diagnostics = adds = drops = canonical = 0;
    Word parent[70], child[70];
    clear((unsigned char *)parent, sizeof(parent));
    clear((unsigned char *)child, sizeof(child));
    parent[1] = scenario == 3 ? 1 : 2;
    parent[2] = 2; parent[3] = 2; parent[4] = scenario == 3 ? 1 : 3;
    parent[6] = 1; parent[7] = 3;
    if (scenario != 0) {
        child[3] = 3; child[6] = 1; child[7] = 3; child[8] = 2;
    }
    Word assignment[] = {2, scenario == 2 ? 1 : scenario == 3 ? 3 : 2};
    Word aborted[] = {0x1b, 0};
    Word link[] = {(Word)(scenario == 4 ? aborted : assignment), 0};
    Word root[] = {0, scenario == 0 ? 0 : (Word)link};
    Word list[] = {(Word)root};
    Word *child_pointer = child;
    *compile_mode() = nesting;
    *source_position() = 123;
    gScrVarPub[7] = (unsigned char)enabled;
    /* Sentinel output must survive disabled-body rewind unchanged. */
    *(unsigned char *)TempMalloc(1) = 0xaa;
    EmitDeveloperStatementList(list, 7, parent, &child_pointer);
    text("C "); decimal(scenario); text(" "); decimal(enabled); text(" "); decimal(nesting); text("\n");
    show_block("P", parent); show_block("B", child);
    text("G "); decimal(*compile_mode()); text(" "); decimal(*source_position());
    text(" "); decimal(diagnostics); text(" "); decimal(rewinds);
    text(" "); decimal(adds); text(" "); decimal(drops); text(" "); decimal(canonical); text("\n");
    text("M"); for (Word i = 0; i < bodies; ++i) { text(" "); decimal(modes[i]); } text("\n");
    text("O"); for (Word i = 0; i < request_count; ++i) { text(" "); decimal(requests[i]); } text("\n");
    text("W"); for (Word i = 0; i < code_size; ++i) { text(" "); decimal(code[i]); } text("\n");
}
static void discarded_function(Word external, Word borrowed)
{
    clear(scrCompileGlob, sizeof(scrCompileGlob));
    clear(gScrCompilePub, sizeof(gScrCompilePub));
    clear(code, sizeof(code));
    drops = adds = canonical = diagnostics = request_count = 0;
    code_size = 1; code[0] = 0xaa;
    *compile_mode() = 2;
    *(Word *)(gScrCompilePub + 4) = 100;
    scrCompileGlob[0x1c] = (unsigned char)borrowed;
    Word function[] = {external ? 0x15 : 0x14, 2, 3};
    EmitFunction(function, 7);
    text("F "); decimal(external); text(" "); decimal(borrowed); text("\n");
    text("G "); decimal(*compile_mode()); text(" "); decimal(*(Word *)(gScrCompilePub + 4));
    text(" "); decimal(diagnostics); text(" "); decimal(adds); text(" "); decimal(canonical); text("\n");
    text("T"); for (Word i = 0; i < drops; ++i) { text(" "); decimal(dropped_names[i]); } text("\n");
    text("W"); for (Word i = 0; i < code_size; ++i) { text(" "); decimal(code[i]); } text("\n");
}
__attribute__((noreturn)) void scope_start(void)
{
    for (Word scenario = 0; scenario < 5; ++scenario)
        for (Word enabled = 0; enabled < 2; ++enabled) developer_case(scenario, enabled, 0);
    for (Word nesting = 1; nesting < 3; ++nesting) developer_case(1, 1, nesting);
    for (Word external = 0; external < 2; ++external)
        for (Word borrowed = 0; borrowed < 2; ++borrowed) discarded_function(external, borrowed);
    scope_exit(0);
}
