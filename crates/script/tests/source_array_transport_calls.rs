// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests preserve checked array owners across deep, indirect and rejected source calls."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn twelve_thousand_array_frames_complete_and_unwind_without_recursive_host_destruction()
-> Result<(), String> {
    let module = module_compile_support::compile(
        b"descend(a,n){if(n==0)return a;return descend(a,n-1);}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Recursive transport source");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let host = objects
        .new_array_value(HeapWorkBudget::default())
        .expect("Host")
        .value;
    let data = objects
        .array_data(host, HeapWorkBudget::default())
        .expect("Data")
        .value;
    for frames in [12_000, 12_001] {
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
            .with_objects(&mut objects, &[]);
        let result = module.program().execute_with_heap(
            0,
            &[
                KernelValue::Array(host),
                KernelValue::Number(LegacyNumber::Integer(12_000)),
            ],
            KernelCallBudget {
                max_setup_work: 100_000_000,
                max_frames: frames,
                ..KernelCallBudget::default()
            },
            &mut context,
        );
        if frames == 12_000 {
            let error = result.expect_err("Deep admitted failure");
            assert_eq!(error.peak_frames, 12_000);
            assert_eq!(
                context
                    .object_heap_usage()
                    .expect("Objects")
                    .live_array_values,
                1
            );
        } else {
            let result = result.expect("12,001 iterative frames");
            assert_eq!(result.usage.peak_frames, 12_001);
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
                    .array_data(returned, HeapWorkBudget::default())
                    .expect("Returned data")
                    .value,
                data
            );
            objects
                .release_array_value(returned, HeapWorkBudget::default())
                .expect("Host releases result");
        }
        assert_eq!(
            objects
                .array_owner_count(data, HeapWorkBudget::default())
                .expect("Host survives")
                .value,
            1
        );
    }
    Ok(())
}

#[test]
fn all_predicates_and_evaluated_extras_release_array_operands_before_reusing_four_slots() {
    let module = module_compile_support::compile(
        b"main(a){isdefined(a,a);isint(a);isfloat(a);isvector(a);isstring(a);
          return isdefined(missing(a));} missing(x,y){if(isdefined(y))return undefined;return x;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Builtins, extras and missing parameters");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_array_values: 4,
            max_array_value_slots: 4,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Four managed slots");
    let host = objects
        .new_array_value(HeapWorkBudget::default())
        .expect("Host")
        .value;
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(
            0,
            &[KernelValue::Array(host)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("Prompt operand release");
    assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(1)));
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        1
    );
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .array_value_slots,
        4
    );
}

#[test]
fn rejected_calls_and_array_operations_restore_host_owners_without_enabling_source_arrays() {
    for source in [
        "main(a){return one(a,a);} one(x){return x;}",
        "main(a){return isint(a,a);}",
        "main(a){return [[a]](a);}",
        "main(a){return a.size;}",
        "main(a){return a[0];}",
        "main(a){if(a)return 1;return 0;}",
        "main(a){o=spawnstruct();o.private=a;return 0;}",
        "main(a){return a==a;}",
    ] {
        let module = module_compile_support::compile(
            source.as_bytes(),
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect("Supported expression with rejected runtime type");
        let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
        let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
        let host = objects
            .new_array_value(HeapWorkBudget::default())
            .expect("Host")
            .value;
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
            .with_objects(&mut objects, &[]);
        module
            .program()
            .execute_with_heap(
                0,
                &[KernelValue::Array(host)],
                KernelCallBudget::default(),
                &mut context,
            )
            .expect_err("Unsupported operation");
        assert_eq!(
            context
                .object_heap_usage()
                .expect("Objects")
                .live_array_values,
            1
        );
    }
}
