// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exercise cache ownership after bounded fuel exhaustion."
)]
use crate::kernel_heap_support;
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelResource, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::HeapWorkBudget;

#[test]
fn fuel_failure_preserves_bounded_storage_and_next_invocation_reclaims_previous_cache() {
    let code = [
        KernelInstruction::StringLiteral(0),
        KernelInstruction::Discard,
        KernelInstruction::Jump(0),
    ];
    let literals = vec![b"private".to_vec()];
    let program = VerifiedKernelProgram::verify_with_literals(
        &code,
        KernelFunctionLayout::default(),
        &literals,
        KernelVerificationBudget::default(),
    )
    .expect("Fuel-bounded loop");
    let mut heap = kernel_heap_support::heap(1);
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    for (invocation, steps) in [(0, 32), (1, 3)] {
        let error = program
            .execute_with_heap(
                &[],
                KernelExecutionBudget {
                    max_steps: steps,
                    ..KernelExecutionBudget::default()
                },
                &mut context,
            )
            .expect_err("No fabricated completion");
        assert!(matches!(
            error.kind,
            KernelErrorKind::BudgetExceeded {
                resource: KernelResource::ExecutionSteps,
                ..
            }
        ));
        assert_eq!(error.work_steps, steps);
        assert_eq!(context.heap_usage().live_strings, 1);
        assert_eq!(context.heap_usage().payload_bytes, 7);
        assert_eq!(context.usage().collections, invocation);
        assert!(!format!("{error:?} {context:?} {program:?}").contains("private"));
    }
}
