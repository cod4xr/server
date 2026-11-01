// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exhaust every combined invocation work prefix and scratch boundary."
)]
use crate::kernel_heap_support;
use crate::object_heap_support::allocate;
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelResource, KernelValue, KernelVerificationBudget,
    VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectId, StringHeap};
use cod4x_script::legacy::LegacyArithmetic;

fn stores() -> (StringHeap, ObjectHeap, ObjectId, [KernelValue; 3]) {
    let mut strings = kernel_heap_support::heap(4);
    let held = kernel_heap_support::allocate(&mut strings, b"held");
    let left = kernel_heap_support::allocate(&mut strings, b"a");
    let right = kernel_heap_support::allocate(&mut strings, b"b");
    let garbage = kernel_heap_support::allocate(&mut strings, b"dead");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let root = allocate(&mut objects);
    let dead = allocate(&mut objects);
    for (id, key, value) in [
        (root, b"held".as_slice(), KernelValue::String(held)),
        (dead, b"self", KernelValue::Object(dead)),
        (dead, b"value", KernelValue::String(garbage)),
    ] {
        objects
            .put(id, key, value, &strings, HeapWorkBudget::default())
            .expect("Object fields");
    }
    (
        strings,
        objects,
        root,
        [
            KernelValue::Object(root),
            KernelValue::String(left),
            KernelValue::String(right),
        ],
    )
}

fn program() -> VerifiedKernelProgram {
    VerifiedKernelProgram::verify_function(
        &[
            KernelInstruction::LoadLocal(1),
            KernelInstruction::LoadLocal(2),
            KernelInstruction::Arithmetic(LegacyArithmetic::Add),
            KernelInstruction::Discard,
            KernelInstruction::LoadLocal(0),
            KernelInstruction::Return,
        ],
        KernelFunctionLayout::new(3, 3).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Proof")
}

#[test]
fn object_and_string_pressure_share_exact_global_work_scratch_and_record_partial_completed_collection()
 {
    let program = program();
    let (mut strings, mut objects, root, arguments) = stores();
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    assert_eq!(
        program
            .execute_with_heap(&arguments, KernelExecutionBudget::default(), &mut context)
            .expect("Measure complete operation")
            .value,
        KernelValue::Object(root)
    );
    let usage = context.usage();
    assert_eq!((usage.collections, usage.object_collections), (1, 1));
    let budgets = (0..usage.steps)
        .map(|max_steps| HeapWorkBudget {
            max_steps,
            ..HeapWorkBudget::default()
        })
        .chain([HeapWorkBudget {
            max_scratch_bytes: usage.peak_scratch_bytes - 1,
            ..HeapWorkBudget::default()
        }]);
    let mut saw_completed_object_collection_before_failure = false;
    for budget in budgets {
        let (mut strings, mut objects, root, arguments) = stores();
        let before = objects.usage();
        let mut context =
            KernelHeapContext::new(&mut strings, &[], budget).with_objects(&mut objects, &[]);
        let error = program
            .execute_with_heap(&arguments, KernelExecutionBudget::default(), &mut context)
            .expect_err("One incomplete budget");
        assert!(
            matches!(error.kind, KernelErrorKind::BudgetExceeded { resource: KernelResource::HeapWorkSteps | KernelResource::HeapScratchBytes, maximum, requested } if requested > maximum)
        );
        assert!(context.usage().steps <= budget.max_steps);
        assert!(context.usage().peak_scratch_bytes <= budget.max_scratch_bytes);
        if context.usage().object_collections == 0 {
            assert_eq!(context.object_heap_usage(), Some(before));
        } else {
            saw_completed_object_collection_before_failure |= context.usage().collections == 0;
            assert_eq!(
                context.object_heap_usage().expect("Objects").live_objects,
                1
            );
        }
        assert!(context.usage().collections <= 1);
        // Context borrowing ends here; every live graph field must still resolve.
        let object_usage = context.object_heap_usage();
        assert!(object_usage.is_some());
        assert_eq!(objects.field_count(root), Ok(1));
        let stored = objects
            .get(root, b"held", &strings, HeapWorkBudget::default())
            .expect("No stale reachable field")
            .value;
        assert!(matches!(stored, Some(KernelValue::String(_))));
    }
    assert!(saw_completed_object_collection_before_failure);
    let (mut strings, mut objects, root, arguments) = stores();
    let mut context = KernelHeapContext::new(
        &mut strings,
        &[],
        HeapWorkBudget {
            max_steps: usage.steps,
            max_scratch_bytes: usage.peak_scratch_bytes,
        },
    )
    .with_objects(&mut objects, &[]);
    assert_eq!(
        program
            .execute_with_heap(&arguments, KernelExecutionBudget::default(), &mut context)
            .expect("Exact shared budget")
            .value,
        KernelValue::Object(root)
    );
    assert_eq!(context.usage(), usage);
}
