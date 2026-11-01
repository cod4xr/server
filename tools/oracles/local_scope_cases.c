/* SPDX-License-Identifier: AGPL-3.0-or-later */
static void trace(Word number, Word *statement, Word later)
{
    Scr_CalcLocalVarsStatement(statement, parent);
    if (later) Scr_CalcLocalVarsStatement(assignment(2), parent);
    text("C "); decimal(number); text(" "); decimal(later); text("\n");
    show("R", parent);
    for (Word i = 0; i < allocations; ++i) {
        if (sizes[i] == 0x118) show("B", blocks[i]);
        else { text("A "); decimal(sizes[i]); text("\n"); }
    }
}
static void branch_cases(void)
{
    for (Word later = 0; later < 2; ++later) {
        setup(); trace(0, branch(assignment(2), 0), later);
        setup(); trace(1, branch(assignment(2), assignment(2)), later);
        setup(); trace(2, branch(assignment(2), assignment(4)), later);
        setup(); trace(3, branch(pair(assignment(2), node(0x1c)), assignment(4)), later);
        setup(); trace(4, branch(assignment(2), pair(assignment(4), node(0x1c))), later);
        setup(); trace(5, pair(assignment(2), assignment(4)), later);
        setup(); trace(6, branch(branch(assignment(2), assignment(2)), assignment(2)), later);
        setup(); trace(7, branch(pair(assignment(2), assignment(4)),
            pair(assignment(4), assignment(2))), later);
    }
}
