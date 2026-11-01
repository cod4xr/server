/* SPDX-License-Identifier: AGPL-3.0-or-later */
int main(void)
{
    static const uint16_t sizes[] = {0, 1, 2, 32768, UINT16_MAX};
    static const unsigned int flags[] = {0, VAR_STAT_EXTERNAL, 0x80000000u};
    for (unsigned int type = VAR_THREAD; type <= VAR_ENDON_LIST; ++type) {
        for (unsigned int f = 0; f < sizeof flags / sizeof flags[0]; ++f) {
            for (unsigned int s = 0; s < sizeof sizes / sizeof sizes[0]; ++s) {
                unsigned int word = type | flags[f];
                VariableValueInternal *parent = &gScrVarGlob.variableList[1 + VARIABLELIST_PARENT_BEGIN];
                memset(parent, 0, sizeof *parent);
                parent->w.type = word;
                parent->u.o.u.size = sizes[s];
                removals = 0;
                VariableValue value = {.type = VAR_POINTER, .u.intValue = 1};
                Scr_EvalSizeValue(&value);
                assert(value.type == VAR_INTEGER && removals == 1);
                printf("O %08x %u %d\n", word, sizes[s], value.u.intValue);
            }
        }
    }
    return 0;
}
