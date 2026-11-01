/* SPDX-License-Identifier: AGPL-3.0-or-later */
static void loop_case(Word scenario)
{
    setup(); reset_emission();
    Word *body = scenario == 0 ? assignment(2) :
        scenario <= 2 ? pair(assignment(2), node(scenario == 1 ? 0x3f : 0x40)) :
        scenario == 3 ? branch(assignment(2), assignment(2)) :
        scenario == 4 ? branch(pair(assignment(2), node(0x3f)),
            pair(assignment(4), node(0x40))) :
        scenario <= 7 || scenario == 9 ? pair(assignment(2), node(scenario == 6 ? 0x40 : 0x3f)) :
        while_loop(pair(assignment(2), node(0x3f)));
    if (scenario == 5) body = assignment(2);
    Word *statement = (scenario >= 5 && scenario <= 7) || scenario == 9 ?
        for_loop(body, assignment(4)) : while_loop(body);
    if (scenario == 9) statement[2] = (Word)node(0);
    Scr_CalcLocalVarsStatement(statement, parent);
    text("L "); decimal(scenario); text("\n");
    show("P", parent);
    EmitStatement(statement, 0, 0, parent);
    show("R", parent);
    for (Word i = 0; i < allocations; ++i)
        if (sizes[i] == 0x118) show("B", blocks[i]);
    show_emission();
    text("Q "); decimal(conditions); text("\n");
}
void scope_start(void)
{
    for (Word scenario = 0; scenario < 10; ++scenario) loop_case(scenario);
    scope_exit(0);
}
