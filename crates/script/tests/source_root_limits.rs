// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exhaust shared root-read and ownership budgets and verify original bindings survive every failure."
)]
use crate::module_compile_support;
use crate::script_root_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{CompiledModule, ModuleCompilationBudget};
use cod4x_script::execution::{
    KernelCallBudget, KernelCallOutcome, KernelCallUsage, KernelHeapContext, KernelHeapUsage,
    KernelModuleError,
};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget, ObjectHeapUsage};

fn execute(
    module: &CompiledModule,
    heap: HeapWorkBudget,
    calls: KernelCallBudget,
    objects: ObjectHeapBudget,
) -> (
    Result<KernelCallOutcome, KernelModuleError>,
    KernelHeapUsage,
    ObjectHeapUsage,
) {
    let mut state = script_root_support::state(objects);
    let roots = state.roots();
    let data = state
        .objects
        .array_data(state.game, HeapWorkBudget::default())
        .expect("Host data")
        .value;
    let mut context = KernelHeapContext::new(&mut state.strings, &[], heap)
        .with_objects(&mut state.objects, &[])
        .with_script_roots(roots);
    let result = module
        .program()
        .execute_with_heap(0, &[], calls, &mut context);
    let usage = context.usage();
    assert!(usage.steps <= heap.max_steps);
    assert!(usage.peak_scratch_bytes <= heap.max_scratch_bytes);
    let objects = context.object_heap_usage().expect("Objects");
    assert_eq!(objects.live_array_values, 1, "{result:?}");
    drop(context);
    for root in [state.level, state.receiver, state.animation] {
        assert_eq!(state.objects.field_count(root), Ok(0));
    }
    assert_eq!(
        state
            .objects
            .array_owner_count(data, HeapWorkBudget::default())
            .expect("Host game survives")
            .value,
        1
    );
    (result, usage, objects)
}

fn program() -> CompiledModule {
    module_compile_support::compile(
        b"main(){a=game;x=relay(a);return isdefined(x)+isdefined(level)+isdefined(self)+isdefined(anim);}
          relay(a){b=game;return choose(a,b);}choose(a,b){if(isdefined(a))return b;return game;}",
        LegacyDeveloperMode::Disabled, ModuleCompilationBudget::default(),
    ).expect("Root transport across calls")
}

#[test]
fn exact_root_scopes_pass_and_every_shorter_work_instruction_setup_or_value_limit_unwinds() {
    let module = program();
    let (result, heap, objects) = execute(
        &module,
        HeapWorkBudget::default(),
        KernelCallBudget::default(),
        ObjectHeapBudget::default(),
    );
    let result = result.expect("Baseline");
    let exact_heap = HeapWorkBudget {
        max_steps: heap.steps,
        max_scratch_bytes: heap.peak_scratch_bytes,
    };
    let exact_calls = call_budget(result.usage);
    let exact_objects = ObjectHeapBudget {
        max_array_values: objects.array_value_slots,
        max_array_value_slots: objects.array_value_slots,
        ..ObjectHeapBudget::default()
    };
    assert_eq!(
        execute(&module, exact_heap, exact_calls, exact_objects)
            .0
            .expect("Exact scopes"),
        result
    );
    for maximum in 0..heap.steps {
        execute(
            &module,
            HeapWorkBudget {
                max_steps: maximum,
                ..exact_heap
            },
            exact_calls,
            exact_objects,
        )
        .0
        .expect_err("Every shorter root/heap work scope");
    }
    for maximum in 0..result.usage.steps {
        execute(
            &module,
            exact_heap,
            KernelCallBudget {
                max_steps: maximum,
                ..exact_calls
            },
            exact_objects,
        )
        .0
        .expect_err("Every shorter instruction scope");
    }
    for maximum in 0..result.usage.setup_work {
        execute(
            &module,
            exact_heap,
            KernelCallBudget {
                max_setup_work: maximum,
                ..exact_calls
            },
            exact_objects,
        )
        .0
        .expect_err("Every shorter setup scope");
    }
    execute(
        &module,
        HeapWorkBudget {
            max_scratch_bytes: exact_heap.max_scratch_bytes - 1,
            ..exact_heap
        },
        exact_calls,
        exact_objects,
    )
    .0
    .expect_err("One-less simultaneous scratch");
    execute(
        &module,
        exact_heap,
        exact_calls,
        ObjectHeapBudget {
            max_array_value_slots: exact_objects.max_array_value_slots - 1,
            ..exact_objects
        },
    )
    .0
    .expect_err("One-less value slot");
    for maximum in 1..objects.array_value_slots {
        execute(
            &module,
            exact_heap,
            exact_calls,
            ObjectHeapBudget {
                max_array_values: maximum,
                ..exact_objects
            },
        )
        .0
        .expect_err("Every shorter live owner quota");
    }
}

fn call_budget(usage: KernelCallUsage) -> KernelCallBudget {
    KernelCallBudget {
        max_steps: usage.steps,
        max_setup_work: usage.setup_work,
        max_frames: usage.peak_frames,
        max_frame_bytes: usage.peak_frame_bytes,
        max_stack_values: usage.peak_stack_values,
        max_stack_bytes: usage.peak_stack_bytes,
        max_local_values: usage.peak_local_values,
        max_local_bytes: usage.peak_local_bytes,
    }
}
