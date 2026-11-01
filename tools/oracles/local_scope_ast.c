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
static Word *pair(Word *first, Word *second)
{
    Word *tail = node(0); tail[0] = (Word)second;
    Word *head = node(0); head[0] = (Word)first; head[1] = (Word)tail;
    Word *root = node(0); root[1] = (Word)head;
    Word *handle = node(0); handle[0] = (Word)root;
    Word *result = node(0x2c); result[1] = (Word)handle;
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
