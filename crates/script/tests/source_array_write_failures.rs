// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Source errors preserve COW publication order and repeated context ownership."
)]
use crate::{
    array_read_support::number, source_array_write_support::State, source_scope_support::compile,
};
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelValue};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn source_bad_keys_follow_cow_selection_for_both_normal_assignment_and_literal_clear() {
    for rhs in ["9", "undefined"] {
        for (key, expected) in [
            ("-1", KernelErrorKind::ArrayIndexOutOfRange),
            ("8388608", KernelErrorKind::ArrayIndexOutOfRange),
            ("0.0", KernelErrorKind::ExpectedArrayIndex),
            ("undefined", KernelErrorKind::ExpectedArrayIndex),
            ("&\"Private\"", KernelErrorKind::ExpectedArrayIndex),
        ] {
            let module = compile(format!("main(a){{a[{key}]={rhs};return 0;}}").as_bytes());
            let work = HeapWorkBudget::default();
            let mut state = State::new(ObjectHeapBudget::default());
            let host = state.objects.new_array_value(work).expect("Host").value;
            let data = state
                .objects
                .array_data(host, work)
                .expect("Host data")
                .value;
            for index in 0..3 {
                state
                    .objects
                    .array_put_unique(
                        host,
                        ArrayKey::Index(index),
                        number(7),
                        &state.strings,
                        work,
                    )
                    .expect("Host payload");
            }
            let error = state
                .run(
                    &module,
                    &[KernelValue::Array(host)],
                    KernelCallBudget::default(),
                    work,
                )
                .0
                .expect_err("Invalid key after COW");
            assert_eq!(error.cause.kind, expected);
            assert!(!format!("{error:?} {error}").contains("Private"));
            let usage = state.objects.usage();
            assert_eq!(
                (
                    usage.live_arrays,
                    usage.live_fields,
                    usage.live_array_values
                ),
                (2, 6, 1)
            );
            assert_eq!(
                state
                    .objects
                    .array_owner_count(data, work)
                    .expect("Host only")
                    .value,
                1
            );
            assert_eq!(
                state
                    .objects
                    .array_get(data, ArrayKey::Index(0), &state.strings, work)
                    .expect("Host unchanged")
                    .value,
                Some(number(7))
            );
        }
    }
}

#[test]
fn source_writes_recover_repeatedly_in_one_heap_context_without_retaining_vm_owners() {
    let module = compile(b"main(a,k){a[k]=99;return a.size;}");
    let work = HeapWorkBudget::default();
    let mut state = State::new(ObjectHeapBudget::default());
    let host = state.objects.new_array_value(work).expect("Host").value;
    let data = state
        .objects
        .array_data(host, work)
        .expect("Host data")
        .value;
    for index in 0..3 {
        state
            .objects
            .array_put_unique(
                host,
                ArrayKey::Index(index),
                number(7),
                &state.strings,
                work,
            )
            .expect("Host payload");
    }
    let mut context =
        KernelHeapContext::new(&mut state.strings, &[], work).with_objects(&mut state.objects, &[]);
    for _ in 0..32 {
        let error = module
            .program()
            .execute_with_heap(
                0,
                &[KernelValue::Array(host), number(-1)],
                KernelCallBudget::default(),
                &mut context,
            )
            .expect_err("Bad key after selected copy");
        assert_eq!(error.cause.kind, KernelErrorKind::ArrayIndexOutOfRange);
        assert_eq!(
            module
                .program()
                .execute_with_heap(
                    0,
                    &[KernelValue::Array(host), number(3)],
                    KernelCallBudget::default(),
                    &mut context
                )
                .expect("Same-context recovery")
                .value,
            number(4)
        );
        assert_eq!(
            context
                .object_heap_usage()
                .expect("Objects")
                .live_array_values,
            1
        );
    }
    drop(context);
    assert_eq!(
        state
            .objects
            .array_owner_count(data, work)
            .expect("Only host remains")
            .value,
        1
    );
    assert_eq!(
        state
            .objects
            .reclaim(&[], work)
            .expect("Unowned copies reclaim")
            .objects,
        64
    );
    assert_eq!(state.objects.usage().live_fields, 3);
}
