// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests isolate every live object and nested string root during pressure."
)]
use crate::kernel_call_support::{self, definition};
use crate::kernel_heap_support;
use crate::object_heap_support::allocate;
use cod4x_script::execution::{
    KernelCallBudget, KernelHeapContext, KernelInstruction, KernelValue,
};
use cod4x_script::heap::{HeapErrorKind, HeapWorkBudget, ObjectHeap, ObjectHeapBudget};
use cod4x_script::legacy::LegacyArithmetic;

#[test]
fn object_collection_preserves_caller_stack_nested_strings_external_objects_and_cached_only_literals()
 {
    let clear = KernelInstruction::Constant(KernelValue::Undefined);
    let entry_code = [
        KernelInstruction::LoadLocal(0),
        clear,
        KernelInstruction::StoreLocal(0),
        KernelInstruction::LoadLocal(1),
        clear,
        KernelInstruction::StoreLocal(1),
        KernelInstruction::Call {
            function: 1,
            arguments: 1,
        },
        KernelInstruction::Discard,
        KernelInstruction::Return,
    ];
    let worker_code = [
        KernelInstruction::StringLiteral(0),
        KernelInstruction::Discard,
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadLocal(0),
        clear,
        KernelInstruction::StoreLocal(0),
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        KernelInstruction::Return,
    ];
    let literals = [b"cache".to_vec()];
    let mut definitions = [
        definition(&entry_code, 2, 2),
        definition(&worker_code, 1, 1),
    ];
    definitions[1].string_literals = &literals;
    let module = kernel_call_support::verify(&definitions);
    let mut strings = kernel_heap_support::heap(8);
    let ids: Vec<_> = [
        b"plain".as_slice(),
        b"MENU",
        b"external",
        b"cycle",
        b"x",
        b"host",
        b"garbage",
    ]
    .iter()
    .map(|bytes| kernel_heap_support::allocate(&mut strings, bytes))
    .collect();
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let root = allocate(&mut objects);
    let child = allocate(&mut objects);
    let external = allocate(&mut objects);
    let garbage = allocate(&mut objects);
    for (object, key, value) in [
        (root, b"child".as_slice(), KernelValue::Object(child)),
        (root, b"plain", KernelValue::String(ids[0])),
        (child, b"localized", KernelValue::LocalizedString(ids[1])),
        (external, b"value", KernelValue::String(ids[2])),
        (garbage, b"self", KernelValue::Object(garbage)),
        (garbage, b"value", KernelValue::String(ids[3])),
    ] {
        objects
            .put(object, key, value, &strings, HeapWorkBudget::default())
            .expect("Graph field");
    }
    let external_strings = [ids[5]];
    let external_objects = [external];
    let mut context =
        KernelHeapContext::new(&mut strings, &external_strings, HeapWorkBudget::default())
            .with_objects(&mut objects, &external_objects);
    let result = module
        .execute_with_heap(
            0,
            &[KernelValue::Object(root), KernelValue::String(ids[4])],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("Caller stack is complete root");
    assert_eq!(result.value, KernelValue::Object(root));
    assert_eq!(
        (
            context.usage().collections,
            context.usage().object_collections
        ),
        (1, 1)
    );
    assert_eq!(context.heap_usage().live_strings, 7);
    assert_eq!(
        context.object_heap_usage().expect("Objects").live_objects,
        3
    );
    for id in [ids[0], ids[1], ids[2], ids[4], ids[5]] {
        assert!(context.get(id).is_ok());
    }
    for id in [ids[3], ids[6]] {
        assert_eq!(context.get(id), Err(HeapErrorKind::StaleReference));
    }
}

#[test]
fn object_roots_in_12000_iterative_frames_keep_nested_strings_alive_during_pressure() {
    let count = 12_000;
    let mut code = Vec::new();
    for function in 0..count {
        code.push(if function + 1 == count {
            vec![
                KernelInstruction::StringLiteral(0),
                KernelInstruction::StringLiteral(0),
                KernelInstruction::Arithmetic(LegacyArithmetic::Add),
                KernelInstruction::Discard,
                KernelInstruction::LoadLocal(0),
                KernelInstruction::Return,
            ]
        } else {
            vec![
                KernelInstruction::LoadLocal(0),
                KernelInstruction::Call {
                    function: function + 1,
                    arguments: 1,
                },
                KernelInstruction::Return,
            ]
        });
    }
    let literals = [b"x".to_vec()];
    let mut definitions: Vec<_> = code.iter().map(|body| definition(body, 1, 1)).collect();
    definitions[count - 1].string_literals = &literals;
    let module = kernel_call_support::verify(&definitions);
    let mut strings = kernel_heap_support::heap(3);
    let held = kernel_heap_support::allocate(&mut strings, b"held");
    let garbage = kernel_heap_support::allocate(&mut strings, b"old");
    kernel_heap_support::allocate(&mut strings, b"other");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let root = allocate(&mut objects);
    objects
        .put(
            root,
            b"held",
            KernelValue::String(held),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Nested root");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let result = module
        .execute_with_heap(
            0,
            &[KernelValue::Object(root)],
            KernelCallBudget {
                max_setup_work: 100_000_000,
                ..KernelCallBudget::default()
            },
            &mut context,
        )
        .expect("Deep object-aware calls");
    assert_eq!(result.value, KernelValue::Object(root));
    assert_eq!(result.usage.peak_frames, count as u64);
    assert_eq!(context.get(held).expect("Nested string"), b"held");
    assert_eq!(context.get(garbage), Err(HeapErrorKind::StaleReference));
    assert_eq!(
        (
            context.usage().collections,
            context.usage().object_collections
        ),
        (1, 1)
    );
}
