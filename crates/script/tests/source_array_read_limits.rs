// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exhaust every work prefix and exact retained resources while preserving host array ownership."
)]
use crate::array_read_support::{self, number};
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{CompiledModule, ModuleCompilationBudget};
use cod4x_script::execution::{
    KernelCallBudget, KernelCallOutcome, KernelHeapUsage, KernelModuleError,
};
use cod4x_script::heap::HeapWorkBudget;

fn run(
    module: &CompiledModule,
    calls: KernelCallBudget,
    heap: HeapWorkBudget,
) -> (
    Result<KernelCallOutcome, KernelModuleError>,
    KernelHeapUsage,
) {
    let mut state = array_read_support::state(3, 7);
    let keys = state.keys();
    let mut context = state.context(heap);
    let result = module
        .program()
        .execute_with_heap(0, &[], calls, &mut context);
    let usage = context.usage();
    assert!(usage.steps <= heap.max_steps);
    assert!(usage.peak_scratch_bytes <= heap.max_scratch_bytes);
    drop(context);
    state.assert_host_owners(7);
    assert_eq!(state.keys(), keys);
    (result, usage)
}

#[test]
fn source_read_callees_admit_exact_resources_and_release_aliases_at_every_shorter_prefix() {
    let module = module_compile_support::compile(
        b"main(){a=game;return relay(a,0)+relay(a,65535)+relay(a,\"case\")+a.size;}
          relay(a,k){f=::take;return [[f]](a,k);}
          take(a,k){return isdefined(a[k]);}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Read across direct and indirect calls");
    let (baseline, heap) = run(
        &module,
        KernelCallBudget::default(),
        HeapWorkBudget::default(),
    );
    let baseline = baseline.expect("Baseline");
    assert_eq!(baseline.value, number(11));
    let usage = baseline.usage;
    let calls = KernelCallBudget {
        max_steps: usage.steps,
        max_setup_work: usage.setup_work,
        max_frames: usage.peak_frames,
        max_frame_bytes: usage.peak_frame_bytes,
        max_stack_values: usage.peak_stack_values,
        max_stack_bytes: usage.peak_stack_bytes,
        max_local_values: usage.peak_local_values,
        max_local_bytes: usage.peak_local_bytes,
    };
    let exact = HeapWorkBudget {
        max_steps: heap.steps,
        max_scratch_bytes: heap.peak_scratch_bytes,
    };
    assert_eq!(
        run(&module, calls, exact).0.expect("Exact admission"),
        baseline
    );
    for maximum in 0..heap.steps {
        run(
            &module,
            calls,
            HeapWorkBudget {
                max_steps: maximum,
                ..exact
            },
        )
        .0
        .expect_err("Every shorter heap-work prefix");
    }
    for maximum in 0..usage.steps {
        run(
            &module,
            KernelCallBudget {
                max_steps: maximum,
                ..calls
            },
            exact,
        )
        .0
        .expect_err("Every shorter instruction prefix");
    }
    for maximum in 0..usage.setup_work {
        run(
            &module,
            KernelCallBudget {
                max_setup_work: maximum,
                ..calls
            },
            exact,
        )
        .0
        .expect_err("Every shorter frame setup prefix");
    }
    for resource in 0..7 {
        let mut short = calls;
        match resource {
            0 => short.max_frames -= 1,
            1 => short.max_frame_bytes -= 1,
            2 => short.max_stack_values -= 1,
            3 => short.max_stack_bytes -= 1,
            4 => short.max_local_values -= 1,
            5 => short.max_local_bytes -= 1,
            _ => {
                run(
                    &module,
                    calls,
                    HeapWorkBudget {
                        max_scratch_bytes: exact.max_scratch_bytes - 1,
                        ..exact
                    },
                )
                .0
                .expect_err("One-less shared scratch");
                continue;
            }
        }
        run(&module, short, exact)
            .0
            .expect_err("One-less retained frame resource");
    }
}
