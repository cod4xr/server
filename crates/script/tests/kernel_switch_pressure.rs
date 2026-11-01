// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "A stack-only key read from the last array owner must survive collection before matching."
)]
use crate::kernel_call_support::{self, definition, integer};
use cod4x_script::execution::{
    KernelBuiltin, KernelCallBudget, KernelHeapContext, KernelHeapUsage, KernelInstruction as Op,
    KernelModuleError, KernelSwitchComparison, KernelSwitchKey, KernelValue, VerifiedKernelModule,
};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::{LegacyArithmetic, LegacyNumber};

fn module() -> VerifiedKernelModule {
    let entry = [
        integer(0),
        Op::Call {
            function: 1,
            arguments: 0,
        },
        Op::Index,
        Op::Builtin {
            function: KernelBuiltin::SpawnStruct,
            arguments: 0,
        },
        Op::Discard,
        Op::RequireSwitchKey,
        Op::SwitchCompare {
            key: KernelSwitchKey::StringLabel(0),
            comparison: KernelSwitchComparison::Equal,
        },
        Op::Return,
    ];
    let build = [
        Op::EmptyArray,
        Op::StoreLocal(0),
        Op::StringLiteral(0),
        Op::StringLiteral(1),
        Op::Arithmetic(LegacyArithmetic::Add),
        integer(0),
        Op::StoreArrayLocal(0),
        Op::LoadLocal(0),
        Op::Return,
    ];
    let labels = [b"leftright".to_vec()];
    let literals = [b"left".to_vec(), b"right".to_vec()];
    let mut first = definition(&entry, 0, 0);
    first.switch_labels = &labels;
    let mut second = definition(&build, 0, 1);
    second.string_literals = &literals;
    kernel_call_support::verify(&[first, second])
}

fn run(
    module: &VerifiedKernelModule,
    budget: HeapWorkBudget,
) -> (Result<(), KernelModuleError>, KernelHeapUsage) {
    let mut strings = StringHeap::new(StringHeapBudget {
        max_live_strings: 3,
        max_slots: 3,
        ..StringHeapBudget::default()
    })
    .expect("Two components plus runtime key");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 1,
            max_object_slots: 1,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Collection required");
    let mut context =
        KernelHeapContext::new(&mut strings, &[], budget).with_objects(&mut objects, &[]);
    let result = module.execute_with_heap(0, &[], KernelCallBudget::default(), &mut context);
    let usage = context.usage();
    assert!(usage.steps <= budget.max_steps);
    assert!(usage.peak_scratch_bytes <= budget.max_scratch_bytes);
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        0
    );
    let result = result.map(|result| {
        assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(1)));
        assert_eq!((usage.object_collections, usage.collections), (1, 1));
        assert_eq!(context.object_heap_usage().expect("Objects").live_arrays, 0);
        assert_eq!(
            context.heap_usage().slots,
            3,
            "Label bytes do not create a fourth string"
        );
    });
    (result, usage)
}

#[test]
fn stack_only_runtime_keys_survive_last_array_release_collection_and_every_shorter_heap_scope() {
    let module = module();
    let (result, usage) = run(&module, HeapWorkBudget::default());
    result.expect("Paired collection preserves key");
    let exact = HeapWorkBudget {
        max_steps: usage.steps,
        max_scratch_bytes: usage.peak_scratch_bytes,
    };
    run(&module, exact)
        .0
        .expect("Exact combined roots/marks scope");
    for maximum in 0..usage.steps {
        run(
            &module,
            HeapWorkBudget {
                max_steps: maximum,
                ..exact
            },
        )
        .0
        .expect_err("Every shorter key/root/collection prefix");
    }
    run(
        &module,
        HeapWorkBudget {
            max_scratch_bytes: exact.max_scratch_bytes - 1,
            ..exact
        },
    )
    .0
    .expect_err("One-less overlapping cache/owner/root/mark scratch");
}
