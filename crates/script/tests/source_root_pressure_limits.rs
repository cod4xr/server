// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exact combined cache/owned-array/root/mark scratch and every failed pressure work prefix."
)]
use crate::module_compile_support;
use crate::script_root_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{CompiledModule, ModuleCompilationBudget};
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelHeapUsage, KernelValue};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};
use cod4x_script::legacy::LegacyNumber;

fn execute(module: &CompiledModule, budget: HeapWorkBudget) -> (bool, KernelHeapUsage) {
    let mut state = script_root_support::state(ObjectHeapBudget {
        max_live_objects: 5,
        max_object_slots: 5,
        ..ObjectHeapBudget::default()
    });
    let work = HeapWorkBudget::default();
    state.objects.allocate(work).expect("Reclaimable garbage");
    let held = state
        .strings
        .allocate(b"private root-only payload", work)
        .expect("Held string")
        .id;
    state
        .objects
        .put(
            state.level,
            b"payload",
            KernelValue::String(held),
            &state.strings,
            work,
        )
        .expect("Root-only string");
    let roots = state.roots();
    let mut context = KernelHeapContext::new(&mut state.strings, &[], budget)
        .with_objects(&mut state.objects, &[])
        .with_script_roots(roots);
    let result =
        module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context);
    if let Ok(outcome) = &result {
        assert_eq!(outcome.value, KernelValue::Number(LegacyNumber::Integer(3)));
    }
    let usage = context.usage();
    assert!(usage.steps <= budget.max_steps);
    assert!(usage.peak_scratch_bytes <= budget.max_scratch_bytes);
    assert_eq!(
        context.get(held),
        Ok(b"private root-only payload".as_slice())
    );
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        1,
        "{result:?}"
    );
    drop(context);
    for root in [state.level, state.receiver, state.animation] {
        state
            .objects
            .field_count(root)
            .expect("Root must remain valid");
    }
    assert_eq!(
        state
            .objects
            .get(state.level, b"payload", &state.strings, work)
            .expect("Unchanged root payload")
            .value,
        Some(KernelValue::String(held))
    );
    (result.is_ok(), usage)
}

#[test]
fn combined_cached_literals_owned_game_aliases_and_world_roots_share_exact_pressure_admission() {
    let module = module_compile_support::compile(
        br#"main(){cached="PrivateCached";g=game;temp=spawnstruct(g);
        return isdefined(level.payload)+isdefined(g)+isdefined(cached);}"#,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Overlapping pressure roots");
    let (success, usage) = execute(&module, HeapWorkBudget::default());
    assert!(success);
    assert_eq!((usage.object_collections, usage.collections), (1, 1));
    let exact = HeapWorkBudget {
        max_steps: usage.steps,
        max_scratch_bytes: usage.peak_scratch_bytes,
    };
    assert!(execute(&module, exact).0);
    for maximum in 0..usage.steps {
        assert!(
            !execute(
                &module,
                HeapWorkBudget {
                    max_steps: maximum,
                    ..exact
                }
            )
            .0,
            "{maximum}"
        );
    }
    assert!(
        !execute(
            &module,
            HeapWorkBudget {
                max_scratch_bytes: exact.max_scratch_bytes - 1,
                ..exact
            }
        )
        .0
    );
}
