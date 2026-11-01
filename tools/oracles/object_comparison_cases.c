/* SPDX-License-Identifier: AGPL-3.0-or-later */
int main(void)
{
    const unsigned int child_tags[] = { VAR_UNDEFINED, VAR_INTEGER, VAR_FUNCTION };
    for (int evaluate = 0; evaluate < 2; ++evaluate) {
        for (unsigned int marker = 0; marker < sizeof child_tags / sizeof *child_tags; ++marker) {
            for (unsigned int left = 1; left <= 3; ++left) {
                for (unsigned int right = 1; right <= 3; ++right) {
                    for (unsigned int unequal = 0; unequal < 2; ++unequal) {
                        memset(&gScrVarGlob, 0, sizeof gScrVarGlob);
                        for (unsigned int id = 1; id <= 3; ++id) {
                            gScrVarGlob.variableList[VARIABLELIST_PARENT_BEGIN + id].w.type =
                                VAR_STAT_EXTERNAL | VAR_OBJECT;
                            gScrVarGlob.variableList[VARIABLELIST_CHILD_BEGIN + id].w.type =
                                VAR_STAT_EXTERNAL | child_tags[marker];
                        }
                        gScrVarPub.evaluate = evaluate;
                        removals = removed_ids = 0;
                        VariableValue first = {0}, second = {0};
                        first.type = second.type = VAR_POINTER;
                        first.u.intValue = (int)left;
                        second.u.intValue = (int)right;
                        if (unequal) Scr_EvalInequality(&first, &second);
                        else Scr_EvalEquality(&first, &second);
                        unsigned int expected = unequal ? left != right : left == right;
                        assert(first.type == VAR_INTEGER && first.u.intValue == (int)expected);
                        assert(second.type == VAR_POINTER && second.u.intValue == (int)right);
                        assert(removals == 2 && removed_ids == ((1u << left) | (1u << right)));
                        printf("O %d %u %u %u %u %u\n", evaluate, child_tags[marker], unequal,
                               left, right, expected);
                    }
                }
            }
        }
    }
    return 0;
}
