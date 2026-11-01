/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Actual local analysis only; developer emitters/VM/error unwinding do not run. */
static Word *developer_list(Word *body)
{
    Word *statements[] = {body};
    Word *result = node(0x2d);
    result[1] = (Word)list(statements, 1);
    result[2] = 7;
    return result;
}
static Word *developer_scenario(Word scenario)
{
    Word *body = assignment(2);
    if (scenario == 1) body = pair(assignment(2), assignment(4));
    if (scenario == 2) body = branch(assignment(2), assignment(2));
    if (scenario == 3) body = branch(assignment(2), assignment(4));
    if (scenario == 4) body = pair(assignment(2), node(0x1c));
    if (scenario == 5) body = developer_list(assignment(2));
    if (scenario >= 6) body = pair(assignment(2), node(scenario % 2 ? 0x40 : 0x3f));
    Word *developer = developer_list(body);
    if (scenario == 6 || scenario == 7) return while_loop(developer);
    if (scenario == 8 || scenario == 9) return for_loop(developer, assignment(4));
    return developer;
}
__attribute__((noreturn)) void scope_start(void)
{
    for (Word scenario = 0; scenario < 10; ++scenario) {
        for (Word later = 0; later < 2; ++later) {
            setup();
            Word *statement = developer_scenario(scenario);
            Scr_CalcLocalVarsStatement(statement, parent);
            if (later) Scr_CalcLocalVarsStatement(assignment(2), parent);
            text("D "); decimal(scenario); text(" "); decimal(later); text("\n");
            show("R", parent);
            for (Word i = 0; i < allocations; ++i) {
                if (sizes[i] == 0x118) show("B", blocks[i]);
                else { text("A "); decimal(sizes[i]); text("\n"); }
            }
        }
    }
    scope_exit(0);
}
