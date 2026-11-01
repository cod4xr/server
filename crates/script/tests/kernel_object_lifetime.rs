// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests distinguish returned snapshots from explicitly rooted host ownership."
)]
use crate::kernel_heap_support;
use crate::object_heap_support::allocate;
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{
    HeapErrorKind, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind,
};
use cod4x_script::legacy::LegacyArithmetic;

fn literal_concat() -> VerifiedKernelProgram {
    VerifiedKernelProgram::verify_with_literals(
        &[
            KernelInstruction::StringLiteral(0),
            KernelInstruction::StringLiteral(0),
            KernelInstruction::Arithmetic(LegacyArithmetic::Add),
            KernelInstruction::Return,
        ],
        KernelFunctionLayout::default(),
        &[b"x".to_vec()],
        KernelVerificationBudget::default(),
    )
    .expect("Literal concatenation")
}

#[test]
fn returned_object_snapshot_survives_explicit_external_root_then_reclaims_when_root_is_omitted() {
    let mut strings = kernel_heap_support::heap(3);
    let held = kernel_heap_support::allocate(&mut strings, b"held");
    kernel_heap_support::allocate(&mut strings, b"old");
    kernel_heap_support::allocate(&mut strings, b"other");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let root = allocate(&mut objects);
    for (key, value) in [
        (b"self".as_slice(), KernelValue::Object(root)),
        (b"value", KernelValue::String(held)),
    ] {
        objects
            .put(root, key, value, &strings, HeapWorkBudget::default())
            .expect("Reachable cycle");
    }
    let identity = VerifiedKernelProgram::verify_function(
        &[KernelInstruction::LoadLocal(0), KernelInstruction::Return],
        KernelFunctionLayout::new(1, 1).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Identity");
    let returned = {
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
            .with_objects(&mut objects, &[]);
        identity
            .execute_with_heap(
                &[KernelValue::Object(root)],
                KernelExecutionBudget::default(),
                &mut context,
            )
            .expect("Return snapshot")
            .value
    };
    assert_eq!(returned, KernelValue::Object(root));
    let program = literal_concat();
    {
        let roots = [root];
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
            .with_objects(&mut objects, &roots);
        program
            .execute_with_heap(&[], KernelExecutionBudget::default(), &mut context)
            .expect("Explicit returned root");
        assert_eq!(
            context
                .object_heap_usage()
                .expect("Rooted store")
                .live_objects,
            1
        );
        assert_eq!(context.get(held).expect("Nested root survives"), b"held");
    }
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    program
        .execute_with_heap(&[], KernelExecutionBudget::default(), &mut context)
        .expect("Unrooted snapshot is not a lease");
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Collected graph")
            .live_objects,
        0
    );
    assert_eq!(context.get(held), Err(HeapErrorKind::StaleReference));
    let before = context.usage();
    let error = identity
        .execute_with_heap(&[returned], KernelExecutionBudget::default(), &mut context)
        .expect_err("Stale returned copy");
    assert_eq!(
        error.kind,
        KernelErrorKind::ObjectHeap(ObjectHeapErrorKind::StaleReference)
    );
    assert!(context.usage().steps > before.steps);
    assert_eq!(
        context.usage().object_collections,
        before.object_collections
    );
}

#[test]
fn object_collection_failure_on_stale_stored_string_keeps_reachable_graph_and_remaining_string_payloads()
 {
    let mut strings = kernel_heap_support::heap(2);
    let stale = kernel_heap_support::allocate(&mut strings, b"stale");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let root = allocate(&mut objects);
    objects
        .put(
            root,
            b"stale",
            KernelValue::LocalizedString(stale),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Initially valid field");
    strings
        .reclaim(&[], HeapWorkBudget::default())
        .expect("Host deliberately omits object roots");
    let left = kernel_heap_support::allocate(&mut strings, b"a");
    let right = kernel_heap_support::allocate(&mut strings, b"b");
    let program = kernel_heap_support::binary(KernelInstruction::Arithmetic(LegacyArithmetic::Add));
    let roots = [root];
    let before = strings.usage();
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &roots);
    let error = program
        .execute_with_heap(
            &[KernelValue::String(left), KernelValue::String(right)],
            KernelExecutionBudget::default(),
            &mut context,
        )
        .expect_err("Stored stale string cannot be ignored");
    assert_eq!(error.kind, KernelErrorKind::StaleStringReference);
    assert_eq!(
        (
            context.usage().object_collections,
            context.usage().collections
        ),
        (1, 0)
    );
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Reachable object preserved")
            .live_objects,
        1
    );
    assert_eq!(context.heap_usage(), before);
    assert_eq!(context.get(left).expect("No partial string sweep"), b"a");
    assert_eq!(context.get(right).expect("No partial string sweep"), b"b");
}
