// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests verify immutable field tables independently of runtime string caches."
)]
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelFunctionDefinition, KernelFunctionLayout,
    KernelHeapContext, KernelInstruction, KernelValue, KernelVerificationBudget,
    VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};

#[test]
fn owned_binary_field_names_do_not_allocate_runtime_string_cache_or_expose_payloads() {
    let key = b"PrivateField\0\xff";
    let code = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadField(0),
        KernelInstruction::Return,
    ];
    let mut names = vec![key.to_vec()];
    names.extend((0..70_000).map(|_| b"unused".to_vec()));
    let definition = KernelFunctionDefinition {
        instructions: &code,
        layout: KernelFunctionLayout::new(1, 1).expect("Layout"),
        string_literals: &[],
        field_names: &names,
        switch_labels: &[],
    };
    let program =
        VerifiedKernelProgram::verify_definition(definition, KernelVerificationBudget::default())
            .expect("Independent name proof");
    assert_eq!(program.usage().field_names, 70_001);
    assert_eq!(program.usage().string_literals, 0);
    assert!(program.usage().retained_field_bytes > 70_001 * size_of::<Vec<u8>>() as u64);
    names[0].fill(0);
    names.clear();
    let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
    let root = allocate(&mut objects);
    objects
        .put(root, key, number(7), &strings, HeapWorkBudget::default())
        .expect("Owned key");
    let mut context = KernelHeapContext::new(
        &mut strings,
        &[],
        HeapWorkBudget {
            max_scratch_bytes: 0,
            ..HeapWorkBudget::default()
        },
    )
    .with_objects(&mut objects, &[]);
    assert_eq!(
        program
            .execute_with_heap(
                &[KernelValue::Object(root)],
                KernelExecutionBudget::default(),
                &mut context
            )
            .expect("No cache slots for field names")
            .value,
        number(7)
    );
    assert_eq!(context.heap_usage().live_strings, 0);
    assert_eq!(context.usage().peak_scratch_bytes, 0);
    assert!(!format!("{program:?} {context:?}").contains("PrivateField"));
}

#[test]
fn every_field_operand_checks_its_own_table_even_in_unreachable_code() {
    for instruction in [
        KernelInstruction::LoadField(0),
        KernelInstruction::StoreField(0),
    ] {
        let code = [
            KernelInstruction::Jump(2),
            instruction,
            KernelInstruction::Constant(KernelValue::Undefined),
            KernelInstruction::Return,
        ];
        assert_eq!(
            VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
                .expect_err("All name operands")
                .kind,
            KernelErrorKind::InvalidFieldName { name: 0 }
        );
    }
}
