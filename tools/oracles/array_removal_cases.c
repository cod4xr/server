/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Actual clear/set/remove helpers; source/VM branch selection is modeled, not executed. */
static void removal_case(unsigned int shared, unsigned int string_key, unsigned int key,
                         unsigned int dynamic_undefined)
{
    reset_pool();
    ref_removals = 0;
    unsigned int owner = AllocObject();
    unsigned int array = Scr_AllocArray();
    unsigned int local = new_local(owner, 1, array);
    if (shared) {
        (void)new_local(owner, 2, array);
        AddRefToObject(array);
    }
    for (unsigned int key = 0; key < 3; ++key)
        set_word(array, string_key ? key + 100 : GetInternalVariableIndex(key), VAR_INTEGER, key);
    unsigned int missing = key == 7;
    unsigned int name = string_key ? key + 100 : GetInternalVariableIndex(key);
    VariableValue index = {.type = string_key ? VAR_STRING : VAR_INTEGER,
                           .u.stringValue = string_key ? key + 100 : key};
    unsigned int before_records = record_count;
    if (dynamic_undefined) {
        /* Guarded VM paths create the lvalue first, then remove its index for Undefined. */
        unsigned int target = Scr_EvalArrayRef(local);
        unsigned int slot = GetVariableIndexInternal(target, name);
        if (string_key) SL_RemoveRefToString(index.u.stringValue);
        RemoveVariableValue(target, slot);
    } else ClearArray(local, &index);
    unsigned int target = variable_pool[local + VARIABLELIST_CHILD_BEGIN].u.u.pointerValue;
    assert(GetArraySize(target) == (missing ? 3u : 2u));
    assert(GetArraySize(array) == (shared ? 3u : GetArraySize(target)));
    assert(!FindVariableIndexInternal(target, name));
    assert(ref_removals == shared && object_count == 2 + shared);
    assert(record_count == before_records + 3 * shared + (dynamic_undefined && missing));
    unsigned int expected_removals = string_key
        ? (dynamic_undefined || !missing ? 2u : 1u) : 0u;
    assert(string_removals == expected_removals);
    assert(peak_array_size == 3u + (dynamic_undefined && missing));
    printf("D %u %u %u %u %u %u %u", shared, string_key, key, dynamic_undefined,
           peak_array_size, string_removals, record_count - before_records);
    show_keys(target);
    puts("");
}

static void setter_undefined_case(void)
{
    reset_pool();
    unsigned int array = Scr_AllocArray();
    unsigned int index = GetArrayVariableIndex(array, 7);
    unsigned int id = variable_pool[index + VARIABLELIST_CHILD_BEGIN].hash.id;
    VariableValue value = {.type = VAR_INTEGER, .u.intValue = 42};
    SetVariableValue(id, &value);
    assert(Scr_EvalVariable(id).u.intValue == 42);
    value.type = VAR_UNDEFINED;
    value.u.intValue = 0;
    SetVariableValue(id, &value);
    assert(Scr_EvalVariable(id).type == VAR_UNDEFINED);
    assert(GetArraySize(array) == 1 && FindVariableIndexInternal(array, GetInternalVariableIndex(7)));
    printf("V %u %u", VAR_TYPE((&variable_pool[id + VARIABLELIST_CHILD_BEGIN])), GetArraySize(array));
    show_keys(array);
    RemoveVariableValue(array, index);
    assert(GetArraySize(array) == 0);
    puts(" /0");
}

static void dynamic_undefined_parent_case(void)
{
    reset_pool();
    unsigned int owner = AllocObject();
    VariableValueInternal *slot = child(owner, 1);
    unsigned int local = variable_pool[slot->v.index + VARIABLELIST_CHILD_BEGIN].hash.id;
    unsigned int array = Scr_EvalArrayRef(local);
    unsigned int index = GetArrayVariableIndex(array, 0);
    assert(GetArraySize(array) == 1);
    RemoveVariableValue(array, index);
    assert(VAR_TYPE(slot) == VAR_POINTER && GetArraySize(array) == 0);
    assert(object_count == 2 && peak_array_size == 1);
    puts("B 2 1 0");
}

static void free_list_case(void)
{
    reset_pool();
    unsigned int array = Scr_AllocArray();
    for (unsigned int i = 0; i < 3; ++i)
        set_word(array, GetInternalVariableIndex(i), VAR_INTEGER, i);
    static const unsigned int keys[] = {2, 0, 1};
    for (unsigned int i = 0; i < 3; ++i)
        RemoveVariableValue(array, FindVariableIndexInternal(array, GetInternalVariableIndex(keys[i])));
    assert(!GetArraySize(array) && !gScrVarPub.numScriptValues && !gScrVarPub.totalObjectRefCount);
    unsigned int next = variable_pool[VARIABLELIST_CHILD_BEGIN].u.next;
    unsigned int previous = 0;
    unsigned int free_count = 0;
    while (next) {
        assert(next <= record_count && free_count < 3);
        assert(variable_pool[next + VARIABLELIST_CHILD_BEGIN].hash.u.prev == previous);
        unsigned int id = variable_pool[next + VARIABLELIST_CHILD_BEGIN].hash.id;
        assert(variable_pool[id + VARIABLELIST_CHILD_BEGIN].w.status == VAR_STAT_FREE);
        previous = next;
        next = variable_pool[id + VARIABLELIST_CHILD_BEGIN].u.next;
        ++free_count;
    }
    assert(free_count == 3);
    set_word(array, GetInternalVariableIndex(10), VAR_INTEGER, 7);
    assert(GetArraySize(array) == 1 && gScrVarPub.numScriptValues == 1);
    assert(gScrVarPub.totalObjectRefCount == 1 && record_count == 4);
    printf("F %u %u", free_count, GetArraySize(array));
    show_keys(array);
    puts("");
}

static void rejected_undefined_clear(void)
{
    reset_pool();
    ref_removals = error_calls = 0;
    unsigned int owner = AllocObject();
    VariableValueInternal *slot = child(owner, 1);
    rejected_slot = variable_pool[slot->v.index + VARIABLELIST_CHILD_BEGIN].hash.id;
    error_profile = CLEAR_UNDEFINED;
    VariableValue index = {.type = VAR_INTEGER, .u.intValue = 0};
    ClearArray(rejected_slot, &index);
    abort();
}

static void rejected_clear_index(unsigned int type, unsigned int word)
{
    reset_pool();
    ref_removals = error_calls = 0;
    unsigned int owner = AllocObject();
    rejected_original = Scr_AllocArray();
    rejected_slot = new_local(owner, 1, rejected_original);
    (void)new_local(owner, 2, rejected_original);
    AddRefToObject(rejected_original);
    for (unsigned int i = 0; i < 3; ++i)
        set_word(rejected_original, GetInternalVariableIndex(i), VAR_INTEGER, i);
    rejected_type = type;
    rejected_word = word;
    error_profile = CLEAR_INDEX;
    VariableValue index = {.type = (int)type, .u.stringValue = word};
    ClearArray(rejected_slot, &index);
    abort();
}

static void removal_cases(void)
{
    for (unsigned int shared = 0; shared <= 1; ++shared)
        for (unsigned int string_key = 0; string_key <= 1; ++string_key)
            for (unsigned int key = 0; key <= 3; ++key)
                for (unsigned int dynamic = 0; dynamic <= 1; ++dynamic)
                    removal_case(shared, string_key, key == 3 ? 7 : key, dynamic);
    setter_undefined_case();
    dynamic_undefined_parent_case();
    free_list_case();
}
