// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests checked writable targets, explicit Undefined creation and unreachable operand rejection."
)]
use crate::kernel_call_support::{self, definition, integer};
use crate::object_heap_support::{number, stores};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelInstruction, KernelValue,
};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind};
use cod4x_script::legacy::{LegacyNumber, LegacyVector};

#[test]
fn computed_undefined_autovivifies_and_consumes_a_transient_child_while_literal_clear_rejects() {
    let native = include_str!("fixtures/array_vectors.txt");
    assert!(native.lines().any(|row| row == "B 2 1 0"));
    assert!(native.lines().any(|row| row == "N 1 1 0"));
    let dynamic = [
        KernelInstruction::Constant(KernelValue::Undefined),
        integer(0),
        KernelInstruction::StoreArrayLocal(0),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Return,
    ];
    let literal = [
        integer(0),
        KernelInstruction::ClearArrayLocal(0),
        integer(0),
        KernelInstruction::Return,
    ];
    let module =
        kernel_call_support::verify(&[definition(&dynamic, 1, 1), definition(&literal, 1, 1)]);
    let work = HeapWorkBudget::default();
    let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
    let mut context =
        KernelHeapContext::new(&mut strings, &[], work).with_objects(&mut objects, &[]);
    let error = module
        .execute_with_heap(
            1,
            &[KernelValue::Undefined],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect_err("Native N: Undefined clear does not create an array");
    assert_eq!(error.cause.kind, KernelErrorKind::ExpectedArrayTarget);
    assert_eq!(context.object_heap_usage().expect("Objects").live_arrays, 0);
    let result = module
        .execute_with_heap(
            0,
            &[KernelValue::Undefined],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("Native B: dynamic Undefined creates data and transient child");
    let value = match result.value {
        KernelValue::Array(value) => Some(value),
        _ => None,
    }
    .expect("Transferred independent array");
    let usage = context.object_heap_usage().expect("Objects");
    assert_eq!(
        (
            usage.live_arrays,
            usage.live_fields,
            usage.field_slots,
            usage.live_array_values
        ),
        (1, 0, 1, 1)
    );
    drop(context);
    let data = objects
        .array_data(value, work)
        .expect("Returned empty data")
        .value;
    assert_eq!(
        objects
            .array_len(data, work)
            .expect("Native B final count")
            .value,
        0
    );
    objects
        .release_array_value(value, work)
        .expect("Explicit host release");
}

#[test]
fn readonly_and_wrong_type_targets_keep_private_categories_and_context_recovers_without_arrays() {
    let assign = [
        integer(7),
        integer(0),
        KernelInstruction::StoreArrayLocal(0),
        integer(0),
        KernelInstruction::Return,
    ];
    let clear = [
        integer(0),
        KernelInstruction::ClearArrayLocal(0),
        integer(0),
        KernelInstruction::Return,
    ];
    let module =
        kernel_call_support::verify(&[definition(&assign, 1, 1), definition(&clear, 1, 1)]);
    let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
    let work = HeapWorkBudget::default();
    let string = strings
        .allocate(b"PrivateReadOnly", work)
        .expect("String")
        .id;
    let object = objects.allocate(work).expect("Ordinary object").id;
    let function = module
        .function_reference(0)
        .expect("Opaque function target");
    let mut context =
        KernelHeapContext::new(&mut strings, &[], work).with_objects(&mut objects, &[]);
    for (target, expected) in [
        (
            KernelValue::String(string),
            KernelErrorKind::ReadOnlyStringIndex,
        ),
        (
            KernelValue::Vector(LegacyVector::from_numbers([LegacyNumber::Integer(1); 3])),
            KernelErrorKind::ReadOnlyVectorIndex,
        ),
        (
            KernelValue::LocalizedString(string),
            KernelErrorKind::ExpectedArrayTarget,
        ),
        (
            KernelValue::Object(object),
            KernelErrorKind::ExpectedArrayTarget,
        ),
        (
            KernelValue::Function(function),
            KernelErrorKind::ExpectedArrayTarget,
        ),
        (number(1), KernelErrorKind::ExpectedArrayTarget),
    ] {
        for entry in 0..2 {
            let error = module
                .execute_with_heap(entry, &[target], KernelCallBudget::default(), &mut context)
                .expect_err("Native local array reference category");
            assert_eq!(error.cause.kind, expected);
            assert!(!format!("{error:?} {error}").contains("PrivateReadOnly"));
            let usage = context.object_heap_usage().expect("Objects");
            assert_eq!(
                (
                    usage.live_arrays,
                    usage.live_fields,
                    usage.live_array_values
                ),
                (0, 0, 0)
            );
        }
    }
}

#[test]
fn nested_rhs_rejection_precedes_independent_target_creation_and_releases_operand_aliases() {
    let code = [
        KernelInstruction::LoadLocal(1),
        integer(0),
        KernelInstruction::StoreArrayLocal(0),
        integer(0),
        KernelInstruction::Return,
    ];
    let module = kernel_call_support::verify(&[definition(&code, 2, 2)]);
    let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
    let work = HeapWorkBudget::default();
    let host = objects.new_array_value(work).expect("Host RHS").value;
    let data = objects.array_data(host, work).expect("Data").value;
    let mut context =
        KernelHeapContext::new(&mut strings, &[], work).with_objects(&mut objects, &[]);
    let error = module
        .execute_with_heap(
            0,
            &[KernelValue::Undefined, KernelValue::Array(host)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect_err("No stored Array edges");
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::ObjectHeap(ObjectHeapErrorKind::ManagedArrayFieldUnsupported)
    );
    let usage = context.object_heap_usage().expect("Objects");
    assert_eq!(
        (
            usage.live_arrays,
            usage.live_fields,
            usage.live_array_values
        ),
        (1, 0, 1)
    );
    drop(context);
    assert_eq!(
        objects
            .array_owner_count(data, work)
            .expect("Only host owner")
            .value,
        1
    );
}
