// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests first independent owner creation at deep frame depth and prepaid failed-return cleanup with one value slot."
)]
use crate::array_read_support::number;
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};

#[test]
fn first_independent_construction_twelve_thousand_frames_deep_moves_one_owner_and_unwinds_a_short_return()
-> Result<(), String> {
    let module = module_compile_support::compile(
        b"descend(n){if(n==0)return [];return descend(n-1);}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Deep independent constructor");
    let mut maximum = None;
    for fail in [false, true] {
        let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
        let mut objects = ObjectHeap::new(
            &strings,
            ObjectHeapBudget {
                max_live_objects: 1,
                max_object_slots: 1,
                max_array_values: 1,
                max_array_value_slots: 1,
                ..ObjectHeapBudget::default()
            },
        )
        .expect("Only one independent data/value");
        let work = HeapWorkBudget::default();
        let mut context = KernelHeapContext::new(
            &mut strings,
            &[],
            HeapWorkBudget {
                max_steps: maximum.unwrap_or(work.max_steps),
                ..work
            },
        )
        .with_objects(&mut objects, &[]);
        let result = module.program().execute_with_heap(
            0,
            &[number(12_000)],
            KernelCallBudget {
                max_setup_work: 100_000_000,
                ..KernelCallBudget::default()
            },
            &mut context,
        );
        if fail {
            let error = result.expect_err("Final deep return is one heap-work step short");
            assert_eq!(error.peak_frames, 12_001);
            assert_eq!(
                context
                    .object_heap_usage()
                    .expect("Objects")
                    .live_array_values,
                0
            );
            drop(context);
        } else {
            let result = result.expect("First owner at the leaf");
            assert_eq!(result.usage.peak_frames, 12_001);
            maximum = Some(context.usage().steps - 1);
            let KernelValue::Array(id) = result.value else {
                return Err("Expected one host owner".into());
            };
            assert_eq!(
                context
                    .object_heap_usage()
                    .expect("Objects")
                    .live_array_values,
                1
            );
            drop(context);
            let data = objects
                .array_data(id, work)
                .expect("Fresh empty data")
                .value;
            assert_eq!(
                objects
                    .array_owner_count(data, work)
                    .expect("Transferred owner only")
                    .value,
                1
            );
            objects
                .release_array_value(id, work)
                .expect("Host releases result");
        }
        assert_eq!(objects.usage().array_value_slots, 1);
        assert_eq!(
            objects
                .reclaim(&[], work)
                .expect("Complete cleanup; no recursive destruction")
                .objects,
            1
        );
    }
    Ok(())
}
