// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests verify standalone constructor stack shape, transfer and prepaid cleanup at every work prefix."
)]
use crate::object_heap_support::stores;
use cod4x_script::execution::{
    KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext, KernelInstruction, KernelValue,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};

fn proof(code: &[KernelInstruction]) -> VerifiedKernelProgram {
    VerifiedKernelProgram::verify_function(
        code,
        KernelFunctionLayout::new(0, 1).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Checked constructor stack effects")
}

#[test]
fn standalone_proof_mints_one_transferred_owner_in_each_distinct_store() -> Result<(), String> {
    let program = proof(&[KernelInstruction::EmptyArray, KernelInstruction::Return]);
    let work = HeapWorkBudget::default();
    for _ in 0..2 {
        let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
        let mut context =
            KernelHeapContext::new(&mut strings, &[], work).with_objects(&mut objects, &[]);
        let result = program
            .execute_with_heap(&[], KernelExecutionBudget::default(), &mut context)
            .expect("No embedded heap IDs");
        assert_eq!(result.steps, 2);
        let KernelValue::Array(id) = result.value else {
            return Err("Expected host owner".into());
        };
        assert_eq!(
            context
                .object_heap_usage()
                .expect("Objects")
                .live_array_values,
            1
        );
        drop(context);
        let data = objects.array_data(id, work).expect("Result data").value;
        assert_eq!(objects.array_len(data, work).expect("Empty count").value, 0);
        assert_eq!(
            objects
                .array_owner_count(data, work)
                .expect("One transferred owner")
                .value,
            1
        );
        objects
            .release_array_value(id, work)
            .expect("Explicit release");
        assert_eq!(
            objects
                .reclaim(&[], work)
                .expect("No retained VM lease")
                .objects,
            1
        );
    }
    Ok(())
}

#[test]
fn standalone_constructor_copy_size_and_return_cleanup_survive_every_shorter_heap_and_instruction_scope()
 {
    let program = proof(&[
        KernelInstruction::EmptyArray,
        KernelInstruction::Duplicate,
        KernelInstruction::StoreLocal(0),
        KernelInstruction::Discard,
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Size,
        KernelInstruction::Return,
    ]);
    let run = |heap: HeapWorkBudget, maximum| {
        let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
        let mut context =
            KernelHeapContext::new(&mut strings, &[], heap).with_objects(&mut objects, &[]);
        let result = program.execute_with_heap(
            &[],
            KernelExecutionBudget {
                max_steps: maximum,
                ..KernelExecutionBudget::default()
            },
            &mut context,
        );
        let usage = context.usage();
        assert!(usage.steps <= heap.max_steps);
        assert!(usage.peak_scratch_bytes <= heap.max_scratch_bytes);
        assert_eq!(
            context
                .object_heap_usage()
                .expect("Objects")
                .live_array_values,
            0
        );
        drop(context);
        let dead_data = objects.usage().live_arrays;
        assert_eq!(
            objects
                .reclaim(&[], HeapWorkBudget::default())
                .expect("All temporary data reclaimable")
                .objects,
            dead_data
        );
        (result, usage)
    };
    let (result, usage) = run(HeapWorkBudget::default(), 7);
    result.expect("Baseline");
    let exact = HeapWorkBudget {
        max_steps: usage.steps,
        max_scratch_bytes: usage.peak_scratch_bytes,
    };
    run(exact, 7).0.expect("Exact work and scratch");
    for maximum in 0..usage.steps {
        run(
            HeapWorkBudget {
                max_steps: maximum,
                ..exact
            },
            7,
        )
        .0
        .expect_err("Shorter heap work");
    }
    for maximum in 0..7 {
        run(exact, maximum).0.expect_err("Shorter instruction work");
    }
    run(
        HeapWorkBudget {
            max_scratch_bytes: exact.max_scratch_bytes - 1,
            ..exact
        },
        7,
    )
    .0
    .expect_err("One less shared scratch");
}
