// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests check identity, constant metadata work and reference rejection without field scans."
)]
use crate::kernel_call_support::definition;
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelHeapContext, KernelInstruction, KernelResource,
    KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind};
use cod4x_script::legacy::LegacyComparison;

fn program(comparison: LegacyComparison) -> VerifiedKernelProgram {
    VerifiedKernelProgram::verify_definition(
        definition(
            &[
                KernelInstruction::LoadLocal(0),
                KernelInstruction::LoadLocal(1),
                KernelInstruction::Compare(comparison),
                KernelInstruction::Return,
            ],
            2,
            2,
        ),
        KernelVerificationBudget::default(),
    )
    .expect("Checked comparison")
}

#[test]
fn structs_with_identical_70000_field_contents_remain_distinct_at_constant_exact_heap_work() {
    let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
    let left = allocate(&mut objects);
    let right = allocate(&mut objects);
    for index in 0..70_000 {
        let key = format!("f{index}");
        for id in [left, right] {
            objects
                .put(
                    id,
                    key.as_bytes(),
                    number(index),
                    &strings,
                    HeapWorkBudget::default(),
                )
                .expect("Identical large field contents");
        }
    }
    let before = objects.usage();
    let program = program(LegacyComparison::Equal);
    for (arguments, expected) in [
        (
            [KernelValue::Object(left), KernelValue::Object(left)],
            number(1),
        ),
        (
            [KernelValue::Object(left), KernelValue::Object(right)],
            number(0),
        ),
    ] {
        for max_steps in 0..=5 {
            let mut context = KernelHeapContext::new(
                &mut strings,
                &[],
                HeapWorkBudget {
                    max_steps,
                    max_scratch_bytes: 0,
                },
            )
            .with_objects(&mut objects, &[]);
            let result = program.execute_with_heap(
                &arguments,
                KernelExecutionBudget::default(),
                &mut context,
            );
            if max_steps < 5 {
                assert!(
                    matches!(result.expect_err("Every insufficient work prefix").kind, KernelErrorKind::BudgetExceeded {resource: KernelResource::HeapWorkSteps, requested, maximum} if requested > maximum && maximum == max_steps)
                );
            } else {
                assert_eq!(result.expect("Exact constant work").value, expected);
                assert_eq!(context.usage().steps, 5);
            }
            assert_eq!(context.usage().peak_scratch_bytes, 0);
            assert_eq!(context.usage().object_collections, 0);
            assert_eq!(context.object_heap_usage(), Some(before));
        }
    }
}

#[test]
fn foreign_and_reclaimed_or_reused_struct_references_cannot_compare_as_valid_identities() {
    let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
    let dead = allocate(&mut objects);
    objects
        .reclaim(&[], HeapWorkBudget::default())
        .expect("Vacate slot");
    let live = allocate(&mut objects);
    let (_, mut foreign) = stores(ObjectHeapBudget::default());
    let foreign_id = allocate(&mut foreign);
    let equality = program(LegacyComparison::Equal);
    for (id, expected) in [
        (dead, ObjectHeapErrorKind::StaleReference),
        (foreign_id, ObjectHeapErrorKind::ForeignReference),
    ] {
        for arguments in [
            [KernelValue::Object(id), KernelValue::Object(live)],
            [KernelValue::Object(live), KernelValue::Object(id)],
        ] {
            let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
                .with_objects(&mut objects, &[]);
            assert_eq!(
                equality
                    .execute_with_heap(&arguments, KernelExecutionBudget::default(), &mut context)
                    .expect_err("No stale identity shortcut")
                    .kind,
                KernelErrorKind::ObjectHeap(expected)
            );
        }
    }
    for comparison in [
        LegacyComparison::Less,
        LegacyComparison::Greater,
        LegacyComparison::LessOrEqual,
        LegacyComparison::GreaterOrEqual,
    ] {
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
            .with_objects(&mut objects, &[]);
        assert_eq!(
            program(comparison)
                .execute_with_heap(
                    &[KernelValue::Object(live); 2],
                    KernelExecutionBudget::default(),
                    &mut context
                )
                .expect_err("No object ordering")
                .kind,
            KernelErrorKind::UnmatchingTypes
        );
    }
}
