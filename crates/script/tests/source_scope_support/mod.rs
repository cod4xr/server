// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Synthetic scope fixtures use explicit stores and require complete exhausted-owner cleanup."
)]
use crate::{array_read_support::number, module_compile_support};
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{CompiledModule, ModuleCompilationBudget};
use cod4x_script::execution::{
    KernelCallBudget, KernelCallOutcome, KernelErrorKind, KernelHeapContext, KernelHeapUsage,
    KernelModuleError,
};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind, ObjectHeapUsage,
    ObjectResource, StringHeap, StringHeapBudget,
};

pub fn compile(source: &[u8]) -> CompiledModule {
    module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Reviewed local-scope source")
}

pub struct Run {
    pub result: Result<KernelCallOutcome, KernelModuleError>,
    pub heap: KernelHeapUsage,
    pub objects: ObjectHeapUsage,
}

pub fn run(
    module: &CompiledModule,
    maximum: u64,
    calls: KernelCallBudget,
    heap: HeapWorkBudget,
) -> Run {
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_array_values: maximum,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Objects");
    let mut context =
        KernelHeapContext::new(&mut strings, &[], heap).with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(0, &[], calls, &mut context);
    let heap = context.usage();
    let usage = context.object_heap_usage().expect("Objects");
    assert_eq!(
        usage.live_array_values, 0,
        "No successful scalar return or failure retains a VM owner"
    );
    drop(context);
    assert_eq!(
        objects
            .reclaim(&[], HeapWorkBudget::default())
            .expect("All unowned data")
            .objects,
        usage.live_objects
    );
    Run {
        result,
        heap,
        objects: usage,
    }
}

pub fn succeeds(source: &[u8], maximum: u64) -> Run {
    let result = run(
        &compile(source),
        maximum,
        KernelCallBudget::default(),
        HeapWorkBudget::default(),
    );
    assert_eq!(
        result.result.as_ref().expect("Scoped owners fit").value,
        number(0)
    );
    result
}

pub fn retains(source: &[u8], maximum: u64) {
    let result = run(
        &compile(source),
        maximum,
        KernelCallBudget::default(),
        HeapWorkBudget::default(),
    );
    assert_eq!(
        result
            .result
            .expect_err("A public alias remains live")
            .cause
            .kind,
        KernelErrorKind::ObjectHeap(ObjectHeapErrorKind::BudgetExceeded {
            resource: ObjectResource::ArrayValues,
            requested: maximum + 1,
            maximum,
        })
    );
}
