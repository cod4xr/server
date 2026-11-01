/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Noncolliding storage adapter keeps hash IDs distinct from data IDs. */
unsigned int AllocVariable(void)
{
    assert(object_count + 1 < OBJECT_LIMIT);
    unsigned int id = ++object_count;
    VariableValueInternal *entry = &variable_pool[id + VARIABLELIST_PARENT_BEGIN];
    assert(entry->w.status == 0);
    entry->v.next = (uint16_t)id;
    return id;
}

unsigned int FindVariableIndexInternal(unsigned int parent, unsigned int name)
{
    for (unsigned int i = 0; i < record_count; ++i)
        if (records[i].parent == parent && records[i].name == name)
            return records[i].index;
    return 0;
}

unsigned int FindVariableIndexInternal2(unsigned int name, unsigned int hash)
{
    unsigned int result = 0;
    for (unsigned int i = 0; i < record_count; ++i) {
        if (records[i].name == name && records[i].initial_hash == hash) {
            assert(result == 0);
            result = records[i].index;
        }
    }
    return result;
}

unsigned int GetNewVariableIndexInternal3(unsigned int parent, unsigned int name, unsigned int hash)
{
    assert(parent > 0 && parent <= object_count && record_count < RECORD_LIMIT);
    assert(name != 0 && name <= VAR_NAME_LOW_MASK);
    assert(!FindVariableIndexInternal(parent, name));
    unsigned int index = record_count + 1;
    unsigned int id = index + 0x8000;
    records[record_count].parent = parent;
    records[record_count].name = name;
    records[record_count].index = index;
    records[record_count++].initial_hash = hash;
    variable_pool[index + VARIABLELIST_CHILD_BEGIN].hash.id = (uint16_t)id;
    VariableValueInternal *entry = &variable_pool[id + VARIABLELIST_CHILD_BEGIN];
    entry->w.status = (name << VAR_NAME_BITS) | VAR_STAT_HEAD;
    entry->v.index = (uint16_t)index;
    VariableValueInternal *owner = &variable_pool[parent + VARIABLELIST_PARENT_BEGIN];
    if (VAR_TYPE(owner) == VAR_ARRAY) {
        ++owner->u.o.u.size;
        VariableValue key = Scr_GetArrayIndexValue(name);
        AddRefToValue(key.type, key.u);
    }
    return index;
}

static void reset_pool(void)
{
    memset(variable_pool, 0, sizeof variable_pool);
    memset(records, 0, sizeof records);
    memset(&gScrVarPub, 0, sizeof gScrVarPub);
    record_count = object_count = string_refs = vector_refs = captured_count = captured_pending = 0;
}

static VariableValueInternal *child(unsigned int parent, unsigned int name)
{
    unsigned int index = GetVariableIndexInternal(parent, name);
    unsigned int id = variable_pool[index + VARIABLELIST_CHILD_BEGIN].hash.id;
    assert(id > 0 && id < VARIABLELIST_CHILD_SIZE);
    return &variable_pool[id + VARIABLELIST_CHILD_BEGIN];
}

static void set_word(unsigned int parent, unsigned int name, unsigned int type, uint32_t bits)
{
    VariableValueInternal *entry = child(parent, name);
    assert(VAR_TYPE(entry) == VAR_UNDEFINED);
    entry->w.type |= type;
    entry->u.u.stringValue = bits;
}

static void show_keys(unsigned int parent)
{
    Scr_AddArrayKeys(parent);
    assert(captured_count == GetArraySize(parent));
    unsigned int sibling = FindFirstSibling(parent);
    for (unsigned int i = 0; i < captured_count; ++i) {
        assert(sibling != 0);
        printf(" %c:%d", captured_keys[i].type == VAR_STRING ? 's' : 'i',
               captured_keys[i].u.intValue);
        sibling = FindNextSibling(sibling);
    }
    assert(!sibling && !captured_pending);
}
