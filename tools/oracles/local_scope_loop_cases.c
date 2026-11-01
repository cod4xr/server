/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Nonconstant conditions avoid unavailable native value/string services. */
static void loop_cases(void)
{
    for (Word later = 0; later < 2; ++later) {
        setup(); trace(10, while_loop(assignment(2)), later);
        setup(); trace(11, while_loop(pair(assignment(2), node(0x3f))), later);
        setup(); trace(12, while_loop(pair(assignment(2), node(0x40))), later);
        setup(); trace(13, while_loop(branch(assignment(2), assignment(2))), later);
        setup(); trace(14, while_loop(branch(pair(assignment(2), node(0x3f)),
            pair(assignment(4), node(0x40)))), later);
        setup(); trace(15, for_loop(assignment(2), assignment(4)), later);
        setup(); trace(16, for_loop(pair(assignment(2), node(0x40)), assignment(4)), later);
        setup(); trace(17, for_loop(pair(assignment(2), node(0x3f)), assignment(4)), later);
        setup(); trace(18, while_loop(while_loop(pair(assignment(2), node(0x3f)))), later);
    }
}
void scope_start(void)
{
    branch_cases();
    loop_cases();
    scope_exit(0);
}
