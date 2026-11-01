// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests deliberately reject unsupported object execution."
)]
use crate::kernel_call_support::{self, definition};
use crate::object_heap_support::{self, allocate};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout,
    KernelHeapContext, KernelInstruction, KernelValue, KernelVerificationBudget,
    VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};

#[test]
fn object_constants_and_pure_or_string_only_invocations_reject_before_frame_or_collection_setup() {
    let (mut strings, mut objects) = object_heap_support::stores(ObjectHeapBudget::default());
    let value = KernelValue::Object(allocate(&mut objects));
    let code = [
        KernelInstruction::Constant(value),
        KernelInstruction::Return,
    ];
    assert_eq!(
        VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
            .expect_err("No mutable object constants")
            .kind,
        KernelErrorKind::RuntimeObjectConstant
    );
    let unreachable = [
        KernelInstruction::Jump(2),
        code[0],
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::Return,
    ];
    assert_eq!(
        VerifiedKernelProgram::verify(&unreachable, KernelVerificationBudget::default())
            .expect_err("Unreachable constants also reject")
            .kind,
        KernelErrorKind::RuntimeObjectConstant
    );
    let code = [KernelInstruction::LoadLocal(0), KernelInstruction::Return];
    let program = VerifiedKernelProgram::verify_function(
        &code,
        KernelFunctionLayout::new(1, 1).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Argument proof");
    let module = kernel_call_support::verify(&[definition(&code, 1, 1)]);
    assert_eq!(
        program
            .execute_with_arguments(&[value], KernelExecutionBudget::default())
            .expect_err("No object lifetime context")
            .kind,
        KernelErrorKind::ObjectExecutionUnsupported
    );
    assert_eq!(
        module
            .execute(0, &[value], KernelCallBudget::default())
            .expect_err("No object call lifetime context")
            .cause
            .kind,
        KernelErrorKind::ObjectExecutionUnsupported
    );
    assert_eq!(
        program
            .execute_legacy_script_arguments(&[value], KernelExecutionBudget::default())
            .expect_err("Legacy argument adaptation cannot ignore objects")
            .kind,
        KernelErrorKind::ObjectExecutionUnsupported
    );
    let before = strings.usage();
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default());
    assert_eq!(
        program
            .execute_with_heap(&[value], KernelExecutionBudget::default(), &mut context)
            .expect_err("String context cannot ignore objects")
            .kind,
        KernelErrorKind::ObjectExecutionUnsupported
    );
    assert_eq!(
        module
            .execute_with_heap(0, &[value], KernelCallBudget::default(), &mut context)
            .expect_err("String call context cannot ignore objects")
            .cause
            .kind,
        KernelErrorKind::ObjectExecutionUnsupported
    );
    assert_eq!(context.heap_usage(), before);
    assert_eq!(context.usage().steps, 0);
}
