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
    Word *statements[8]; Word count = 0;
    statements[count++] = case_label(1);
    statements[count++] = first;
    if (scenario != 3 && scenario != 8) statements[count++] = node(0x3f);
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
static void switch_case(Word scenario)
{
    setup(); reset_emission();
    if (scenario == 5) Scr_CalcLocalVarsStatement(assignment(2), parent);
    Word *statement = switch_statement(scenario);
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
}
void scope_start(void)
{
    for (Word scenario = 0; scenario < 9; ++scenario) switch_case(scenario);
    scope_exit(0);
}
