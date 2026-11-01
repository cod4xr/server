// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests every constructor/copy/publication/frame failure prefix and exact storage resources against retained host ownership."
)]
use std::fmt::Write;

use crate::array_read_support::number;
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{CompiledModule, ModuleCompilationBudget};
use cod4x_script::execution::{
    KernelCallBudget, KernelCallOutcome, KernelHeapContext, KernelHeapUsage, KernelModuleError,
};
use cod4x_script::heap::{
    ArrayKey, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapUsage, StringHeap,
    StringHeapBudget,
};

#[derive(Clone, Copy, Default)]
struct Scope {
    calls: KernelCallBudget,
    heap: HeapWorkBudget,
    objects: ObjectHeapBudget,
}

fn run(
    module: &CompiledModule,
    scope: Scope,
) -> (
    Result<KernelCallOutcome, KernelModuleError>,
    KernelHeapUsage,
    ObjectHeapUsage,
) {
    let work = HeapWorkBudget::default();
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, scope.objects).expect("Objects");
    let host = objects
        .new_array_value(work)
        .expect("Retained host array")
        .value;
    let data = objects.array_data(host, work).expect("Host data").value;
    objects
        .array_put_unique(host, ArrayKey::Index(0), number(5), &strings, work)
        .expect("Host payload");
    let mut context =
        KernelHeapContext::new(&mut strings, &[], scope.heap).with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(0, &[], scope.calls, &mut context);
    let usage = context.usage();
    let object_usage = context.object_heap_usage().expect("Objects");
    assert!(usage.steps <= scope.heap.max_steps);
    assert!(usage.peak_scratch_bytes <= scope.heap.max_scratch_bytes);
    assert_eq!(object_usage.live_array_values, 1);
    assert_eq!(object_usage.live_fields, 1);
    drop(context);
    assert_eq!(
        objects
            .array_owner_count(data, work)
            .expect("Only host owner")
            .value,
        1
    );
    assert_eq!(
        objects
            .array_get(data, ArrayKey::Index(0), &strings, work)
            .expect("Host payload unchanged")
            .value,
        Some(number(5))
    );
    (result, usage, object_usage)
}

#[test]
fn independent_constructor_callees_share_exact_admission_and_cleanup_every_failed_prefix() {
    let mut text = String::from("main(){");
    for index in 0..37 {
        write!(text, "a{index}=[];").expect("Growth source");
    }
    text.push_str("f=::relay;b=[[f]](a36);return b.size;}relay(a){discard=[];return a;}");
    let module = module_compile_support::compile(
        text.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Independent constructors with returned alias");
    let (result, heap, objects) = run(&module, Scope::default());
    let result = result.expect("Baseline");
    assert_eq!(result.value, number(0));
    let usage = result.usage;
    let exact = Scope {
        calls: KernelCallBudget {
            max_steps: usage.steps,
            max_setup_work: usage.setup_work,
            max_frames: usage.peak_frames,
            max_frame_bytes: usage.peak_frame_bytes,
            max_stack_values: usage.peak_stack_values,
            max_stack_bytes: usage.peak_stack_bytes,
            max_local_values: usage.peak_local_values,
            max_local_bytes: usage.peak_local_bytes,
        },
        heap: HeapWorkBudget {
            max_steps: heap.steps,
            max_scratch_bytes: heap.peak_scratch_bytes,
        },
        objects: ObjectHeapBudget {
            max_live_objects: objects.live_objects,
            max_object_slots: objects.object_slots,
            max_array_values: objects.array_value_slots,
            max_array_value_slots: objects.array_value_slots,
            max_metadata_bytes: objects.metadata_bytes,
            ..ObjectHeapBudget::default()
        },
    };
    assert_eq!(run(&module, exact).0.expect("All exact scopes"), result);
    for maximum in 0..heap.steps {
        let mut short = exact;
        short.heap.max_steps = maximum;
        run(&module, short)
            .0
            .expect_err("Every shorter heap-work prefix");
    }
    for maximum in 0..usage.steps {
        let mut short = exact;
        short.calls.max_steps = maximum;
        run(&module, short)
            .0
            .expect_err("Every shorter instruction prefix");
    }
    for maximum in 0..usage.setup_work {
        let mut short = exact;
        short.calls.max_setup_work = maximum;
        run(&module, short)
            .0
            .expect_err("Every shorter frame setup prefix");
    }
    for resource in 0..12 {
        let mut short = exact;
        match resource {
            0 => short.calls.max_frames -= 1,
            1 => short.calls.max_frame_bytes -= 1,
            2 => short.calls.max_stack_values -= 1,
            3 => short.calls.max_stack_bytes -= 1,
            4 => short.calls.max_local_values -= 1,
            5 => short.calls.max_local_bytes -= 1,
            6 => short.heap.max_scratch_bytes -= 1,
            7 => short.objects.max_live_objects -= 1,
            8 => short.objects.max_object_slots -= 1,
            9 => short.objects.max_array_values -= 1,
            10 => short.objects.max_array_value_slots -= 1,
            _ => short.objects.max_metadata_bytes -= 1,
        }
        run(&module, short)
            .0
            .expect_err("One-less frame/storage/scratch scope");
    }
}
