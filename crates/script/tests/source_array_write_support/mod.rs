// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Synthetic source invocations inspect explicit owned heaps."
)]
use cod4x_script::compilation::CompiledModule;
use cod4x_script::execution::{
    KernelCallBudget, KernelCallOutcome, KernelHeapContext, KernelHeapUsage, KernelModuleError,
    KernelValue,
};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};

pub struct State {
    pub strings: StringHeap,
    pub objects: ObjectHeap,
}

impl State {
    pub fn new(budget: ObjectHeapBudget) -> Self {
        let strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
        let objects = ObjectHeap::new(&strings, budget).expect("Objects");
        Self { strings, objects }
    }

    pub fn run(
        &mut self,
        module: &CompiledModule,
        arguments: &[KernelValue],
        calls: KernelCallBudget,
        heap: HeapWorkBudget,
    ) -> (
        Result<KernelCallOutcome, KernelModuleError>,
        KernelHeapUsage,
    ) {
        let mut context = KernelHeapContext::new(&mut self.strings, &[], heap)
            .with_objects(&mut self.objects, &[]);
        let result = module
            .program()
            .execute_with_heap(0, arguments, calls, &mut context);
        let usage = context.usage();
        assert!(usage.steps <= heap.max_steps);
        assert!(usage.peak_scratch_bytes <= heap.max_scratch_bytes);
        (result, usage)
    }
}
