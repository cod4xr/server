/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Actual helper execution; pool/lookup/refs/output services are the explicit test model. */
static void key_cases(void)
{
    static const uint32_t words[] = {
        0, 1, 2, 65535, 65536, 0x7ffffe, 0x7fffff, 0x800000,
        0xffffff, 0x7fffffff, 0x80000000, 0xfffffffe, 0xffffffff
    };
    for (unsigned int i = 0; i < sizeof words / sizeof words[0]; ++i) {
        unsigned int word = words[i];
        int valid = IsValidArrayIndex(word);
        printf("I %08x %d", word, valid);
        if (valid) {
            unsigned int name = GetInternalVariableIndex(word);
            VariableValue value = Scr_GetArrayIndexValue(name);
            assert(value.type == VAR_INTEGER && value.u.stringValue == word);
            printf(" %06x %d %08x", name, value.type, value.u.stringValue);
        }
        puts("");
    }
    static const unsigned int names[] = {1, 2, 0xffff, 0x10000, 0x10001, 0x17ffd,
        0x17ffe, 0x7fffff, 0x800000, 0x800001, 0xfffffe, 0xffffff};
    for (unsigned int i = 0; i < sizeof names / sizeof names[0]; ++i) {
        VariableValue value = Scr_GetArrayIndexValue(names[i]);
        printf("K %06x %d %08x\n", names[i], value.type, value.u.stringValue);
    }
}

static void copy_case(unsigned int count, unsigned int nested_count, unsigned int rotation)
{
    reset_pool();
    unsigned int original = Scr_AllocArray();
    unsigned int nested = Scr_AllocArray();
    unsigned int shared = AllocObject();
    for (unsigned int i = 0; i < nested_count; ++i)
        set_word(nested, GetInternalVariableIndex(i), VAR_INTEGER, 0x80000000u + i);
    for (unsigned int i = 0; i < count; ++i) {
        unsigned int key = (i + rotation) % count;
        unsigned int name = key % 2 ? key + 1 : GetInternalVariableIndex(key);
        set_word(original, name, VAR_INTEGER, key);
    }
    set_word(original, 100, VAR_POINTER, nested);
    set_word(original, 101, VAR_POINTER, shared);
    set_word(original, 102, VAR_STRING, 103);
    set_word(original, 104, VAR_ISTRING, 105);
    set_word(original, 106, VAR_FLOAT, 0x80000000);
    unsigned int duplicate = Scr_AllocArray();
    unsigned int refs_before = string_refs;
    CopyArray(original, duplicate);
    unsigned int copied_nested = child(duplicate, 100)->u.u.pointerValue;
    assert(copied_nested != nested && Scr_GetObjectType(copied_nested) == VAR_ARRAY);
    assert(child(duplicate, 101)->u.u.pointerValue == shared);
    assert(variable_pool[shared + VARIABLELIST_PARENT_BEGIN].u.o.refCount == 1);
    assert(variable_pool[nested + VARIABLELIST_PARENT_BEGIN].u.o.refCount == 0);
    assert(child(duplicate, 102)->u.u.stringValue == 103);
    assert(child(duplicate, 104)->u.u.stringValue == 105);
    assert(child(duplicate, 106)->u.u.stringValue == 0x80000000);
    assert(!vector_refs && gScrVarPub.totalObjectRefCount == gScrVarPub.numScriptValues + 1);
    assert(gScrVarPub.varUsagePos == NULL);
    printf("C %u %u %u %u %u", count, nested_count, rotation, object_count,
           string_refs - refs_before);
    show_keys(original);
    printf(" /%u", GetArraySize(duplicate));
    show_keys(duplicate);
    printf(" /%u", GetArraySize(nested));
    show_keys(nested);
    printf(" /%u", GetArraySize(copied_nested));
    show_keys(copied_nested);
    puts("");

    /* Mutation of the nested copy cannot mutate the old array. Struct identity stays shared. */
    child(copied_nested, GetInternalVariableIndex(0))->u.u.intValue = 7;
    if (nested_count)
        assert(child(nested, GetInternalVariableIndex(0))->u.u.stringValue == 0x80000000);
    variable_pool[shared + VARIABLELIST_PARENT_BEGIN].u.o.u.entnum = 77;
    assert(variable_pool[child(original, 101)->u.u.pointerValue + VARIABLELIST_PARENT_BEGIN]
               .u.o.u.entnum == 77);

    unsigned int repeated = Scr_AllocArray();
    CopyArray(duplicate, repeated);
    assert(child(repeated, 101)->u.u.pointerValue == shared);
    assert(variable_pool[shared + VARIABLELIST_PARENT_BEGIN].u.o.refCount == 2);
    assert(gScrVarPub.totalObjectRefCount == gScrVarPub.numScriptValues + 2);
    printf("R %u %u %u", count, nested_count, rotation);
    show_keys(repeated);
    puts("");
}

static void replacement_case(void)
{
    reset_pool();
    unsigned int array = Scr_AllocArray();
    for (unsigned int i = 0; i < 3; ++i) {
        unsigned int index = GetArrayVariableIndex(array, i);
        unsigned int id = GetArrayVariable(array, i);
        assert(variable_pool[index + VARIABLELIST_CHILD_BEGIN].hash.id == id);
        set_word(array, GetInternalVariableIndex(i), VAR_INTEGER, i);
    }
    child(array, GetInternalVariableIndex(1))->u.u.intValue = 77;
    assert(record_count == 3 && GetArraySize(array) == 3);
    printf("U");
    show_keys(array);
    puts("");
}

int main(int argc, char **argv)
{
    if (argc == 2) {
        if (strcmp(argv[1], "clear-undefined") == 0) rejected_undefined_clear();
        char *end;
        unsigned long tag = strtoul(argv[1], &end, 10);
        assert(*argv[1] && !*end && tag >= 1 && tag <= VAR_INCLUDE_CODEPOS && tag != VAR_STACK);
        rejected_local((unsigned int)tag);
        abort();
    }
    if (argc == 4 && strcmp(argv[1], "clear-index") == 0) {
        char *end;
        unsigned long tag = strtoul(argv[2], &end, 10);
        assert(*argv[2] && !*end && tag <= VAR_FUNCTION && tag != VAR_STRING);
        unsigned long word = strtoul(argv[3], &end, 16);
        assert(*argv[3] && !*end && word <= UINT32_MAX);
        rejected_clear_index((unsigned int)tag, (unsigned int)word);
        abort();
    }
    assert(argc == 1);
    key_cases();
    replacement_case();
    for (unsigned int count = 0; count <= 6; ++count)
        for (unsigned int nested = 0; nested <= 3; ++nested)
            for (unsigned int rotation = 0; rotation < (count ? count : 1); ++rotation)
                copy_case(count, nested, rotation);
    copy_case(64, 64, 63);
    lvalue_cases();
    removal_cases();
    return 0;
}
