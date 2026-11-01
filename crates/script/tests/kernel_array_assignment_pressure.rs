// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests runtime RHS graphs held only on the operand stack through COW and insertion pressure retries."
)]
use crate::kernel_call_support::{self, definition, integer};
use crate::object_heap_support::{number, stores};
use cod4x_script::execution::{
    KernelBuiltin, KernelCallBudget, KernelHeapContext, KernelHeapUsage, KernelInstruction,
    KernelModuleError, KernelValue, VerifiedKernelModule,
};
use cod4x_script::heap::{
    ArrayId, ArrayKey, ArrayValueId, HeapErrorKind, HeapWorkBudget, ObjectHeap, ObjectHeapBudget,
    StringHeap,
};
use cod4x_script::legacy::LegacyArithmetic;

#[derive(Clone, Copy)]
enum Pressure {
    Copy,
    Insertion,
}

fn module(pressure: Pressure) -> VerifiedKernelModule {
    let key = i32::from(matches!(pressure, Pressure::Insertion));
    let code = [
        KernelInstruction::Builtin {
            function: KernelBuiltin::SpawnStruct,
            arguments: 0,
        },
        KernelInstruction::StoreLocal(1),
        KernelInstruction::StringLiteral(0),
        KernelInstruction::StringLiteral(1),
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::StoreField(0),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::StoreLocal(1),
        integer(key),
        KernelInstruction::StoreArrayLocal(0),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Return,
    ];
    let literals = [b"left".to_vec(), b"right".to_vec()];
    let fields = [b"payload".to_vec()];
    let mut entry = definition(&code, 1, 2);
    entry.string_literals = &literals;
    entry.field_names = &fields;
    kernel_call_support::verify(&[entry])
}

fn run(
    module: &VerifiedKernelModule,
    pressure: Pressure,
    budget: HeapWorkBudget,
) -> (Result<(), KernelModuleError>, KernelHeapUsage) {
    let (mut strings, mut objects) = stores(ObjectHeapBudget {
        max_live_fields: if matches!(pressure, Pressure::Copy) {
            3
        } else {
            4
        },
        ..ObjectHeapBudget::default()
    });
    let work = HeapWorkBudget::default();
    let discarded = strings
        .allocate(b"unreachable", work)
        .expect("Garbage payload")
        .id;
    let host = objects.new_array_value(work).expect("Host target").value;
    let data = objects.array_data(host, work).expect("Host data").value;
    objects
        .array_put_unique(host, ArrayKey::Index(0), number(7), &strings, work)
        .expect("Host field");
    let garbage = objects.allocate(work).expect("Garbage object").id;
    objects
        .put(
            garbage,
            b"discard",
            KernelValue::String(discarded),
            &strings,
            work,
        )
        .expect("Garbage field");
    let mut context =
        KernelHeapContext::new(&mut strings, &[], budget).with_objects(&mut objects, &[]);
    let result = module.execute_with_heap(
        0,
        &[KernelValue::Array(host)],
        KernelCallBudget::default(),
        &mut context,
    );
    let usage = context.usage();
    assert!(usage.steps <= budget.max_steps);
    assert!(usage.peak_scratch_bytes <= budget.max_scratch_bytes);
    let returned = result.as_ref().ok().map(|outcome| {
        match outcome.value {
            KernelValue::Array(id) => Some(id),
            _ => None,
        }
        .expect("Returned local array owner")
    });
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        1 + u64::from(returned.is_some())
    );
    if returned.is_some() {
        assert_eq!((usage.object_collections, usage.collections), (1, 1));
        assert_eq!(context.get(discarded), Err(HeapErrorKind::StaleReference));
        assert_eq!(context.heap_usage().live_strings, 3);
    }
    drop(context);
    if let Some(returned) = returned {
        let key = u64::from(matches!(pressure, Pressure::Insertion));
        assert_ne!(rhs_data(&objects, &strings, returned, key), data);
        objects
            .release_array_value(returned, work)
            .expect("Host releases result");
    }
    objects
        .reclaim(&[], work)
        .expect("All nonhost data is unowned after success/failure");
    assert_eq!(
        (
            objects.usage().live_objects,
            objects.usage().live_fields,
            objects.usage().live_array_values
        ),
        (1, 1, 1)
    );
    assert_eq!(
        objects
            .array_get(data, ArrayKey::Index(0), &strings, work)
            .expect("Original host data")
            .value,
        Some(number(7))
    );
    (result.map(|_| ()), usage)
}

fn rhs_data(objects: &ObjectHeap, strings: &StringHeap, value: ArrayValueId, key: u64) -> ArrayId {
    let work = HeapWorkBudget::default();
    let data = objects
        .array_data(value, work)
        .expect("Selected data")
        .value;
    let object = match objects
        .array_get(data, ArrayKey::Index(key), strings, work)
        .expect("Stored RHS")
        .value
    {
        Some(KernelValue::Object(id)) => Some(id),
        _ => None,
    }
    .expect("Shared RHS identity");
    let string = match objects
        .get(object, b"payload", strings, work)
        .expect("RHS graph")
        .value
    {
        Some(KernelValue::String(id)) => Some(id),
        _ => None,
    }
    .expect("Runtime concatenation");
    assert_eq!(strings.get(string), Ok(b"leftright".as_slice()));
    data
}

#[test]
fn rhs_stack_only_struct_string_graph_survives_copy_pressure_with_every_shorter_work_prefix() {
    check(Pressure::Copy);
}

#[test]
fn rhs_stack_only_struct_string_graph_survives_insertion_pressure_with_every_shorter_work_prefix() {
    check(Pressure::Insertion);
}

fn check(pressure: Pressure) {
    let module = module(pressure);
    let (result, usage) = run(&module, pressure, HeapWorkBudget::default());
    result.expect("Baseline pressure retry");
    let exact = HeapWorkBudget {
        max_steps: usage.steps,
        max_scratch_bytes: usage.peak_scratch_bytes,
    };
    run(&module, pressure, exact)
        .0
        .expect("Exact combined scratch/work");
    for maximum in 0..usage.steps {
        run(
            &module,
            pressure,
            HeapWorkBudget {
                max_steps: maximum,
                ..exact
            },
        )
        .0
        .expect_err("Every shorter COW/field/root/cleanup work prefix");
    }
    run(
        &module,
        pressure,
        HeapWorkBudget {
            max_scratch_bytes: exact.max_scratch_bytes - 1,
            ..exact
        },
    )
    .0
    .expect_err("One less combined scratch");
}
