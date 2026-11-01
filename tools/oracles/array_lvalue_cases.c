/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Actual local-slot reference evaluator; ref decrement is modeled, no unwinding/GC. */
static void autovivification_case(void)
{
    reset_pool();
    unsigned int owner = AllocObject();
    VariableValueInternal *slot = child(owner, 1);
    unsigned int local = variable_pool[slot->v.index + VARIABLELIST_CHILD_BEGIN].hash.id;
    assert(VAR_TYPE(slot) == VAR_UNDEFINED);
    unsigned int array = Scr_EvalArrayRef(local);
    assert(VAR_TYPE(slot) == VAR_POINTER && slot->u.u.pointerValue == array);
    assert(Scr_GetObjectType(array) == VAR_ARRAY && GetArraySize(array) == 0);
    assert(Scr_EvalArrayRef(local) == array && object_count == 2);
    puts("A 1 0 2");
}

static void detach_case(unsigned int count, unsigned int aliases)
{
    reset_pool();
    ref_removals = 0;
    unsigned int owner = AllocObject();
    unsigned int original = Scr_AllocArray();
    unsigned int nested = Scr_AllocArray();
    unsigned int shared = AllocObject();
    unsigned int original_local = new_local(owner, 1, original);
    for (unsigned int i = 0; i < count; ++i) {
        set_word(original, GetInternalVariableIndex(i), VAR_INTEGER, 100 + i);
        set_word(nested, GetInternalVariableIndex(i), VAR_INTEGER, 200 + i);
    }
    set_word(original, 100, VAR_POINTER, nested);
    set_word(original, 101, VAR_POINTER, shared);
    unsigned int locals[3];
    assert(aliases <= 3);
    for (unsigned int i = 0; i < aliases; ++i) {
        locals[i] = new_local(owner, i + 2, original);
        AddRefToObject(original);
    }
    for (unsigned int i = 0; i < aliases; ++i) {
        unsigned int detached = Scr_EvalArrayRef(locals[i]);
        assert(detached != original && Scr_GetObjectType(detached) == VAR_ARRAY);
        assert(variable_pool[locals[i] + VARIABLELIST_CHILD_BEGIN].u.u.pointerValue == detached);
        assert(variable_pool[original + VARIABLELIST_PARENT_BEGIN].u.o.refCount == aliases - i - 1);
        assert(object_count == 4 + 2 * (i + 1));
        unsigned int new_nested = child(detached, 100)->u.u.pointerValue;
        assert(new_nested != nested && Scr_GetObjectType(new_nested) == VAR_ARRAY);
        assert(child(detached, 101)->u.u.pointerValue == shared);
        assert(variable_pool[shared + VARIABLELIST_PARENT_BEGIN].u.o.refCount == i + 1);
        assert(Scr_EvalArrayRef(locals[i]) == detached);
        assert(object_count == 4 + 2 * (i + 1) && ref_removals == i + 1);
        printf("W %u %u %u", count, aliases, i);
        show_keys(original);
        printf(" /%u", GetArraySize(detached));
        show_keys(detached);
        printf(" /%u", GetArraySize(new_nested));
        show_keys(new_nested);
        puts("");
        if (count) {
            child(detached, GetInternalVariableIndex(0))->u.u.intValue = 7;
            child(new_nested, GetInternalVariableIndex(0))->u.u.intValue = 8;
            assert(child(original, GetInternalVariableIndex(0))->u.u.intValue == 100);
            assert(child(nested, GetInternalVariableIndex(0))->u.u.intValue == 200);
        }
    }
    assert(Scr_EvalArrayRef(original_local) == original);
    assert(ref_removals == aliases && gScrVarPub.totalObjectRefCount == aliases);
}

static void rejected_local(unsigned int type)
{
    reset_pool();
    unsigned int owner = AllocObject();
    unsigned int object = AllocObject();
    set_word(owner, 1, type, type == VAR_POINTER ? object : 0);
    rejected_slot = variable_pool[FindVariableIndexInternal(owner, 1)
                                     + VARIABLELIST_CHILD_BEGIN].hash.id;
    rejected_type = type;
    ref_removals = error_calls = error_category = 0;
    (void)Scr_EvalArrayRef(rejected_slot);
    abort();
}

static void lvalue_cases(void)
{
    autovivification_case();
    for (unsigned int width = 0; width <= 6; ++width)
        for (unsigned int aliases = 0; aliases <= 3; ++aliases)
            detach_case(width, aliases);
    detach_case(64, 3);
}
