/* SPDX-License-Identifier: AGPL-3.0-or-later */
static Word *case_label(Word value)
{
    Word *literal = node(7); literal[1] = value;
    Word *result = node(0x3d); result[1] = (Word)literal;
    return result;
}
static Word *switch_statement(Word scenario)
{
    Word *first = assignment(2), *second = assignment(
        scenario == 1 || scenario == 3 || scenario == 5 || scenario == 8 ? 4 : 2);
    if (scenario == 6) first = branch(assignment(2), assignment(2));
    if (scenario == 7) first = pair(assignment(2), assignment(4));
    if (scenario == 8) first = pair(assignment(2), node(0x40));
    if (scenario == 9)
        first = branch(pair(assignment(2), node(0x3f)),
                       pair(assignment(2), node(0x3f)));
    if (scenario == 10) first = branch(pair(assignment(2), node(0x3f)), 0);
    Word *statements[8]; Word count = 0;
    statements[count++] = case_label(1);
    statements[count++] = first;
    if (scenario != 3 && scenario != 8 && scenario != 9 && scenario != 10)
        statements[count++] = node(0x3f);
    if (scenario == 4) statements[count++] = case_label(2);
    if (scenario != 0) {
        statements[count++] = node(0x3e);
        statements[count++] = second;
        statements[count++] = node(0x3f);
    }
    Word *result = node(0x3c);
    result[1] = (Word)expression(3); result[2] = (Word)list(statements, count);
    return result;
}
static Word *string_switch_statement(Word scenario)
{
    Word *statements[12]; Word count = 0;
    Word arms = scenario == 11 ? 1 : 2;
    for (Word i = 0; i < arms; ++i) {
        Word *label = case_label(i + 1);
        Word *literal = (Word *)label[1];
        if (!(scenario == 12 && i == 0)) {
            literal[0] = 0xb;
            literal[1] = scenario == 12 ? 1 : scenario == 14 ? 2 - i : i + 1;
        }
        statements[count++] = label;
        statements[count++] = assignment(2);
        statements[count++] = node(0x3f);
    }
    statements[count++] = node(0x3e);
    statements[count++] = assignment(2);
    statements[count++] = node(0x3f);
    Word *result = node(0x3c);
    result[1] = (Word)expression(3); result[2] = (Word)list(statements, count);
    return result;
}
static void switch_case(Word scenario)
{
    setup(); reset_emission();
    sorted_count = transfer_count = 0;
    if (scenario == 5) Scr_CalcLocalVarsStatement(assignment(2), parent);
    Word *statement = scenario >= 11 ? string_switch_statement(scenario) : switch_statement(scenario);
    if (scenario == 8) {
        Word *loop = node(0x27);
        loop[1] = (Word)expression(3); loop[2] = (Word)statement;
        statement = loop;
    }
    Scr_CalcLocalVarsStatement(statement, parent);
    if (scenario == 5) (void)Scr_FindLocalVarIndex(2, 0, 1, parent);
    text("S "); decimal(scenario); text("\n"); show("P", parent);
    EmitStatement(statement, 0, 0, parent);
    show("R", parent);
    for (Word i = 0; i < allocations; ++i)
        if (sizes[i] == 0x118) show("B", blocks[i]);
    show_emission(); text("Q "); decimal(conditions); text("\n");
    if (scenario < 11 && transfer_count) unexpected("integer switch string transfer");
    if (scenario >= 11) {
        text("J");
        for (Word i = 0; i < sorted_count; ++i) { text(" "); decimal(sorted_keys[i]); }
        text("\nT");
        for (Word i = 0; i < transfer_count; ++i) { text(" "); decimal(transferred_strings[i]); }
        text("\n");
    }
}
void scope_start(void)
{
    for (Word scenario = 0; scenario < 15; ++scenario) switch_case(scenario);
    scope_exit(0);
}
