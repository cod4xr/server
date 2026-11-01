/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Function words/nested profiles are helper-only; vector payload reads require native pointer ABI. */
static void setup_read(unsigned int rotation, unsigned int aliases)
{
    reset_pool();
    parent_removals = read_errors = read_category = 0;
    active_parent = Scr_AllocArray();
    unsigned int shared = AllocObject();
    Scr_AllocArray(); /* Reserved modeled nested-data identity for the separate helper profile. */
    unsigned int names[] = { GetInternalVariableIndex(0), GetInternalVariableIndex(2),
        GetInternalVariableIndex(65535), GetInternalVariableIndex(0x7fffff), 11, 12, 13, 65535, 15 };
    static const unsigned int types[] = {VAR_INTEGER, VAR_FLOAT, VAR_UNDEFINED, VAR_STRING,
        VAR_ISTRING, VAR_POINTER, VAR_INTEGER, VAR_INTEGER, VAR_INTEGER};
    uint32_t words[] = {7, 0x80000000u, 0, 101, 102, shared, 13, 0xffffff85u, 17};
    for (unsigned int i = 0; i < 9; ++i) {
        unsigned int item = (i + rotation) % 9;
        set_word(active_parent, names[item], types[item], words[item]);
    }
    for (unsigned int i = 0; i < aliases; ++i) AddRefToObject(active_parent);
    active_aliases = aliases;
    assert(GetArraySize(active_parent) == 9);
}

static void read_case(unsigned int rotation, unsigned int aliases, unsigned int type,
                      uint32_t word, unsigned int profile)
{
    setup_read(rotation, aliases);
    if (profile) {
        assert(type == VAR_STRING && profile <= 2);
        VariableValueInternal *entry = child(active_parent, word);
        entry->w.type = (entry->w.type & ~VAR_MASK) | (profile == 1 ? VAR_CODEPOS : VAR_POINTER);
        entry->u.u.stringValue = profile == 1 ? 0x1234 : 3;
    }
    VariableValue value = {.type = VAR_POINTER, .u.intValue = (int)active_parent};
    VariableValue index = {.type = (int)type, .u.stringValue = word};
    unsigned int found = type == VAR_INTEGER ? FindArrayVariable(active_parent, (int)word)
                                            : FindVariable(active_parent, word);
    if (type == VAR_STRING) SL_AddRefToString(word);
    unsigned int before_strings = string_refs, before_vectors = vector_refs;
    Scr_EvalArray(&value, &index);
    assert(value.type == VAR_POINTER && value.u.stringValue == active_parent);
    assert(parent_removals == 1 && !read_errors && object_count == 3 && record_count == 9);
    assert(variable_pool[active_parent + VARIABLELIST_PARENT_BEGIN].u.o.refCount == aliases - 1);
    unsigned int subtype = index.type == VAR_POINTER ? Scr_GetObjectType(index.u.stringValue) : 0;
    if (index.type == VAR_POINTER)
        assert(variable_pool[index.u.stringValue + VARIABLELIST_PARENT_BEGIN].u.o.refCount == 1);
    printf("R %u %u %u %u %08x %u %d %08x %u %u %u %u %u", profile, rotation, aliases, type, word,
           found != 0, index.type, index.u.stringValue, subtype,
           string_refs - before_strings, vector_refs - before_vectors,
           string_removals, GetArraySize(active_parent));
    show_keys(active_parent);
    puts("");
}

static void size_case(unsigned int rotation, unsigned int aliases)
{
    setup_read(rotation, aliases);
    VariableValue value = {.type = VAR_POINTER, .u.intValue = (int)active_parent};
    Scr_EvalSizeValue(&value);
    assert(value.type == VAR_INTEGER && value.u.intValue == 9 && parent_removals == 1);
    assert(variable_pool[active_parent + VARIABLELIST_PARENT_BEGIN].u.o.refCount == aliases - 1);
    printf("Z %u %u %d %u\n", rotation, aliases, value.u.intValue, parent_removals);
}

static void successful_reads(void)
{
    static const unsigned int aliases[] = {1, 2, 7};
    static const unsigned int keys[][2] = {{VAR_INTEGER,0}, {VAR_INTEGER,2}, {VAR_INTEGER,65535},
        {VAR_INTEGER,0x7fffff}, {VAR_STRING,11}, {VAR_STRING,12}, {VAR_STRING,13},
        {VAR_STRING,65535}, {VAR_STRING,15}, {VAR_INTEGER,1}, {VAR_STRING,14}};
    for (unsigned int rotation = 0; rotation <= 3; rotation += 3) {
        for (unsigned int a = 0; a < sizeof aliases / sizeof aliases[0]; ++a) {
            for (unsigned int key = 0; key < sizeof keys / sizeof keys[0]; ++key)
                read_case(rotation, aliases[a], keys[key][0], keys[key][1], 0);
            read_case(rotation, aliases[a], VAR_STRING, 65535, 1);
            read_case(rotation, aliases[a], VAR_STRING, 15, 2);
            size_case(rotation, aliases[a]);
        }
    }
}

static _Noreturn void validate_read_error(void)
{
    assert(read_errors == 1 && read_category != 0 && parent_removals == 0);
    assert(record_count == 9 && object_count == 3 && GetArraySize(1) == 9);
    assert(variable_pool[active_parent + VARIABLELIST_PARENT_BEGIN].u.o.refCount == active_aliases);
    assert(string_removals == 0 && vector_refs == 0);
    printf("E %u %u %08x %u %d %u\n", Scr_GetObjectType(active_parent), active_key_type,
           active_key_word, read_category, gScrVarPub.error_index, parent_removals);
    exit(0);
}

int main(int argc, char **argv)
{
    if (argc == 1) { successful_reads(); return 0; }
    assert(argc == 4);
    setup_read(0, 1);
    unsigned int ordinary = (unsigned int)strtoul(argv[1], NULL, 0);
    active_key_type = (unsigned int)strtoul(argv[2], NULL, 0);
    active_key_word = (uint32_t)strtoul(argv[3], NULL, 0);
    assert(ordinary <= 1 && active_key_type < VAR_COUNT);
    if (ordinary) {
        active_parent = 2;
        AddRefToObject(active_parent);
    }
    VariableValue value = {.type = VAR_POINTER, .u.intValue = (int)active_parent};
    VariableValue index = {.type = (int)active_key_type, .u.stringValue = active_key_word};
    Scr_EvalArray(&value, &index);
    abort();
}
