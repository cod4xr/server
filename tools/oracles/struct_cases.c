/* SPDX-License-Identifier: AGPL-3.0-or-later */
int main(void)
{
    for (unsigned int debug = 0; debug < 2; ++debug) {
        memset(&gScrVarGlob, 0, sizeof gScrVarGlob);
        memset(&debug_state, 0, sizeof debug_state);
        gScrVarDebugPub = debug ? &debug_state : NULL;
        allocated = gScrVarPub.totalObjectRefCount = 0;
        for (unsigned int sequence = 1; sequence <= RECORD_LIMIT; ++sequence) {
            memset(&output, 0, sizeof output);
            output_calls = 0;
            Scr_AddStruct();
            unsigned int id = output.u.pointerValue;
            assert(id == sequence && output_calls == 1 && output.type == VAR_POINTER);
            VariableValueInternal *parent = &gScrVarGlob.variableList[id + VARIABLELIST_PARENT_BEGIN];
            unsigned int debug_refs = debug_state.leakCount[id + VARIABLELIST_PARENT_BEGIN];
            assert(VAR_TYPE(parent) == VAR_OBJECT && parent->u.o.refCount == 0);
            assert(parent->nextSibling == 0 && debug_refs == debug);
            assert(gScrVarPub.totalObjectRefCount == sequence);
            printf("S %u %u %u %u %u %u %u %u\n", debug, sequence, output.type,
                   VAR_TYPE(parent), parent->u.o.refCount, gScrVarPub.totalObjectRefCount,
                   debug_refs, output_calls);
        }
    }
    return 0;
}
