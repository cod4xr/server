/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Bounded reference/error services. Selected unlink/free helper paths execute from source. */
static void SL_RemoveRefToString(unsigned int label)
{
    assert(label > 0 && label < SL_MAX_STRING_INDEX);
    ++string_removals;
}

static _Noreturn void validate_error(void)
{
    assert(rejected_slot > 0 && error_calls == 1);
    if (error_profile == LOCAL_TYPE) {
        assert(rejected_type > 0);
        assert(gScrVarPub.error_index == 1 && !ref_removals);
        assert(object_count == 2 && record_count == 1);
        assert(VAR_TYPE((&variable_pool[rejected_slot + VARIABLELIST_CHILD_BEGIN])) == rejected_type);
        printf("E %u %u %u\n", rejected_type, error_category, error_calls);
    } else if (error_profile == CLEAR_UNDEFINED) {
        assert(error_category == 1 && gScrVarPub.error_index == 1 && !ref_removals);
        assert(object_count == 1 && record_count == 1);
        assert(VAR_TYPE((&variable_pool[rejected_slot + VARIABLELIST_CHILD_BEGIN])) == VAR_UNDEFINED);
        puts("N 1 1 0");
    } else if (error_profile == CLEAR_INDEX) {
        assert(gScrVarPub.error_index == 0 && ref_removals == 1 && object_count == 3);
        assert(record_count == 8 && !string_removals);
        assert(error_category == (rejected_type == VAR_INTEGER ? 4u : 5u));
        unsigned int detached = variable_pool[rejected_slot + VARIABLELIST_CHILD_BEGIN].u.u.pointerValue;
        assert(detached != rejected_original && GetArraySize(detached) == 3);
        assert(GetArraySize(rejected_original) == 3);
        printf("T %u %08" PRIx32 " %u", rejected_type, rejected_word, error_category);
        show_keys(rejected_original);
        printf(" /");
        show_keys(detached);
        puts("");
    } else abort();
    exit(EXIT_SUCCESS);
}
