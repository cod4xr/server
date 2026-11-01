/* SPDX-License-Identifier: AGPL-3.0-or-later */
static Word scalar_drops;
void RemoveRefToValue(Word type, Word value)
{
    (void)value;
    if (type != 0 && type != 5 && type != 6) unexpected("scope constant non-scalar release");
    ++scalar_drops;
}
void Scr_Error(const char *message) { (void)message; unexpected("scope bool error"); }
const char *va(const char *format, ...) { (void)format; unexpected("scope bool diagnostic"); }
static Word *literal_expression(Word number)
{
    Word *literal = node(7); literal[1] = number;
    Word *result = node(6); result[1] = (Word)literal;
    return result;
}
static void constant_case(Word scenario)
{
    setup(); reset_emission(); scalar_drops = 0;
    Word *body = scenario < 4 ? pair(assignment(2), node(0x3f)) :
        branch(pair(assignment(2), node(0x3f)),
            pair(assignment(4), node(scenario >= 6 ? 0x3f : 0x40)));
    Word for_statement = scenario == 2 || scenario == 3 || scenario == 5 || scenario == 7;
    Word *statement = for_statement ? for_loop(body, assignment(4)) : while_loop(body);
    Word *condition = literal_expression(scenario == 1 || scenario == 3 ? 0 : 1);
    if (for_statement) ((Word *)statement[2])[1] = (Word)condition;
    else statement[1] = (Word)condition;
    Scr_CalcLocalVarsStatement(statement, parent);
    text("K "); decimal(scenario); text("\n"); show("P", parent);
    if (scenario != 1 && scenario != 3) EmitStatement(statement, 0, 0, parent);
    show("R", parent);
    for (Word i = 0; i < allocations; ++i)
        if (sizes[i] == 0x118) show("B", blocks[i]);
    show_emission(); text("Q "); decimal(conditions); text("\n");
    text("D "); decimal(scalar_drops); text("\n");
}
void scope_start(void)
{
    for (Word scenario = 0; scenario < 8; ++scenario) constant_case(scenario);
    scope_exit(0);
}
