// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests reject forged domains, dead locations, immutable runtime constants and unsupported owned fields."
)]
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{
    ArrayKey, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind, StringHeap,
    StringHeapBudget,
};

#[test]
fn stale_and_foreign_array_arguments_fail_before_cloning_and_pure_entries_reject_heap_values() {
    let proof = VerifiedKernelProgram::verify_function(
        &[KernelInstruction::LoadLocal(0), KernelInstruction::Return],
        KernelFunctionLayout::new(1, 1).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Transport proof");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let mut foreign = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Other arena");
    let host = objects
        .new_array_value(HeapWorkBudget::default())
        .expect("Host")
        .value;
    let dead = objects
        .copy_array_value(host, HeapWorkBudget::default())
        .expect("Dead location")
        .value;
    objects
        .release_array_value(dead, HeapWorkBudget::default())
        .expect("Invalidate");
    let other = foreign
        .new_array_value(HeapWorkBudget::default())
        .expect("Foreign")
        .value;
    for (value, expected) in [
        (dead, ObjectHeapErrorKind::StaleArrayValue),
        (other, ObjectHeapErrorKind::ForeignArrayValue),
    ] {
        let before = objects.usage();
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
            .with_objects(&mut objects, &[]);
        let error = proof
            .execute_with_heap(
                &[KernelValue::Array(value)],
                KernelExecutionBudget::default(),
                &mut context,
            )
            .expect_err("Checked domain/generation");
        assert_eq!(error.kind, KernelErrorKind::ObjectHeap(expected));
        drop(context);
        assert_eq!(objects.usage(), before);
    }
    assert_eq!(
        proof
            .execute_with_arguments(
                &[KernelValue::Array(host)],
                KernelExecutionBudget::default()
            )
            .expect_err("Pure context")
            .kind,
        KernelErrorKind::ObjectExecutionUnsupported
    );
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default());
    assert_eq!(
        proof
            .execute_with_heap(
                &[KernelValue::Array(host)],
                KernelExecutionBudget::default(),
                &mut context
            )
            .expect_err("String-only context")
            .kind,
        KernelErrorKind::ObjectExecutionUnsupported
    );
    assert_eq!(
        format!("{:?}", KernelValue::Array(host)),
        "KernelValue::Array(..)"
    );
}

#[test]
fn even_unreachable_array_constants_and_owned_array_fields_are_rejected_atomically() {
    let strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let budget = HeapWorkBudget::default();
    let host = objects.new_array_value(budget).expect("Host").value;
    let value = KernelValue::Array(host);
    let code = [
        KernelInstruction::Jump(2),
        KernelInstruction::Constant(value),
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::Return,
    ];
    assert_eq!(
        VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
            .expect_err("Unreachable heap owner constant")
            .kind,
        KernelErrorKind::RuntimeArrayConstant
    );
    let receiver = objects.allocate(budget).expect("Struct").id;
    let before = objects.usage();
    assert_eq!(
        objects
            .put(receiver, b"private", value, &strings, budget)
            .expect_err("Owned struct field")
            .kind,
        ObjectHeapErrorKind::ManagedArrayFieldUnsupported
    );
    assert_eq!(objects.usage(), before);
    assert_eq!(
        objects
            .array_put_unique(host, ArrayKey::Index(0), value, &strings, budget)
            .expect_err("Nested array entry")
            .kind,
        ObjectHeapErrorKind::ManagedArrayFieldUnsupported
    );
    assert_eq!(objects.usage(), before);
    assert_eq!(objects.field_count(receiver), Ok(0));
    let data = objects.array_data(host, budget).expect("Data").value;
    assert_eq!(
        objects
            .array_len(data, budget)
            .expect("Array remains empty")
            .value,
        0
    );
}
