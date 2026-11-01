// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Private tests force generation/counter boundaries and wrong-kind handles."
)]
use crate::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use crate::heap::objects::arrays::{ArrayId, ArrayValueState};
use crate::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind, ObjectResource, StringHeap,
    StringHeapBudget,
};

#[test]
fn shared_struct_apis_and_vm_arguments_reject_array_data_even_when_given_a_private_wrong_tag() {
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut heap = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Heap");
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget).expect("Owned array").value;
    let data = heap.array_data(value, budget).expect("Data").value;
    let ordinary = heap.allocate(budget).expect("Struct").id;
    let before = heap.usage();
    assert_eq!(
        heap.field_count(data.object),
        Err(ObjectHeapErrorKind::ExpectedStruct)
    );
    assert_eq!(
        heap.get(data.object, b"key", &strings, budget)
            .expect_err("No raw array field access")
            .kind,
        ObjectHeapErrorKind::ExpectedStruct
    );
    assert_eq!(
        heap.put(
            data.object,
            b"key",
            KernelValue::Undefined,
            &strings,
            budget
        )
        .expect_err("No COW bypass")
        .kind,
        ObjectHeapErrorKind::ExpectedStruct
    );
    assert_eq!(
        heap.remove(data.object, b"key", &strings, budget)
            .expect_err("No raw deletion")
            .kind,
        ObjectHeapErrorKind::ExpectedStruct
    );
    assert_eq!(
        heap.put(
            ordinary,
            b"array",
            KernelValue::Object(data.object),
            &strings,
            budget
        )
        .expect_err("No wrong-tag value injection")
        .kind,
        ObjectHeapErrorKind::ExpectedStruct
    );
    assert_eq!(
        heap.array_len(ArrayId { object: ordinary }, budget)
            .expect_err("No wrong-kind data")
            .kind,
        ObjectHeapErrorKind::ExpectedArray
    );
    let program = VerifiedKernelProgram::verify_function(
        &[KernelInstruction::LoadLocal(0), KernelInstruction::Return],
        KernelFunctionLayout::new(1, 1).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Identity program");
    let mut context =
        KernelHeapContext::new(&mut strings, &[], budget).with_objects(&mut heap, &[]);
    assert_eq!(
        program
            .execute_with_heap(
                &[KernelValue::Object(data.object)],
                KernelExecutionBudget::default(),
                &mut context
            )
            .expect_err("VM needs a real struct tag")
            .kind,
        KernelErrorKind::ObjectHeap(ObjectHeapErrorKind::ExpectedStruct)
    );
    assert_eq!(heap.usage(), before);
}

#[test]
fn exhausted_array_value_generation_retires_the_slot_instead_of_reusing_a_stale_identity() {
    let strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut heap = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_array_value_slots: 1,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("One value slot");
    let budget = HeapWorkBudget::default();
    let mut value = heap.new_array_value(budget).expect("Value").value;
    let data = heap.array_data(value, budget).expect("Snapshot").value;
    heap.arrays.values[value.slot].generation = u64::MAX;
    value.generation = u64::MAX;
    heap.release_array_value(value, budget)
        .expect("Retire full-domain generation");
    assert!(matches!(
        heap.arrays.values[value.slot].state,
        ArrayValueState::Retired
    ));
    let before = heap.usage();
    assert!(matches!(
        heap.retain_array(data, budget)
            .expect_err("Retired slots remain admitted")
            .kind,
        ObjectHeapErrorKind::BudgetExceeded {
            resource: ObjectResource::ArrayValueSlots,
            requested: 2,
            maximum: 1
        }
    ));
    assert_eq!(heap.usage(), before);
    assert_eq!(
        heap.array_data(value, budget).expect_err("No ABA").kind,
        ObjectHeapErrorKind::StaleArrayValue
    );
}

#[test]
fn managed_owner_counter_exhaustion_returns_a_typed_error_before_alias_publication() {
    let strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut heap = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Heap");
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget).expect("Value").value;
    let data = heap.array_data(value, budget).expect("Data").value;
    heap.arrays.metadata[data.object.slot].owners = u64::MAX;
    let before = heap.usage();
    assert_eq!(
        heap.copy_array_value(value, budget)
            .expect_err("No wrapping ownership count")
            .kind,
        ObjectHeapErrorKind::SizeOverflow
    );
    assert_eq!(heap.usage(), before);
    assert_eq!(heap.arrays.metadata[data.object.slot].owners, u64::MAX);
}
