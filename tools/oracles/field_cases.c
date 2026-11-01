/* SPDX-License-Identifier: AGPL-3.0-or-later */
int main(void)
{
    const int tags[] = { VAR_UNDEFINED, VAR_POINTER, VAR_STRING, VAR_ISTRING,
                         VAR_FLOAT, VAR_INTEGER, VAR_FUNCTION };
    const uint32_t words[] = { 0, 1, 0x80000000u, 0xffffffffu, 0x7fffffffu,
                               0x3f800000u, 0x80000001u, 0x7fc00001u };
    for (unsigned int tag = 0; tag < sizeof tags / sizeof *tags; ++tag) {
        for (unsigned int word = 0; word < sizeof words / sizeof *words; ++word) {
            if ((tags[tag] == VAR_POINTER || tags[tag] == VAR_STRING || tags[tag] == VAR_ISTRING)
                && word != 1) continue;
            memset(&gScrVarGlob, 0, sizeof gScrVarGlob);
            present = add_calls = remove_calls = lookup_calls = 0;
            VariableValueInternal *parent = &gScrVarGlob.variableList[VARIABLELIST_PARENT_BEGIN + 1];
            parent->w.type = VAR_STAT_EXTERNAL | VAR_OBJECT;
            VariableValueInternal *child = &gScrVarGlob.variableList[VARIABLELIST_CHILD_BEGIN + 1];
            child->w.type = VAR_STAT_EXTERNAL | VAR_UNDEFINED;
            assert(Scr_GetVariableFieldIndex(1, 101) == 1);
            VariableValue input = {0};
            input.type = tags[tag];
            memcpy(&input.u.intValue, &words[word], sizeof words[word]);
            SetVariableValue(1, &input);
            VariableValue output = Scr_FindVariableField(1, 101);
            uint32_t result;
            memcpy(&result, &output.u.intValue, sizeof result);
            assert(output.type == tags[tag] && result == words[word]);
            assert(add_calls == 1 && remove_calls == 1);
            VariableValue missing = Scr_FindVariableField(1, 102);
            assert(missing.type == VAR_UNDEFINED);
            VariableValue scratch[2];
            memset(scratch, 0, sizeof scratch);
            ClearVariableField(1, 101, scratch);
            assert(!present);
            output = Scr_FindVariableField(1, 101);
            assert(output.type == VAR_UNDEFINED);
            ClearVariableField(1, 101, scratch);
            assert(!present && lookup_calls == 5);
            printf("F %d %08" PRIx32 " %d %08" PRIx32 "\n", tags[tag], words[word],
                   tags[tag], result);
        }
    }
    return 0;
}
