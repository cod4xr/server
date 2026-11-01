// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests measure independent immutable string/name tables and exact proof budgets."
)]
use crate::kernel_call_support::{definition, integer};
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelInstruction,
    KernelModuleVerificationBudget, KernelResource, KernelValue, KernelVerificationBudget,
    VerifiedKernelModule, VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};

#[test]
fn field_and_string_tables_share_exact_proof_work_and_metadata_across_distinct_functions() {
    let code = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadField(0),
        KernelInstruction::Return,
    ];
    let strings = [b"unused literal".to_vec()];
    let fields = [b"owned field".to_vec(), b"unused field".to_vec()];
    let mut function = definition(&code, 1, 1);
    function.string_literals = &strings;
    function.field_names = &fields;
    let program =
        VerifiedKernelProgram::verify_definition(function, KernelVerificationBudget::default())
            .expect("Measured proof");
    let usage = program.usage();
    assert_eq!(usage.field_names, 2);
    assert_eq!(
        usage.retained_field_bytes,
        2 * size_of::<Vec<u8>>() as u64 + 23
    );
    let exact = KernelVerificationBudget {
        max_steps: usage.verification_steps,
        max_metadata_bytes: usage.verification_metadata_bytes,
        ..KernelVerificationBudget::default()
    };
    assert_eq!(
        VerifiedKernelProgram::verify_definition(function, exact)
            .expect("Exact proof")
            .usage(),
        usage
    );
    for (max_steps, max_metadata_bytes, resource) in [
        (
            exact.max_steps - 1,
            exact.max_metadata_bytes,
            KernelResource::VerificationSteps,
        ),
        (
            exact.max_steps,
            exact.max_metadata_bytes - 1,
            KernelResource::VerificationMetadataBytes,
        ),
    ] {
        let error = VerifiedKernelProgram::verify_definition(
            function,
            KernelVerificationBudget {
                max_steps,
                max_metadata_bytes,
                ..exact
            },
        )
        .expect_err("One less proof resource");
        assert!(
            matches!(error.kind, KernelErrorKind::BudgetExceeded {resource: actual, requested, maximum} if actual == resource && requested > maximum)
        );
    }
    let definitions = [function, function];
    let usage =
        VerifiedKernelModule::verify(&definitions, KernelModuleVerificationBudget::default())
            .expect("Global proof")
            .usage();
    assert_eq!((usage.field_names, usage.string_literals), (4, 2));
    let exact = KernelModuleVerificationBudget {
        max_steps: usage.verification_steps,
        max_metadata_bytes: usage.verification_metadata_bytes,
        ..KernelModuleVerificationBudget::default()
    };
    assert_eq!(
        VerifiedKernelModule::verify(&definitions, exact)
            .expect("Exact global proof")
            .usage(),
        usage
    );
    for (max_steps, max_metadata_bytes) in [
        (exact.max_steps - 1, exact.max_metadata_bytes),
        (exact.max_steps, exact.max_metadata_bytes - 1),
    ] {
        assert!(
            matches!(VerifiedKernelModule::verify(&definitions, KernelModuleVerificationBudget {max_steps, max_metadata_bytes, ..exact}).expect_err("One less global proof").cause.kind, KernelErrorKind::BudgetExceeded {requested, maximum, ..} if requested > maximum)
        );
    }
}

#[test]
fn void_field_stores_preserve_stack_proofs_and_per_function_name_namespaces() {
    let caller = [
        integer(7),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::StoreField(0),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Call {
            function: 1,
            arguments: 1,
        },
        KernelInstruction::Discard,
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadField(0),
        KernelInstruction::Return,
    ];
    let worker = [
        integer(9),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::StoreField(0),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadField(0),
        KernelInstruction::Return,
    ];
    let first_names = [b"first".to_vec()];
    let second_names = [b"second".to_vec()];
    let mut first = definition(&caller, 1, 1);
    first.field_names = &first_names;
    let mut second = definition(&worker, 1, 1);
    second.field_names = &second_names;
    let module =
        VerifiedKernelModule::verify(&[first, second], KernelModuleVerificationBudget::default())
            .expect("Independent field namespaces");
    let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
    let root = allocate(&mut objects);
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    assert_eq!(
        module
            .execute_with_heap(
                0,
                &[KernelValue::Object(root)],
                KernelCallBudget::default(),
                &mut context
            )
            .expect("Balanced void stores")
            .value,
        number(7)
    );
    assert_eq!(
        objects
            .get(root, b"second", &strings, HeapWorkBudget::default())
            .expect("Callee table")
            .value,
        Some(number(9))
    );
    for code in [
        vec![
            integer(7),
            KernelInstruction::StoreField(0),
            KernelInstruction::Return,
        ],
        vec![
            integer(7),
            KernelInstruction::LoadLocal(0),
            KernelInstruction::StoreField(0),
            KernelInstruction::Return,
        ],
    ] {
        let mut bad = definition(&code, 1, 1);
        bad.field_names = &first_names;
        assert!(
            VerifiedKernelProgram::verify_definition(bad, KernelVerificationBudget::default())
                .is_err()
        );
    }
}
