// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Wrong key domains and stale/foreign references must unwind owners privately."
)]
use crate::{
    kernel_call_support::{self, integer},
    kernel_switch_support,
};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelInstruction,
    KernelSwitchComparison, KernelSwitchKey, KernelValue,
};
use cod4x_script::heap::{
    ArrayKey, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::{LegacyNumber, LegacyVector};

fn number(value: i32) -> KernelValue {
    KernelValue::Number(LegacyNumber::Integer(value))
}

#[test]
fn wrong_key_categories_fail_and_one_context_recovers_without_retaining_array_owners() {
    let module = kernel_switch_support::module(
        KernelSwitchKey::Integer(0),
        KernelSwitchComparison::Equal,
        &[],
    );
    let work = HeapWorkBudget::default();
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let localized = strings.allocate(b"PrivateKey", work).expect("Localized").id;
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let ordinary = objects.allocate(work).expect("Object").id;
    let host = objects.new_array_value(work).expect("Host").value;
    let data = objects.array_data(host, work).expect("Host data").value;
    objects
        .array_put_unique(host, ArrayKey::Index(0), number(7), &strings, work)
        .expect("Host payload");
    let function = module.function_reference(0).expect("Function");
    let keys = [
        KernelValue::Undefined,
        KernelValue::Number(LegacyNumber::Float(0.0)),
        KernelValue::Vector(LegacyVector::from_numbers([LegacyNumber::Integer(0); 3])),
        KernelValue::LocalizedString(localized),
        KernelValue::Object(ordinary),
        KernelValue::Function(function),
        KernelValue::Array(host),
    ];
    let mut context =
        KernelHeapContext::new(&mut strings, &[], work).with_objects(&mut objects, &[]);
    for key in keys {
        let error = module
            .execute_with_heap(0, &[key], KernelCallBudget::default(), &mut context)
            .expect_err("Strict switch domain");
        assert_eq!(error.cause.kind, KernelErrorKind::ExpectedSwitchKey);
        assert!(!format!("{error:?} {error}").contains("PrivateKey"));
        assert_eq!(
            context
                .object_heap_usage()
                .expect("Objects")
                .live_array_values,
            1
        );
        assert_eq!(
            module
                .execute_with_heap(0, &[number(0)], KernelCallBudget::default(), &mut context)
                .expect("Same context recovery")
                .value,
            number(1)
        );
    }
    drop(context);
    assert_eq!(
        objects
            .array_owner_count(data, work)
            .expect("Host owner")
            .value,
        1
    );
    assert_eq!(
        objects
            .array_get(data, ArrayKey::Index(0), &strings, work)
            .expect("Host unchanged")
            .value,
        Some(number(7))
    );
}

#[test]
fn foreign_stale_and_contextless_strings_fail_before_becoming_switch_authority() {
    let module = kernel_switch_support::module(
        KernelSwitchKey::StringLabel(0),
        KernelSwitchComparison::Equal,
        &[b"Private".to_vec()],
    );
    let work = HeapWorkBudget::default();
    let mut owner = StringHeap::new(StringHeapBudget::default()).expect("Owner");
    let id = owner.allocate(b"Private", work).expect("String").id;
    assert_eq!(
        module
            .execute(0, &[KernelValue::String(id)], KernelCallBudget::default())
            .expect_err("No fabricated heap")
            .cause
            .kind,
        KernelErrorKind::HeapContextRequired
    );
    let mut other = StringHeap::new(StringHeapBudget::default()).expect("Other heap");
    let mut context = KernelHeapContext::new(&mut other, &[], work);
    assert_eq!(
        module
            .execute_with_heap(
                0,
                &[KernelValue::String(id)],
                KernelCallBudget::default(),
                &mut context
            )
            .expect_err("Foreign owner")
            .cause
            .kind,
        KernelErrorKind::ForeignStringReference
    );
    owner.reclaim(&[], work).expect("Stale input");
    let mut context = KernelHeapContext::new(&mut owner, &[], work);
    assert_eq!(
        module
            .execute_with_heap(
                0,
                &[KernelValue::String(id)],
                KernelCallBudget::default(),
                &mut context
            )
            .expect_err("Stale generation")
            .cause
            .kind,
        KernelErrorKind::StaleStringReference
    );
}

#[test]
fn standalone_comparisons_validate_types_without_requiring_a_separate_guard_instruction() {
    let code = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::SwitchCompare {
            key: KernelSwitchKey::Integer(0),
            comparison: KernelSwitchComparison::Equal,
        },
        KernelInstruction::Return,
    ];
    let module = kernel_call_support::verify(&[kernel_call_support::definition(&code, 1, 1)]);
    assert_eq!(
        module
            .execute(0, &[KernelValue::Undefined], KernelCallBudget::default())
            .expect_err("Comparator checks its own domain")
            .cause
            .kind,
        KernelErrorKind::ExpectedSwitchKey
    );
    let code = [
        integer(0),
        KernelInstruction::RequireSwitchKey,
        KernelInstruction::Return,
    ];
    let module = kernel_call_support::verify(&[kernel_call_support::definition(&code, 0, 0)]);
    assert_eq!(
        module
            .execute(0, &[], KernelCallBudget::default())
            .expect("Pure key guard")
            .value,
        number(0)
    );
}
