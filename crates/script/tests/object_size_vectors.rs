// SPDX-License-Identifier: AGPL-3.0-or-later
//! Actual C pointer-size helper with modeled parent/ref services, not native cleanup/VM parity.
#![allow(
    clippy::expect_used,
    reason = "Tests replay only the checked ordinary-struct subtype."
)]
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelResource, KernelValue, KernelVerificationBudget,
    VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};

fn program() -> VerifiedKernelProgram {
    VerifiedKernelProgram::verify_function(
        &[
            KernelInstruction::LoadLocal(0),
            KernelInstruction::Size,
            KernelInstruction::Return,
        ],
        KernelFunctionLayout::new(1, 1).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Size proof")
}

#[test]
fn native_ordinary_struct_size_outcomes_replay_while_other_subtypes_remain_helper_only()
-> Result<(), String> {
    let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
    let object = allocate(&mut objects);
    let program = program();
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let (mut replayed, mut helper_only) = (0, 0);
    for row in include_str!("fixtures/object_size_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let fields: Vec<_> = row.split_whitespace().collect();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "O");
        let parent = u32::from_str_radix(fields[1], 16).map_err(|error| error.to_string())?;
        let native_word = fields[2]
            .parse::<u16>()
            .map_err(|error| error.to_string())?;
        let expected = fields[3]
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        assert_eq!(
            expected,
            if parent & 31 == 21 {
                i32::from(native_word)
            } else {
                1
            }
        );
        if parent & 31 != 18 {
            helper_only += 1;
            continue;
        }
        let result = program
            .execute_with_heap(
                &[KernelValue::Object(object)],
                KernelExecutionBudget::default(),
                &mut context,
            )
            .expect("Ordinary shared struct");
        assert_eq!(result.value, number(expected));
        assert_eq!(result.steps, 3);
        replayed += 1;
    }
    assert_eq!((replayed, helper_only), (15, 165));
    Ok(())
}

#[test]
fn struct_size_is_one_at_70000_fields_with_constant_work_and_one_less_global_budget_rejects() {
    let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
    let object = allocate(&mut objects);
    for key in 0_i32..70_000 {
        objects
            .put(
                object,
                &key.to_le_bytes(),
                number(key),
                &strings,
                HeapWorkBudget::default(),
            )
            .expect("Expanded field count");
    }
    assert_eq!(objects.field_count(object), Ok(70_000));
    let before = objects.usage();
    let program = program();
    let mut context = KernelHeapContext::new(
        &mut strings,
        &[],
        HeapWorkBudget {
            max_steps: 3,
            ..HeapWorkBudget::default()
        },
    )
    .with_objects(&mut objects, &[]);
    assert_eq!(
        program
            .execute_with_heap(
                &[KernelValue::Object(object)],
                KernelExecutionBudget::default(),
                &mut context
            )
            .expect("Exact constant admission")
            .value,
        number(1)
    );
    assert_eq!(context.usage().steps, 3);
    assert_eq!(context.object_heap_usage(), Some(before));
    let mut context = KernelHeapContext::new(
        &mut strings,
        &[],
        HeapWorkBudget {
            max_steps: 2,
            ..HeapWorkBudget::default()
        },
    )
    .with_objects(&mut objects, &[]);
    let error = program
        .execute_with_heap(
            &[KernelValue::Object(object)],
            KernelExecutionBudget::default(),
            &mut context,
        )
        .expect_err("One less");
    assert_eq!(
        error.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::HeapWorkSteps,
            requested: 3,
            maximum: 2
        }
    );
    assert_eq!(error.instruction, Some(1));
    assert_eq!(context.object_heap_usage(), Some(before));
}
