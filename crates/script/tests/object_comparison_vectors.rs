// SPDX-License-Identifier: AGPL-3.0-or-later
//! Ordinary native helper subset with modeled records/ref services; not array/VM parity.
#![allow(
    clippy::expect_used,
    reason = "Tests replay classified ordinary-pointer native identity outcomes."
)]
use crate::kernel_call_support::definition;
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::execution::{
    KernelExecutionBudget, KernelHeapContext, KernelInstruction, KernelValue,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};
use cod4x_script::legacy::LegacyComparison;

#[test]
fn ordinary_pointer_equality_and_inequality_native_helper_results_replay_through_checked_execution()
-> Result<(), String> {
    let programs = [LegacyComparison::Equal, LegacyComparison::NotEqual].map(|comparison| {
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
        .expect("Comparison proof")
    });
    let mut count = 0;
    for row in include_str!("fixtures/object_comparison_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let columns: Vec<_> = row.split_whitespace().collect();
        assert_eq!((columns.len(), columns[0]), (7, "O"));
        let values: Vec<usize> = columns[1..]
            .iter()
            .map(|column| column.parse())
            .collect::<Result<_, _>>()
            .map_err(|error: std::num::ParseIntError| error.to_string())?;
        assert!(values[0] <= 1 && [0, 6, 9].contains(&values[1]) && values[2] <= 1);
        let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
        let ids = [
            allocate(&mut objects),
            allocate(&mut objects),
            allocate(&mut objects),
        ];
        let left = *ids
            .get(values[3].checked_sub(1).ok_or("Invalid native left ID")?)
            .ok_or("Invalid left ID")?;
        let right = *ids
            .get(values[4].checked_sub(1).ok_or("Invalid native right ID")?)
            .ok_or("Invalid right ID")?;
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
            .with_objects(&mut objects, &[]);
        assert_eq!(
            programs[values[2]]
                .execute_with_heap(
                    &[KernelValue::Object(left), KernelValue::Object(right)],
                    KernelExecutionBudget::default(),
                    &mut context
                )
                .expect("Ordinary pointer identity")
                .value,
            number(i32::try_from(values[5]).map_err(|error| error.to_string())?)
        );
        assert_eq!(context.usage().steps, 5);
        assert_eq!(context.usage().peak_scratch_bytes, 0);
        count += 1;
    }
    assert_eq!(count, 108);
    Ok(())
}
