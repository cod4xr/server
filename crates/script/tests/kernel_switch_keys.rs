// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests explicit normalized switch domains, owned labels and complete operand proof."
)]
use crate::{
    kernel_call_support::{self, integer},
    kernel_switch_support,
};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelInstruction,
    KernelSwitchComparison as Comparison, KernelSwitchKey as Key, KernelValue,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};
use cod4x_script::legacy::LegacyNumber;

fn number(value: i32) -> KernelValue {
    KernelValue::Number(LegacyNumber::Integer(value))
}

#[test]
fn integer_and_string_categories_do_not_coerce_or_need_fabricated_heaps() {
    let labels = [b"1".to_vec()];
    for (value, key, equal, less) in [
        (1, Key::Integer(1), 1, 0),
        (0, Key::Integer(1), 0, 1),
        (2, Key::Integer(1), 0, 0),
        (i32::MIN, Key::Integer(i32::MAX), 0, 1),
        (i32::MAX, Key::Integer(i32::MIN), 0, 0),
        (1, Key::StringLabel(0), 0, 1),
    ] {
        for (comparison, expected) in [(Comparison::Equal, equal), (Comparison::Less, less)] {
            let module = kernel_switch_support::module(key, comparison, &labels);
            assert_eq!(
                module
                    .execute(0, &[number(value)], KernelCallBudget::default())
                    .expect("Numeric matching uses no heap")
                    .value,
                number(expected)
            );
        }
    }
}

#[test]
fn complete_string_bytes_keep_case_prefix_nul_and_high_byte_identity_without_label_allocations() {
    for (label, input, equal, less) in [
        (b"".as_slice(), b"".as_slice(), 1, 0),
        (b"a", b"a", 1, 0),
        (b"A", b"a", 0, 0),
        (b"ab", b"a", 0, 1),
        (b"a", b"ab", 0, 0),
        (b"a\0b", b"a\0c", 0, 0),
        (b"a\0b", b"a\0", 0, 1),
        (b"\xff", b"\x80", 0, 1),
    ] {
        for (comparison, expected) in [(Comparison::Equal, equal), (Comparison::Less, less)] {
            let labels = [label.to_vec()];
            let module = kernel_switch_support::module(Key::StringLabel(0), comparison, &labels);
            let mut strings = StringHeap::new(StringHeapBudget {
                max_live_strings: 1,
                max_slots: 1,
                ..StringHeapBudget::default()
            })
            .expect("One input slot");
            let id = strings
                .allocate(input, HeapWorkBudget::default())
                .expect("Input")
                .id;
            let before = strings.usage();
            let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default());
            let result = module
                .execute_with_heap(
                    0,
                    &[KernelValue::String(id)],
                    KernelCallBudget::default(),
                    &mut context,
                )
                .expect("Case data requires no heap string");
            assert_eq!(result.value, number(expected));
            assert_eq!(context.heap_usage(), before);
            assert_eq!(
                (
                    context.usage().collections,
                    context.usage().object_collections
                ),
                (0, 0)
            );
        }
    }
}

#[test]
fn string_inputs_remain_distinct_from_numeric_case_words() {
    for comparison in [Comparison::Equal, Comparison::Less] {
        let module = kernel_switch_support::module(Key::Integer(1), comparison, &[]);
        let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
        let id = strings
            .allocate(b"1", HeapWorkBudget::default())
            .expect("Input")
            .id;
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default());
        assert_eq!(
            module
                .execute_with_heap(
                    0,
                    &[KernelValue::String(id)],
                    KernelCallBudget::default(),
                    &mut context
                )
                .expect("String category sorts after integers")
                .value,
            number(0)
        );
    }
}

#[test]
fn complete_proof_rejects_unreachable_and_wide_invalid_label_operands() {
    for label in [1, usize::MAX] {
        let code = [
            integer(0),
            KernelInstruction::Return,
            KernelInstruction::SwitchCompare {
                key: Key::StringLabel(label),
                comparison: Comparison::Equal,
            },
        ];
        let labels = [b"PrivateLabel".to_vec()];
        let mut definition = kernel_call_support::definition(&code, 0, 0);
        definition.switch_labels = &labels;
        let error = VerifiedKernelProgram::verify_definition(
            definition,
            KernelVerificationBudget::default(),
        )
        .expect_err("Unreachable operands still require complete proof");
        assert_eq!(error.instruction, Some(2));
        assert_eq!(
            error.kind,
            KernelErrorKind::InvalidSwitchLabel {
                label: label as u64
            }
        );
        assert!(!format!("{error:?} {error}").contains("PrivateLabel"));
    }
}

#[test]
fn label_storage_is_owned_separate_from_literals_and_debug_redacts_key_constants() {
    let code = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::RequireSwitchKey,
        KernelInstruction::SwitchCompare {
            key: Key::StringLabel(0),
            comparison: Comparison::Equal,
        },
        KernelInstruction::Return,
    ];
    let mut labels = [b"PrivateLabel".to_vec()];
    let mut definition = kernel_call_support::definition(&code, 1, 1);
    definition.switch_labels = &labels;
    let proof =
        VerifiedKernelProgram::verify_definition(definition, KernelVerificationBudget::default())
            .expect("Owned table");
    labels[0].fill(b'x');
    assert_eq!(
        (proof.usage().string_literals, proof.usage().switch_labels),
        (0, 1)
    );
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let id = strings
        .allocate(b"PrivateLabel", HeapWorkBudget::default())
        .expect("Input")
        .id;
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default());
    assert_eq!(
        proof
            .execute_with_heap(
                &[KernelValue::String(id)],
                cod4x_script::execution::KernelExecutionBudget::default(),
                &mut context
            )
            .expect("Input edits do not change proof")
            .value,
        number(1)
    );
    assert!(!format!("{proof:?} {:?}", Key::Integer(314_159)).contains("314159"));
    assert!(!format!("{proof:?}").contains("PrivateLabel"));
}
