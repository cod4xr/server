/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Modeled graph/ownership setup, checked actual helper execution and free lists. */
static unsigned int free_parents(void)
{
    unsigned int index = variable_pool[VARIABLELIST_PARENT_BEGIN].u.next;
    unsigned int previous = 0, count = 0;
    while (index) {
        assert(index <= object_count && count < object_count);
        VariableValueInternal *entry = &variable_pool[index + VARIABLELIST_PARENT_BEGIN];
        assert(entry->w.status == VAR_STAT_FREE && entry->hash.id == index);
        assert(entry->hash.u.prev == previous);
        previous = index;
        index = entry->u.next;
        ++count;
    }
    return count;
}

static unsigned int free_children(void)
{
    unsigned int index = variable_pool[VARIABLELIST_CHILD_BEGIN].u.next;
    unsigned int previous = 0, count = 0;
    while (index) {
        assert(index <= record_count && count < record_count);
        Variable *entry = &variable_pool[index + VARIABLELIST_CHILD_BEGIN].hash;
        VariableValueInternal *value = &variable_pool[entry->id + VARIABLELIST_CHILD_BEGIN];
        assert(value->w.status == VAR_STAT_FREE && entry->u.prev == previous);
        previous = index;
        index = value->u.next;
        ++count;
    }
    return count;
}

static void lifetime_case(unsigned int depth, int cycle_at, unsigned int aliases, unsigned int width)
{
    reset_pool();
    unsigned int writer = AllocObject();
    unsigned int array = Scr_AllocArray();
    set_word(writer, 100, VAR_POINTER, array); /* Transfer array creation owner to writer field. */
    unsigned int index = FindVariableIndexInternal(writer, 100);
    unsigned int slot = variable_pool[index + VARIABLELIST_CHILD_BEGIN].hash.id;
    for (unsigned int key = 0; key < width; ++key)
        set_word(array, GetInternalVariableIndex(key), VAR_INTEGER, key);
    unsigned int nodes[4];
    for (unsigned int index = 0; index < depth; ++index) {
        nodes[index] = AllocObject();
        if (index) {
            set_word(nodes[index - 1], 101, VAR_POINTER, nodes[index]);
            AddRefToObject(nodes[index]);
            RemoveRefToObject(nodes[index]); /* Drop child creation owner. */
        }
        for (unsigned int alias = 0; alias < aliases; ++alias) {
            set_word(nodes[index], 110 + alias, VAR_POINTER, array);
            AddRefToObject(array);
        }
        if ((int)index == cycle_at) {
            set_word(nodes[index], 120, VAR_POINTER, nodes[index]);
            AddRefToObject(nodes[index]);
        }
    }
    RemoveRefToObject(nodes[0]);
    unsigned int freed = cycle_at < 0 ? depth : (unsigned int)cycle_at;
    unsigned int retained = (depth - freed) * aliases;
    unsigned int freed_fields = cycle_at < 0 ? depth * aliases + depth - 1 : freed * (aliases + 1);
    assert(free_parents() == freed);
    assert(free_children() == freed_fields);
    assert(gScrVarPub.numScriptObjects == object_count - freed);
    assert(variable_pool[array + VARIABLELIST_PARENT_BEGIN].u.o.refCount == retained);
    for (unsigned int index = 0; index < depth; ++index) {
        VariableValueInternal *node = &variable_pool[nodes[index] + VARIABLELIST_PARENT_BEGIN];
        assert((node->w.status == VAR_STAT_FREE) == (index < freed));
    }
    unsigned int selected = Scr_EvalArrayRef(slot);
    unsigned int detached = selected != array;
    assert(detached == (retained > 0));
    assert(variable_pool[array + VARIABLELIST_PARENT_BEGIN].u.o.refCount == (retained ? retained - 1 : 0));
    assert(!variable_pool[selected + VARIABLELIST_PARENT_BEGIN].u.o.refCount);
    assert(GetArraySize(selected) == width && GetArraySize(array) == width);
    assert(free_parents() == freed && gScrVarPub.numScriptObjects == object_count - freed);
    assert(free_children() == freed_fields);
    unsigned int allocated = object_count;
    assert(Scr_EvalArrayRef(slot) == selected && object_count == allocated);
    unsigned int excess = 0;
    for (unsigned int id = 1; id <= object_count; ++id)
        if (variable_pool[id + VARIABLELIST_PARENT_BEGIN].w.status != VAR_STAT_FREE)
            excess += variable_pool[id + VARIABLELIST_PARENT_BEGIN].u.o.refCount;
    assert(gScrVarPub.totalObjectRefCount == gScrVarPub.numScriptObjects + gScrVarPub.numScriptValues + excess);
    printf("Q %u %d %u %u %u %u %u %u %u", depth, cycle_at, aliases, width, freed, freed_fields, retained,
           detached, variable_pool[array + VARIABLELIST_PARENT_BEGIN].u.o.refCount);
    show_keys(array);
    printf(" |");
    show_keys(selected);
    puts("");
}

int main(void)
{
    for (unsigned int depth = 1; depth <= 4; ++depth)
        for (int cycle_at = -1; cycle_at < (int)depth; ++cycle_at)
            for (unsigned int aliases = 1; aliases <= 3; ++aliases)
                for (unsigned int width = 0; width <= 4; ++width)
                    lifetime_case(depth, cycle_at, aliases, width);
    return 0;
}
