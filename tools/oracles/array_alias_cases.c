/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Modeled root/field ownership setup; actual selected add/remove/COW/key helpers. */
static void cycle_case(unsigned int holders, unsigned int count)
{
    reset_pool();
    unsigned int root_owner = AllocObject();
    unsigned int array = Scr_AllocArray();
    set_word(root_owner, 100, VAR_POINTER, array);
    unsigned int index = FindVariableIndexInternal(root_owner, 100);
    unsigned int slot = variable_pool[index + VARIABLELIST_CHILD_BEGIN].hash.id;
    for (unsigned int key = 0; key < count; ++key)
        set_word(array, GetInternalVariableIndex(key), VAR_INTEGER, key);
    for (unsigned int index = 0; index < holders; ++index) {
        unsigned int cycle = AllocObject();
        set_word(cycle, 101, VAR_POINTER, cycle);
        AddRefToObject(cycle);
        set_word(cycle, 102, VAR_POINTER, array);
        AddRefToObject(array);
        VariableValueInternal *record = &variable_pool[cycle + VARIABLELIST_PARENT_BEGIN];
        assert(record->u.o.refCount == 1 && record->nextSibling);
        RemoveRefToObject(cycle);
        assert(record->u.o.refCount == 0 && record->nextSibling);
        assert(child(cycle, 101)->u.u.pointerValue == cycle);
        assert(child(cycle, 102)->u.u.pointerValue == array);
    }
    VariableValueInternal *original = &variable_pool[array + VARIABLELIST_PARENT_BEGIN];
    assert(original->u.o.refCount == holders);
    unsigned int changed = Scr_EvalArrayRef(slot);
    assert(changed != array && child(root_owner, 100)->u.u.pointerValue == changed);
    assert(original->u.o.refCount == holders - 1);
    assert(variable_pool[changed + VARIABLELIST_PARENT_BEGIN].u.o.refCount == 0);
    assert(GetArraySize(array) == count && GetArraySize(changed) == count);
    unsigned int allocations = object_count;
    assert(Scr_EvalArrayRef(slot) == changed && object_count == allocations);
    printf("L %u %u %u %u", holders, count, original->u.o.refCount,
           variable_pool[changed + VARIABLELIST_PARENT_BEGIN].u.o.refCount);
    show_keys(array);
    printf(" |");
    show_keys(changed);
    puts("");
}

int main(void)
{
    for (unsigned int holders = 1; holders <= 4; ++holders)
        for (unsigned int count = 0; count <= 4; ++count)
            cycle_case(holders, count);
    return 0;
}
