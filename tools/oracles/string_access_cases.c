/* SPDX-License-Identifier: AGPL-3.0-or-later */
static void index_cases(unsigned int label)
{
    static const uint32_t words[] = {0x80000000, 0xfffffffe, 0xffffffff, 0, 1, 2, 3, 4,
                                   5, 6, 65535, 65536, 69999, 70000, 0x7fffffff};
    static const unsigned int types[] = {
        VAR_UNDEFINED, VAR_INTEGER, VAR_FLOAT, VAR_STRING, VAR_ISTRING, VAR_VECTOR, VAR_FUNCTION
    };
    for (unsigned int t = 0; t < sizeof types / sizeof types[0]; ++t) {
        for (unsigned int i = 0; i < sizeof words / sizeof words[0]; ++i) {
            reset_services();
            VariableValue object = {.type = VAR_STRING, .u.stringValue = label};
            VariableValue index = {.type = (int)types[t], .u.stringValue = words[i]};
            Scr_EvalArray(&object, &index);
            assert(gScrVarPub.error_index == 0);
            assert(script_error <= 2 && removals == (script_error == 0));
            assert(creations == (script_error == 0));
            assert(conversions <= 1);
            printf("R %u %u %08x %u", label, types[t], words[i], script_error);
            if (!script_error) {
                assert(index.type == VAR_STRING && index.u.stringValue > 0x100
                       && index.u.stringValue <= 0x1ff);
                printf(" %02x", index.u.stringValue - 0x100);
            } else {
                assert(index.type == (int)types[t] && index.u.stringValue == words[i]);
            }
            puts("");
        }
    }
}

static void size_cases(unsigned int label)
{
    static const unsigned int types[] = {
        VAR_UNDEFINED, VAR_INTEGER, VAR_FLOAT, VAR_STRING, VAR_ISTRING, VAR_VECTOR, VAR_FUNCTION
    };
    for (unsigned int i = 0; i < sizeof types / sizeof types[0]; ++i) {
        reset_services();
        VariableValue value = {.type = (int)types[i], .u.stringValue = label};
        Scr_EvalSizeValue(&value);
        assert(gScrVarPub.error_index == 0 && removals == 1 && creations == 0);
        assert(script_error == 0 || script_error == 3);
        printf("Z %u %u %u", label, types[i], script_error);
        if (!script_error) {
            assert(value.type == VAR_INTEGER && conversions == 1);
            printf(" %d", value.u.intValue);
        } else {
            assert(value.type == VAR_UNDEFINED && conversions == 0 && removed_type == types[i]);
        }
        puts("");
    }
}

int main(void)
{
    memset(large_string, 'z', sizeof large_string - 1);
    for (unsigned int label = 1; label <= sizeof texts / sizeof texts[0]; ++label) {
        index_cases(label);
        size_cases(label);
    }
    return 0;
}
