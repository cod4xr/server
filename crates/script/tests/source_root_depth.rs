// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests first game-owner creation at deep frame depth and fuel-independent deep failure cleanup."
)]
use crate::module_compile_support;
use crate::script_root_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn first_game_read_at_twelve_thousand_frames_transfers_and_failed_return_releases_every_owner()
-> Result<(), String> {
    let module = module_compile_support::compile(
        b"descend(n){if(n==0)return game;return descend(n-1);}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Deep root source");
    let mut maximum = None;
    for fail in [false, true] {
        let mut state = script_root_support::state(ObjectHeapBudget::default());
        let roots = state.roots();
        let data = state
            .objects
            .array_data(state.game, HeapWorkBudget::default())
            .expect("Game data")
            .value;
        let budget = HeapWorkBudget {
            max_steps: maximum.unwrap_or(64_000_000),
            ..HeapWorkBudget::default()
        };
        let mut context = KernelHeapContext::new(&mut state.strings, &[], budget)
            .with_objects(&mut state.objects, &[])
            .with_script_roots(roots);
        let result = module.program().execute_with_heap(
            0,
            &[KernelValue::Number(LegacyNumber::Integer(12_000))],
            KernelCallBudget {
                max_setup_work: 100_000_000,
                ..KernelCallBudget::default()
            },
            &mut context,
        );
        if fail {
            let error = result.expect_err("Final deep return lacks one heap-work unit");
            assert_eq!(error.peak_frames, 12_001);
            assert_eq!(
                context
                    .object_heap_usage()
                    .expect("Objects")
                    .live_array_values,
                1
            );
            drop(context);
        } else {
            let result = result.expect("12,001 flat frames with first array at the leaf");
            assert_eq!(result.usage.peak_frames, 12_001);
            let KernelValue::Array(returned) = result.value else {
                return Err("Expected game result".into());
            };
            maximum = Some(context.usage().steps - 1);
            assert_eq!(
                context
                    .object_heap_usage()
                    .expect("Objects")
                    .live_array_values,
                2
            );
            drop(context);
            assert_eq!(
                state
                    .objects
                    .array_data(returned, HeapWorkBudget::default())
                    .expect("Return data")
                    .value,
                data
            );
            state
                .objects
                .release_array_value(returned, HeapWorkBudget::default())
                .expect("Host releases returned owner");
        }
        assert_eq!(
            state
                .objects
                .array_owner_count(data, HeapWorkBudget::default())
                .expect("Original host root")
                .value,
            1
        );
        assert_eq!(state.objects.usage().array_value_slots, 2);
    }
    Ok(())
}
