// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "First populated independent arrays at deep call depth must transfer and unwind iteratively."
)]
use crate::{
    array_read_support::number, source_array_write_support::State, source_scope_support::compile,
};
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn first_populated_array_twelve_thousand_frames_deep_returns_and_exhausted_unwind_releases_every_owner()
 {
    let module = compile(
        b"descend(n){if(n==0){a=[];a[0]=\"left\"+\"right\";return a;}return descend(n-1);}",
    );
    let mut maximum = HeapWorkBudget::default().max_steps;
    let work = HeapWorkBudget::default();
    for fail in [false, true] {
        let mut state = State::new(ObjectHeapBudget {
            max_live_objects: 1,
            max_object_slots: 1,
            max_live_fields: 1,
            max_field_slots: 1,
            max_array_values: 2,
            max_array_value_slots: 2,
            ..ObjectHeapBudget::default()
        });
        let (result, usage) = state.run(
            &module,
            &[number(12_000)],
            KernelCallBudget {
                max_setup_work: 100_000_000,
                ..KernelCallBudget::default()
            },
            HeapWorkBudget {
                max_steps: maximum,
                ..work
            },
        );
        if fail {
            assert_eq!(
                result
                    .expect_err("One-less final owner transfer")
                    .peak_frames,
                12_001
            );
            assert_eq!(state.objects.usage().live_array_values, 0);
        } else {
            let result = result.expect("Populated leaf result");
            assert_eq!(result.usage.peak_frames, 12_001);
            maximum = usage.steps - 1;
            let returned = match result.value {
                KernelValue::Array(id) => Some(id),
                _ => None,
            }
            .expect("Owned result");
            let data = state
                .objects
                .array_data(returned, work)
                .expect("Returned data")
                .value;
            let string = match state
                .objects
                .array_get(data, ArrayKey::Index(0), &state.strings, work)
                .expect("Runtime payload")
                .value
            {
                Some(KernelValue::String(id)) => Some(id),
                _ => None,
            }
            .expect("String");
            assert_eq!(state.strings.get(string), Ok(b"leftright".as_slice()));
            assert_eq!(
                state
                    .objects
                    .array_owner_count(data, work)
                    .expect("Only transferred host owner")
                    .value,
                1
            );
            state
                .objects
                .release_array_value(returned, work)
                .expect("Host releases result");
        }
        assert_eq!(state.objects.usage().array_value_slots, 2);
        let reclaimed = state.objects.reclaim(&[], work).expect("Iterative cleanup");
        assert_eq!((reclaimed.objects, reclaimed.fields), (1, 1));
    }
}
