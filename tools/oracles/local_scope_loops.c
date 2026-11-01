/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* The optional for condition is tag 0x41 around an expression, or tag zero. */
static Word *while_loop(Word *body)
{
    Word *result = node(0x27);
    result[1] = (Word)expression(3); result[2] = (Word)body;
    return result;
}
static Word *for_loop(Word *body, Word *update)
{
    Word *result = node(0x28);
    Word *condition = node(0x41); condition[1] = (Word)expression(3);
    result[1] = (Word)assignment(5); result[2] = (Word)condition;
    result[3] = (Word)update; result[4] = (Word)body;
    return result;
}
