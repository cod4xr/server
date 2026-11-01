// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests distinguish validated immutable aliases from real content comparisons."
)]
use crate::kernel_heap_support::{self, allocate};
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelHeapContext, KernelInstruction, KernelResource,
    KernelValue,
};
use cod4x_script::heap::HeapWorkBudget;
use cod4x_script::legacy::{LegacyComparison, LegacyNumber};

#[test]
fn validated_large_string_aliases_compare_without_scanning_payload_in_either_category() {
    let program = kernel_heap_support::binary(KernelInstruction::Compare(LegacyComparison::Equal));
    let mut heap = kernel_heap_support::heap(2);
    let bytes = vec![0xff; 1024 * 1024];
    let left = allocate(&mut heap, &bytes);
    let right = allocate(&mut heap, &bytes);
    assert_ne!(left, right);
    for value in [
        KernelValue::String(left),
        KernelValue::LocalizedString(left),
    ] {
        let mut context = KernelHeapContext::new(
            &mut heap,
            &[],
            HeapWorkBudget {
                max_steps: 6,
                ..HeapWorkBudget::default()
            },
        );
        let result = program
            .execute_with_heap(
                &[value, value],
                KernelExecutionBudget::default(),
                &mut context,
            )
            .expect("Checked alias needs no payload visits");
        assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(1)));
        assert_eq!(context.usage().steps, 6);
    }
    for arguments in [
        [KernelValue::String(left), KernelValue::String(right)],
        [
            KernelValue::LocalizedString(left),
            KernelValue::LocalizedString(right),
        ],
    ] {
        let mut context = KernelHeapContext::new(
            &mut heap,
            &[],
            HeapWorkBudget {
                max_steps: 6,
                ..HeapWorkBudget::default()
            },
        );
        let error = program
            .execute_with_heap(&arguments, KernelExecutionBudget::default(), &mut context)
            .expect_err("Distinct handles still require a content comparison");
        assert!(matches!(
            error.kind,
            KernelErrorKind::BudgetExceeded {
                resource: KernelResource::HeapWorkSteps,
                maximum: 6,
                ..
            }
        ));
        assert_eq!(context.usage().steps, 6);
        assert_eq!(context.heap_usage().live_strings, 2);
    }
}
