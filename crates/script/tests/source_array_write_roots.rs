// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Source-produced graphs must survive the last array owner and later pressure."
)]
use crate::{source_array_write_support::State, source_scope_support::compile};
use cod4x_script::compilation::CompiledModule;
use cod4x_script::execution::{KernelCallBudget, KernelHeapUsage, KernelModuleError, KernelValue};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};

fn run(
    module: &CompiledModule,
    slots: u64,
    budget: HeapWorkBudget,
) -> (Result<(), KernelModuleError>, KernelHeapUsage) {
    let mut state = State::new(ObjectHeapBudget {
        max_live_objects: slots,
        max_object_slots: slots,
        ..ObjectHeapBudget::default()
    });
    let (result, usage) = state.run(module, &[], KernelCallBudget::default(), budget);
    assert_eq!(state.objects.usage().live_array_values, 0);
    let result = result.map(|result| {
        let value = match result.value {
            KernelValue::String(id) => Some(id),
            _ => None,
        }
        .expect("Runtime string result");
        assert_eq!(state.strings.get(value), Ok(b"leftright".as_slice()));
        assert_eq!((usage.object_collections, usage.collections), (1, 1));
        assert_eq!(state.objects.usage().live_arrays, 0);
    });
    (result, usage)
}

#[test]
fn independently_populated_source_results_remain_rooted_after_last_array_release_and_callee_collection()
 {
    for (source, slots) in [
        (b"main(){return consume(make()[0]);}make(){a=[];a[0]=\"left\"+\"right\";return a;}
            consume(p){spawnstruct();return p;}".as_slice(), 1),
        (b"main(){return consume(make()[0]);}make(){a=[];o=spawnstruct();o.payload=\"left\"+\"right\";
            a[0]=o;o=undefined;return a;}consume(p){spawnstruct();return p.payload;}", 2),
    ] {
        let module = compile(source);
        let (result, usage) = run(&module, slots, HeapWorkBudget::default());
        result.expect("No host or cached full-string backing");
        let exact = HeapWorkBudget { max_steps: usage.steps, max_scratch_bytes: usage.peak_scratch_bytes };
        run(&module, slots, exact).0.expect("Exact combined roots/marks scope");
        for maximum in 0..usage.steps {
            run(&module, slots, HeapWorkBudget { max_steps: maximum, ..exact }).0
                .expect_err("Every shorter graph publication/collection prefix");
        }
        run(&module, slots, HeapWorkBudget { max_scratch_bytes: exact.max_scratch_bytes - 1, ..exact }).0
            .expect_err("One-less overlapping frame/cache/owner/mark scratch");
    }
}
