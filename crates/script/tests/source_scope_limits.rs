// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exact and exhausted scope admissions."
)]
use crate::{module_compile_support, source_scope_support};
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{FunctionLoweringBudget, ModuleCompilationBudget};
use cod4x_script::execution::KernelCallBudget;
use cod4x_script::heap::HeapWorkBudget;

const SOURCE: &[u8] = b"main(){a=[];if(1){b=a;}for(i=0;i<2;i++){switch(i){
    case 0:if(1){d=a;break;}default:e=a;break;}}
    while((\"a\"+\"b\")==\"ab\"){if(i){f=a;break;}else{g=a;break;}}return 0;}";

#[test]
fn planning_cleanup_and_constant_folding_share_every_compiler_resource() {
    let module = source_scope_support::compile(SOURCE);
    let usage = module.functions()[0].usage();
    let exact = FunctionLoweringBudget {
        max_steps: usage.compiler_steps,
        max_instructions: usage.instructions,
        max_task_frames: usage.peak_task_frames,
        max_metadata_bytes: usage.peak_metadata_bytes,
        max_local_bindings: usage.local_bindings,
        max_identifier_bytes: usage.identifier_bytes,
        max_control_bytes: usage.peak_control_bytes,
        ..FunctionLoweringBudget::default()
    };
    let compile = |function| {
        module_compile_support::compile(
            SOURCE,
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget {
                function,
                ..ModuleCompilationBudget::default()
            },
        )
    };
    assert_eq!(
        compile(exact).expect("Exact resources").functions()[0].usage(),
        usage
    );
    for maximum in 0..usage.compiler_steps {
        compile(FunctionLoweringBudget {
            max_steps: maximum,
            ..exact
        })
        .expect_err("Every shorter planning/emission prefix");
    }
    for resource in 0..6 {
        let mut short = exact;
        match resource {
            0 => short.max_instructions -= 1,
            1 => short.max_task_frames -= 1,
            2 => short.max_metadata_bytes -= 1,
            3 => short.max_local_bindings -= 1,
            4 => short.max_identifier_bytes -= 1,
            _ => short.max_control_bytes -= 1,
        }
        compile(short).expect_err("One-less compiler resource");
    }
}

#[test]
fn every_cleanup_instruction_setup_and_heap_prefix_releases_all_vm_owners() {
    let module = source_scope_support::compile(SOURCE);
    let run = |calls, heap| source_scope_support::run(&module, 32, calls, heap);
    let baseline = run(KernelCallBudget::default(), HeapWorkBudget::default());
    let usage = baseline.result.expect("Baseline").usage;
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
    let heap = HeapWorkBudget {
        max_steps: baseline.heap.steps,
        max_scratch_bytes: baseline.heap.peak_scratch_bytes,
    };
    assert_eq!(
        run(calls, heap).result.expect("All exact resources").usage,
        usage
    );
    for maximum in 0..usage.steps {
        run(
            KernelCallBudget {
                max_steps: maximum,
                ..calls
            },
            heap,
        )
        .result
        .expect_err("Every shorter instruction prefix");
    }
    for maximum in 0..usage.setup_work {
        run(
            KernelCallBudget {
                max_setup_work: maximum,
                ..calls
            },
            heap,
        )
        .result
        .expect_err("Every shorter setup prefix");
    }
    for maximum in 0..heap.max_steps {
        run(
            calls,
            HeapWorkBudget {
                max_steps: maximum,
                ..heap
            },
        )
        .result
        .expect_err("Every shorter heap prefix");
    }
    for resource in 0..7 {
        let mut short = calls;
        let mut short_heap = heap;
        match resource {
            0 => short.max_frames -= 1,
            1 => short.max_frame_bytes -= 1,
            2 => short.max_stack_values -= 1,
            3 => short.max_stack_bytes -= 1,
            4 => short.max_local_values -= 1,
            5 => short.max_local_bytes -= 1,
            _ => short_heap.max_scratch_bytes -= 1,
        }
        run(short, short_heap)
            .result
            .expect_err("One-less runtime resource");
    }
}
