// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests COW publication before bad-key errors and distinct literal/computed transient quota semantics."
)]
use crate::kernel_call_support::{self, definition, integer};
use crate::object_heap_support::{number, stores};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelInstruction, KernelValue,
    VerifiedKernelModule,
};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget};
use cod4x_script::legacy::{LegacyNumber, LegacyVector};

fn failure_module() -> VerifiedKernelModule {
    let assign = [
        integer(99),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::StoreArrayLocal(0),
        integer(0),
        KernelInstruction::Return,
    ];
    let clear = [
        KernelInstruction::LoadLocal(1),
        KernelInstruction::ClearArrayLocal(0),
        integer(0),
        KernelInstruction::Return,
    ];
    let good = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Size,
        KernelInstruction::Return,
    ];
    kernel_call_support::verify(&[
        definition(&assign, 2, 2),
        definition(&clear, 2, 2),
        definition(&good, 2, 2),
    ])
}

#[test]
fn invalid_keys_follow_real_cow_publication_and_cleanup_all_vm_owners_before_context_reuse() {
    let module = failure_module();
    let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
    let work = HeapWorkBudget::default();
    let localized = strings
        .allocate(b"PrivateIndex", work)
        .expect("Localized key")
        .id;
    let ordinary = objects
        .allocate(work)
        .expect("Ordinary pointer-key identity")
        .id;
    let host = objects.new_array_value(work).expect("Host").value;
    let data = objects.array_data(host, work).expect("Original data").value;
    for key in 0..3 {
        objects
            .array_put_unique(host, ArrayKey::Index(key), number(7), &strings, work)
            .expect("Host entry");
    }
    let vector = LegacyVector::from_numbers([LegacyNumber::Integer(0); 3]);
    for (key, expected) in [
        (number(-1), KernelErrorKind::ArrayIndexOutOfRange),
        (number(0x80_0000), KernelErrorKind::ArrayIndexOutOfRange),
        (KernelValue::Undefined, KernelErrorKind::ExpectedArrayIndex),
        (
            KernelValue::Object(ordinary),
            KernelErrorKind::ExpectedArrayIndex,
        ),
        (
            KernelValue::LocalizedString(localized),
            KernelErrorKind::ExpectedArrayIndex,
        ),
        (
            KernelValue::Vector(vector),
            KernelErrorKind::ExpectedArrayIndex,
        ),
        (
            KernelValue::Number(LegacyNumber::Float(0.0)),
            KernelErrorKind::ExpectedArrayIndex,
        ),
        (
            KernelValue::Function(module.function_reference(0).expect("Checked function")),
            KernelErrorKind::ExpectedArrayIndex,
        ),
        (
            KernelValue::Array(host),
            KernelErrorKind::ExpectedArrayIndex,
        ),
    ] {
        for entry in 0..2 {
            let roots = [ordinary];
            let mut context =
                KernelHeapContext::new(&mut strings, &[], work).with_objects(&mut objects, &roots);
            let arguments = [KernelValue::Array(host), key];
            let error = module
                .execute_with_heap(entry, &arguments, KernelCallBudget::default(), &mut context)
                .expect_err("Bad key after target selection");
            assert_eq!(error.cause.kind, expected);
            assert!(!format!("{error:?} {error}").contains("PrivateIndex"));
            let usage = context.object_heap_usage().expect("Objects");
            assert_eq!(
                (
                    usage.live_arrays,
                    usage.live_fields,
                    usage.live_array_values
                ),
                (2, 6, 1),
                "The selected local really detached before key validation"
            );
            assert_eq!(
                module
                    .execute_with_heap(2, &arguments, KernelCallBudget::default(), &mut context)
                    .expect("Same-context recovery")
                    .value,
                number(3)
            );
            drop(context);
            assert_eq!(
                objects
                    .array_owner_count(data, work)
                    .expect("Host only")
                    .value,
                1
            );
            let keys = objects.array_keys(data, work).expect("Original key order");
            assert!(matches!(
                keys.value.as_slice(),
                [ArrayKey::Index(2), ArrayKey::Index(1), ArrayKey::Index(0)]
            ));
            assert_eq!(
                objects
                    .reclaim(&roots, work)
                    .expect("Only the unowned copy is dead")
                    .objects,
                1
            );
        }
    }
}

#[test]
fn missing_computed_undefined_cannot_bypass_transient_field_or_key_capacity() {
    for dynamic in [false, true] {
        let mut code = vec![
            KernelInstruction::EmptyArray,
            KernelInstruction::StoreLocal(0),
        ];
        if dynamic {
            code.push(KernelInstruction::Constant(KernelValue::Undefined));
        }
        code.push(integer(0));
        code.push(if dynamic {
            KernelInstruction::StoreArrayLocal(0)
        } else {
            KernelInstruction::ClearArrayLocal(0)
        });
        code.extend([integer(0), KernelInstruction::Return]);
        let module = kernel_call_support::verify(&[definition(&code, 0, 1)]);
        for (max_live_fields, max_key_bytes) in [(0, 9), (1, 0)] {
            let (mut strings, mut objects) = stores(ObjectHeapBudget {
                max_live_fields,
                max_key_bytes,
                ..ObjectHeapBudget::default()
            });
            let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
                .with_objects(&mut objects, &[]);
            let result =
                module.execute_with_heap(0, &[], KernelCallBudget::default(), &mut context);
            if dynamic {
                result.expect_err("Computed lvalue must admit a transient child");
            } else {
                assert_eq!(
                    result
                        .expect("Literal missing clear creates no child")
                        .value,
                    number(0)
                );
            }
            let usage = context.object_heap_usage().expect("Objects");
            assert_eq!(
                (
                    usage.live_array_values,
                    usage.live_fields,
                    usage.field_slots
                ),
                (0, 0, 0)
            );
        }
    }
}
