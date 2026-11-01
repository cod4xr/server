// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compare managed host owners with VM copy/move/return and exhausted-budget cleanup."
)]
use crate::kernel_call_support::{self, definition, integer};
use cod4x_script::execution::{
    KernelCallBudget, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};

#[test]
fn returned_array_transfers_one_new_host_owner_after_local_copy_and_overwrite() -> Result<(), String>
{
    let code = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Duplicate,
        KernelInstruction::StoreLocal(1),
        KernelInstruction::Discard,
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Return,
    ];
    let proof = VerifiedKernelProgram::verify_function(
        &code,
        KernelFunctionLayout::new(1, 2).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Transport program");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let original = objects
        .new_array_value(HeapWorkBudget::default())
        .expect("Host root")
        .value;
    let data = objects
        .array_data(original, HeapWorkBudget::default())
        .expect("Data")
        .value;
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let result = proof
        .execute_with_heap(
            &[KernelValue::Array(original)],
            KernelExecutionBudget::default(),
            &mut context,
        )
        .expect("Managed transport");
    let KernelValue::Array(returned) = result.value else {
        return Err("Expected owned array return".into());
    };
    assert_ne!(returned, original);
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        2
    );
    drop(context);
    assert_eq!(
        objects
            .array_owner_count(data, HeapWorkBudget::default())
            .expect("Owners")
            .value,
        2
    );
    objects
        .release_array_value(returned, HeapWorkBudget::default())
        .expect("Release returned host owner");
    assert_eq!(
        objects
            .array_owner_count(data, HeapWorkBudget::default())
            .expect("Owners")
            .value,
        1
    );
    Ok(())
}

#[test]
fn callee_argument_owners_release_before_the_caller_resumes_under_tight_value_slots()
-> Result<(), String> {
    let caller = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Call {
            function: 1,
            arguments: 1,
        },
        KernelInstruction::Discard,
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Return,
    ];
    let worker = [integer(1), KernelInstruction::Return];
    let module =
        kernel_call_support::verify(&[definition(&caller, 1, 1), definition(&worker, 1, 1)]);
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_array_values: 3,
            max_array_value_slots: 3,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Three value slots");
    let original = objects
        .new_array_value(HeapWorkBudget::default())
        .expect("Host root")
        .value;
    let data = objects
        .array_data(original, HeapWorkBudget::default())
        .expect("Data")
        .value;
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let result = module
        .execute_with_heap(
            0,
            &[KernelValue::Array(original)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("Callee's dead aliases cannot occupy a fourth slot");
    let KernelValue::Array(returned) = result.value else {
        return Err("Expected array return".into());
    };
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        2
    );
    drop(context);
    assert_eq!(
        objects
            .array_owner_count(data, HeapWorkBudget::default())
            .expect("Owners")
            .value,
        2
    );
    objects
        .release_array_value(returned, HeapWorkBudget::default())
        .expect("Return release");
    Ok(())
}

#[test]
fn every_insufficient_heap_work_prefix_unwinds_owned_arrays_without_remaining_fuel()
-> Result<(), String> {
    let code = [KernelInstruction::LoadLocal(0), KernelInstruction::Return];
    let proof = VerifiedKernelProgram::verify_function(
        &code,
        KernelFunctionLayout::new(1, 1).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Transport proof");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let original = objects
        .new_array_value(HeapWorkBudget::default())
        .expect("Host root")
        .value;
    let data = objects
        .array_data(original, HeapWorkBudget::default())
        .expect("Data")
        .value;
    let baseline = {
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
            .with_objects(&mut objects, &[]);
        let result = proof
            .execute_with_heap(
                &[KernelValue::Array(original)],
                KernelExecutionBudget::default(),
                &mut context,
            )
            .expect("Baseline");
        (result.value, context.usage())
    };
    let KernelValue::Array(returned) = baseline.0 else {
        return Err("Expected array baseline".into());
    };
    objects
        .release_array_value(returned, HeapWorkBudget::default())
        .expect("Baseline result release");
    for maximum in 0..baseline.1.steps {
        let mut context = KernelHeapContext::new(
            &mut strings,
            &[],
            HeapWorkBudget {
                max_steps: maximum,
                ..HeapWorkBudget::default()
            },
        )
        .with_objects(&mut objects, &[]);
        let error = proof
            .execute_with_heap(
                &[KernelValue::Array(original)],
                KernelExecutionBudget::default(),
                &mut context,
            )
            .expect_err("Short heap-work scope");
        assert!(context.usage().steps <= maximum, "{maximum}: {error:?}");
        assert_eq!(
            context
                .object_heap_usage()
                .expect("Objects")
                .live_array_values,
            1,
            "{maximum}: {error:?}"
        );
        drop(context);
        assert_eq!(
            objects
                .array_owner_count(data, HeapWorkBudget::default())
                .expect("Owners")
                .value,
            1
        );
    }
    Ok(())
}
