// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Source switches must match decoded byte labels across distinct key domains."
)]
use crate::{
    array_read_support::number, source_array_write_support::State, source_scope_support::compile,
};
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};

#[test]
fn mixed_source_labels_keep_integer_words_plain_strings_empty_strings_and_defaults_distinct() {
    let module = compile(b"main(k){switch(k){case \"2\":return 2;case 2:return 20;case \"\":return 3;case 0:return 30;default:return 99;}}");
    assert_eq!(
        (module.usage().string_literals, module.usage().switch_labels),
        (0, 2)
    );
    for (key, expected) in [(2, 20), (0, 30), (-1, 99)] {
        assert_eq!(
            module
                .program()
                .execute(0, &[number(key)], KernelCallBudget::default())
                .expect("String labels do not fabricate a heap")
                .value,
            number(expected)
        );
    }
    let mut state = State::new(ObjectHeapBudget::default());
    for (bytes, expected) in [(b"2".as_slice(), 2), (b"", 3), (b"other", 99)] {
        let id = state
            .strings
            .allocate(bytes, HeapWorkBudget::default())
            .expect("Input")
            .id;
        assert_eq!(
            state
                .run(
                    &module,
                    &[KernelValue::String(id)],
                    KernelCallBudget::default(),
                    HeapWorkBudget::default()
                )
                .0
                .expect("Mixed dispatch")
                .value,
            number(expected)
        );
    }
}

#[test]
fn source_label_decoding_and_case_sensitive_byte_identity_match_independent_heap_inputs() {
    let module = compile(br#"main(k){switch(k){case "a\"b":return 42;case "Beta":return 12;case "beta":return 13;default:return 99;}}"#);
    let mut state = State::new(ObjectHeapBudget::default());
    for (bytes, expected) in [
        (b"a\"b".as_slice(), 42),
        (b"Beta", 12),
        (b"beta", 13),
        (b"BETA", 99),
    ] {
        let id = state
            .strings
            .allocate(bytes, HeapWorkBudget::default())
            .expect("Decoded input")
            .id;
        assert_eq!(
            state
                .run(
                    &module,
                    &[KernelValue::String(id)],
                    KernelCallBudget::default(),
                    HeapWorkBudget::default()
                )
                .0
                .expect("Decoded source labels")
                .value,
            number(expected)
        );
    }
}

#[test]
fn default_only_empty_and_unmatched_string_switches_validate_then_release_their_key() {
    for (source, expected) in [
        (b"main(){switch(\"key\"){default:return 7;}}".as_slice(), 7),
        (b"main(){switch(\"key\"){}return 9;}", 9),
        (b"main(){switch(\"key\"){case 1:return 3;}return 9;}", 9),
        (
            b"main(){switch(\"key\"){case \"key\":return 42;default:return 0;}}",
            42,
        ),
    ] {
        let mut state = State::new(ObjectHeapBudget::default());
        assert_eq!(
            state
                .run(
                    &compile(source),
                    &[],
                    KernelCallBudget::default(),
                    HeapWorkBudget::default()
                )
                .0
                .expect("Valid plain-string key")
                .value,
            number(expected)
        );
        assert_eq!(state.objects.usage().live_array_values, 0);
    }
}
