// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native helper constructor traces, not native allocator/ref ownership/VM conformance.
#![allow(
    clippy::expect_used,
    reason = "Fixture replay admits bounded native constructor traces."
)]
use std::collections::HashSet;

use crate::object_heap_support::stores;
use cod4x_script::execution::KernelValue;
use cod4x_script::execution::{
    KernelBuiltin, KernelErrorKind, KernelExecutionBudget, KernelHeapContext, KernelInstruction,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};

#[test]
fn native_fresh_empty_struct_constructor_traces_replay_through_checked_execution()
-> Result<(), String> {
    let code = [
        KernelInstruction::Builtin {
            function: KernelBuiltin::SpawnStruct,
            arguments: 0,
        },
        KernelInstruction::Return,
    ];
    let program = VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
        .expect("Constructor proof");
    assert_eq!(
        program
            .execute(KernelExecutionBudget::default())
            .expect_err("Explicit backing required")
            .kind,
        KernelErrorKind::ObjectExecutionUnsupported
    );
    let mut seen = HashSet::new();
    let mut rows = 0;
    for debug in 0..2 {
        let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
        for row in include_str!("fixtures/struct_vectors.txt")
            .lines()
            .filter(|row| !row.starts_with('#'))
        {
            let fields: Vec<_> = row.split_whitespace().collect();
            assert_eq!(fields.len(), 9);
            assert_eq!(fields[0], "S");
            let values: Vec<u64> = fields[1..]
                .iter()
                .map(|field| field.parse::<u64>())
                .collect::<Result<_, _>>()
                .map_err(|error| error.to_string())?;
            if values[0] != debug {
                continue;
            }
            assert_eq!(
                (values[2], values[3], values[4], values[6], values[7]),
                (1, 18, 0, debug, 1)
            );
            assert_eq!(values[1], values[5]);
            let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
                .with_objects(&mut objects, &[]);
            let result = program
                .execute_with_heap(&[], KernelExecutionBudget::default(), &mut context)
                .expect("Fresh constructor");
            let KernelValue::Object(id) = result.value else {
                return Err("Expected fresh struct".into());
            };
            assert!(seen.insert(id));
            assert_eq!(
                context
                    .object_heap_usage()
                    .expect("Object store")
                    .live_objects,
                values[1]
            );
            assert_eq!(objects.field_count(id), Ok(0));
            rows += 1;
        }
    }
    assert_eq!((rows, seen.len()), (32, 32));
    Ok(())
}

#[test]
fn constructor_accepts_65536_evaluated_extras_with_exact_stack_and_fuel_admission() {
    let arguments = 65_536;
    let mut code = vec![KernelInstruction::Constant(KernelValue::Undefined); arguments];
    code.extend([
        KernelInstruction::Builtin {
            function: KernelBuiltin::SpawnStruct,
            arguments,
        },
        KernelInstruction::Return,
    ]);
    let program = VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
        .expect("Wide constructor extras");
    let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let budget = KernelExecutionBudget {
        max_steps: arguments as u64 + 2,
        max_stack_values: arguments as u64,
        max_stack_bytes: (arguments * size_of::<KernelValue>()) as u64,
        ..KernelExecutionBudget::default()
    };
    let result = program
        .execute_with_heap(&[], budget, &mut context)
        .expect("Exact wide stack");
    assert!(matches!(result.value, KernelValue::Object(_)));
    assert_eq!(result.steps, budget.max_steps);
    assert_eq!(result.peak_stack_values, arguments as u64);
    assert_eq!(
        context
            .object_heap_usage()
            .expect("One result")
            .live_objects,
        1
    );
    let before = context.object_heap_usage();
    assert!(
        program
            .execute_with_heap(
                &[],
                KernelExecutionBudget {
                    max_steps: arguments as u64,
                    ..budget
                },
                &mut context
            )
            .is_err()
    );
    assert_eq!(context.object_heap_usage(), before);
    assert!(
        program
            .execute_with_heap(
                &[],
                KernelExecutionBudget {
                    max_stack_values: arguments as u64 - 1,
                    ..budget
                },
                &mut context
            )
            .is_err()
    );
    assert_eq!(context.object_heap_usage(), before);
}
