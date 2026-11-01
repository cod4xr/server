/* SPDX-License-Identifier: AGPL-3.0-or-later */
static void if_emission_case(Word scenario)
{
    setup(); reset_emission();
    Word *body = scenario >= 3 && scenario <= 5 ? pair(assignment(2), assignment(4)) :
        assignment(scenario == 6 ? 1 : 2);
    Word prior = scenario == 2 || scenario == 4 ? 2 : scenario == 5 ? 4 : 0;
    if (prior) Scr_CalcLocalVarsStatement(assignment(prior), parent);
    Word *statement = branch(body, 0);
    Scr_CalcLocalVarsStatement(statement, parent);
    if (scenario == 1) Scr_CalcLocalVarsStatement(assignment(2), parent);
    if (prior) (void)Scr_FindLocalVarIndex(prior, 0, 1, parent);
    Word *child = (Word *)statement[4];
    text("I "); decimal(scenario); text("\n");
    show("P", parent); show("B", child);
    EmitIfStatement((Word *)statement[1], body, 0, 0, 0, parent, &child);
    show("R", parent); show("F", child); show_emission();
}
static void if_else_emission_case(Word scenario)
{
    setup(); reset_emission();
    Word *yes = scenario == 2 ? pair(assignment(2), assignment(4)) : assignment(2);
    Word *no = scenario == 2 ? pair(assignment(4), assignment(2)) :
        assignment(scenario == 1 || scenario == 3 ? 4 : scenario == 4 ? 1 : 2);
    if (scenario == 3) Scr_CalcLocalVarsStatement(assignment(4), parent);
    Word *statement = branch(yes, no);
    Scr_CalcLocalVarsStatement(statement, parent);
    if (scenario == 3) (void)Scr_FindLocalVarIndex(4, 0, 1, parent);
    Word *then_block = (Word *)statement[6], *else_block = (Word *)statement[7];
    text("E "); decimal(scenario); text("\n");
    show("P", parent); show("B", then_block); show("D", else_block);
    EmitIfElseStatement((Word *)statement[1], yes, no, 0, 0, 0, 0,
        parent, &then_block, &else_block);
    show("R", parent); show("F", then_block); show("G", else_block); show_emission();
}
static void transfer_case(Word continuing, Word created, Word target_public)
{
    setup(); reset_emission();
    Word target[70];
    for (Word i = 0; i < 70; ++i) target[i] = 0;
    target[2] = target_public;
    parent[1] = parent[3] = created;
    for (Word i = 2; i < created; ++i) parent[6 + i] = i + 2;
    scrCompileGlob[0x2c] = scrCompileGlob[0x34] = 1;
    Word pointer = (Word)target;
    memcpy(scrCompileGlob + 0x44, &pointer, sizeof(pointer));
    text("T "); decimal(continuing); text(" "); decimal(created);
    text(" "); decimal(target_public); text("\n");
    if (continuing) EmitContinueStatement(0, parent);
    else EmitBreakStatement(0, parent);
    show("R", parent); show_emission();
}
void scope_start(void)
{
    for (Word scenario = 0; scenario < 7; ++scenario) if_emission_case(scenario);
    for (Word scenario = 0; scenario < 5; ++scenario) if_else_emission_case(scenario);
    for (Word continuing = 0; continuing < 2; ++continuing)
        for (Word created = 2; created <= 5; ++created)
            for (Word target_public = 0; target_public <= 2; ++target_public)
                transfer_case(continuing, created, target_public);
    scope_exit(0);
}
