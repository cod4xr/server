// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests independent populated results with no host/cache backing across last-array release and paired reclamation."
)]
use crate::kernel_call_support::{self, definition, integer};
use cod4x_script::execution::{
    KernelBuiltin, KernelCallBudget, KernelCallOutcome, KernelHeapContext, KernelHeapUsage,
    KernelInstruction, KernelModuleError, KernelValue, VerifiedKernelModule,
};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::LegacyArithmetic;

#[derive(Clone, Copy)]
enum Payload {
    SharedObject,
    RuntimeString,
}

fn module(payload: Payload) -> VerifiedKernelModule {
    let spawn = KernelInstruction::Builtin {
        function: KernelBuiltin::SpawnStruct,
        arguments: 0,
    };
    let entry = [
        integer(0),
        KernelInstruction::Call {
            function: 1,
            arguments: 0,
        },
        KernelInstruction::Index,
        KernelInstruction::Call {
            function: 2,
            arguments: 1,
        },
        KernelInstruction::Return,
    ];
    let mut build = vec![
        KernelInstruction::EmptyArray,
        KernelInstruction::StoreLocal(0),
    ];
    if matches!(payload, Payload::SharedObject) {
        build.extend([spawn, KernelInstruction::StoreLocal(1)]);
    }
    build.extend([
        KernelInstruction::StringLiteral(0),
        KernelInstruction::StringLiteral(1),
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
    ]);
    if matches!(payload, Payload::SharedObject) {
        build.extend([
            KernelInstruction::LoadLocal(1),
            KernelInstruction::StoreField(0),
            KernelInstruction::LoadLocal(1),
        ]);
    }
    build.extend([integer(0), KernelInstruction::StoreArrayLocal(0)]);
    if matches!(payload, Payload::SharedObject) {
        build.extend([
            KernelInstruction::Constant(KernelValue::Undefined),
            KernelInstruction::StoreLocal(1),
        ]);
    }
    build.extend([KernelInstruction::LoadLocal(0), KernelInstruction::Return]);
    let mut consume = vec![
        spawn,
        KernelInstruction::Discard,
        KernelInstruction::LoadLocal(0),
    ];
    if matches!(payload, Payload::SharedObject) {
        consume.push(KernelInstruction::LoadField(0));
    }
    consume.push(KernelInstruction::Return);
    let literals = [b"left".to_vec(), b"right".to_vec()];
    let fields = [b"payload".to_vec()];
    let mut definitions = [
        definition(&entry, 0, 0),
        definition(&build, 0, 2),
        definition(&consume, 1, 1),
    ];
    definitions[1].string_literals = &literals;
    if matches!(payload, Payload::SharedObject) {
        definitions[1].field_names = &fields;
        definitions[2].field_names = &fields;
    }
    kernel_call_support::verify(&definitions)
}

fn run(
    module: &VerifiedKernelModule,
    payload: Payload,
    budget: HeapWorkBudget,
) -> (
    Result<KernelCallOutcome, KernelModuleError>,
    KernelHeapUsage,
) {
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let slots = if matches!(payload, Payload::SharedObject) {
        2
    } else {
        1
    };
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: slots,
            max_object_slots: slots,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("No room until the last array data is reclaimed");
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
    if let Ok(outcome) = &result {
        let value = match outcome.value {
            KernelValue::String(value) => Some(value),
            _ => None,
        }
        .expect("Copied runtime string");
        assert_eq!(context.get(value), Ok(b"leftright".as_slice()));
        assert_eq!((usage.object_collections, usage.collections), (1, 1));
        assert_eq!(context.object_heap_usage().expect("Objects").live_arrays, 0);
        assert_eq!(
            context.heap_usage().live_strings,
            3,
            "Two literals and the frame-only concatenation"
        );
    }
    (result, usage)
}

#[test]
fn last_array_owner_can_drop_before_a_shared_object_result_survives_callee_collection() {
    check(Payload::SharedObject);
}

#[test]
fn last_array_owner_can_drop_before_a_runtime_string_result_survives_callee_collection() {
    check(Payload::RuntimeString);
}

fn check(payload: Payload) {
    let module = module(payload);
    let (baseline, usage) = run(&module, payload, HeapWorkBudget::default());
    baseline.expect("Complete baseline");
    let exact = HeapWorkBudget {
        max_steps: usage.steps,
        max_scratch_bytes: usage.peak_scratch_bytes,
    };
    run(&module, payload, exact)
        .0
        .expect("Exact combined cache/owner/frame/mark scope");
    for maximum in 0..usage.steps {
        run(
            &module,
            payload,
            HeapWorkBudget {
                max_steps: maximum,
                ..exact
            },
        )
        .0
        .expect_err("Every shorter heap-work prefix releases all VM owners");
    }
    run(
        &module,
        payload,
        HeapWorkBudget {
            max_scratch_bytes: exact.max_scratch_bytes - 1,
            ..exact
        },
    )
    .0
    .expect_err("One less combined scratch");
}
