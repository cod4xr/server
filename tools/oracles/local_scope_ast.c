/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Synthetic AST/name IDs, not the native parser, bytecode VM or runtime lifetimes. */
static Word nodes[128][16], node_count;
static Word parent[70];

static Word *node(Word tag)
{
    if (node_count == 128) unexpected("scope AST bound");
    Word *result = nodes[node_count++];
    result[0] = tag;
    return result;
}
static Word *name(Word id) { Word *result = node(4); result[1] = id; return result; }
static Word *expression(Word id)
{
    Word *value = node(0x11); value[1] = (Word)name(id);
    Word *result = node(6); result[1] = (Word)value;
    return result;
}
static Word *assignment(Word id)
{
    Word *result = node(2); result[1] = (Word)name(id); result[2] = (Word)expression(1);
    return result;
}
static Word *list(Word **statements, Word count)
{
    if (count > 32) unexpected("scope AST list bound");
    Word *head = 0;
    while (count) {
        Word *link = node(0); link[0] = (Word)statements[--count]; link[1] = (Word)head;
        head = link;
    }
    Word *root = node(0); root[1] = (Word)head;
    Word *handle = node(0); handle[0] = (Word)root;
    return handle;
}
static Word *pair(Word *first, Word *second)
{
    Word *statements[] = {first, second};
    Word *result = node(0x2c); result[1] = (Word)list(statements, 2);
    return result;
}
static Word *branch(Word *yes, Word *no)
{
    Word *result = node(no ? 0x26 : 0x25);
    result[1] = (Word)expression(3); result[2] = (Word)yes;
    result[3] = (Word)no;
    return result;
}
static void setup(void)
{
    reset(); node_count = 0;
    for (Word i = 0; i < 128; ++i) for (Word j = 0; j < 16; ++j) nodes[i][j] = 0;
    for (Word i = 0; i < 70; ++i) parent[i] = 0;
    parent[1] = parent[2] = parent[3] = 2;
    parent[4] = 3;
    parent[6] = 1; parent[7] = 3;
}
