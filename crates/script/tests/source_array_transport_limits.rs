// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests measure exact admitted array transport and every shorter global failure scope."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{CompiledModule, ModuleCompilationBudget};
use cod4x_script::execution::{
    KernelCallBudget, KernelCallOutcome, KernelHeapContext, KernelHeapUsage, KernelModuleError,
    KernelValue,
};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapUsage, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::LegacyNumber;

#[derive(Clone, Copy, Default)]
struct Scope {
    calls: KernelCallBudget,
    heap: HeapWorkBudget,
    objects: ObjectHeapBudget,
}

fn program() -> CompiledModule {
    module_compile_support::compile(
        b"main(a){p=a;r=relay(a,a);p=undefined;return isdefined(r);}
          relay(left,right){f=::take;return [[f]](left,right);}
          take(x,y){unused=x;return y;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Direct/indirect transport source")
}

fn run(
    module: &CompiledModule,
    scope: Scope,
) -> (
    Result<KernelCallOutcome, KernelModuleError>,
    KernelHeapUsage,
    ObjectHeapUsage,
) {
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, scope.objects).expect("Objects");
    let host = objects
        .new_array_value(HeapWorkBudget::default())
        .expect("Host")
        .value;
    let data = objects
        .array_data(host, HeapWorkBudget::default())
        .expect("Data")
        .value;
    let mut context =
        KernelHeapContext::new(&mut strings, &[], scope.heap).with_objects(&mut objects, &[]);
    let result = module.program().execute_with_heap(
        0,
        &[KernelValue::Array(host)],
        scope.calls,
        &mut context,
    );
    let usage = context.usage();
    let heap_usage = context.object_heap_usage().expect("Objects");
    assert_eq!(heap_usage.live_array_values, 1, "{result:?}");
    assert!(usage.steps <= scope.heap.max_steps);
    assert!(usage.peak_scratch_bytes <= scope.heap.max_scratch_bytes);
    drop(context);
    assert_eq!(
        objects
            .array_owner_count(data, HeapWorkBudget::default())
            .expect("Host survives")
            .value,
        1
    );
    (result, usage, heap_usage)
}

#[test]
fn direct_and_indirect_transport_admit_exact_shared_resources_and_unwind_every_shorter_scope() {
    let module = program();
    let (result, heap, objects) = run(&module, Scope::default());
    let result = result.expect("Baseline");
    assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(1)));
    assert_eq!(result.usage.peak_frames, 3);
    let calls = result.usage;
    let exact = Scope {
        calls: KernelCallBudget {
            max_steps: calls.steps,
            max_setup_work: calls.setup_work,
            max_frames: calls.peak_frames,
            max_frame_bytes: calls.peak_frame_bytes,
            max_stack_values: calls.peak_stack_values,
            max_stack_bytes: calls.peak_stack_bytes,
            max_local_values: calls.peak_local_values,
            max_local_bytes: calls.peak_local_bytes,
        },
        heap: HeapWorkBudget {
            max_steps: heap.steps,
            max_scratch_bytes: heap.peak_scratch_bytes,
        },
        objects: ObjectHeapBudget {
            max_array_values: objects.array_value_slots,
            max_array_value_slots: objects.array_value_slots,
            ..ObjectHeapBudget::default()
        },
    };
    assert_eq!(run(&module, exact).0.expect("Exact scopes"), result);
    for limit in 0..heap.steps {
        let mut short = exact;
        short.heap.max_steps = limit;
        run(&module, short)
            .0
            .expect_err("Every shorter heap-work scope");
    }
    for limit in 0..calls.steps {
        let mut short = exact;
        short.calls.max_steps = limit;
        run(&module, short)
            .0
            .expect_err("Every shorter instruction scope");
    }
    for limit in 0..calls.setup_work {
        let mut short = exact;
        short.calls.max_setup_work = limit;
        run(&module, short)
            .0
            .expect_err("Every shorter setup scope");
    }
    for index in 0..8 {
        let mut short = exact;
        match index {
            0 => short.heap.max_scratch_bytes -= 1,
            1 => short.calls.max_frames -= 1,
            2 => short.calls.max_frame_bytes -= 1,
            3 => short.calls.max_stack_values -= 1,
            4 => short.calls.max_stack_bytes -= 1,
            5 => short.calls.max_local_values -= 1,
            6 => short.calls.max_local_bytes -= 1,
            _ => short.objects.max_array_value_slots -= 1,
        }
        run(&module, short)
            .0
            .expect_err("One-less retained resource");
    }
    for limit in 1..objects.array_value_slots {
        let mut short = exact;
        short.objects.max_array_values = limit;
        run(&module, short)
            .0
            .expect_err("Every shorter live-value scope");
    }
}

#[test]
fn ordinary_runtime_failure_releases_pending_operands_and_context_can_execute_again() {
    let module = module_compile_support::compile(
        b"bad(a){x=a;return a+1;} good(a){return isdefined(a);}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Failure and recovery source");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let host = objects
        .new_array_value(HeapWorkBudget::default())
        .expect("Host")
        .value;
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let arguments = [KernelValue::Array(host)];
    for _ in 0..64 {
        module
            .program()
            .execute_with_heap(0, &arguments, KernelCallBudget::default(), &mut context)
            .expect_err("Array arithmetic remains unsupported");
        assert_eq!(
            context
                .object_heap_usage()
                .expect("Objects")
                .live_array_values,
            1
        );
        let result = module
            .program()
            .execute_with_heap(1, &arguments, KernelCallBudget::default(), &mut context)
            .expect("Recovery with the same context");
        assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(1)));
        assert_eq!(
            context
                .object_heap_usage()
                .expect("Objects")
                .live_array_values,
            1
        );
    }
}
