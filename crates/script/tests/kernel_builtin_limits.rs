// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests measure finite pure builtin proof/execution admissions."
)]

use cod4x_script::execution::{
    KernelBuiltin, KernelErrorKind, KernelExecutionBudget, KernelInstruction, KernelResource,
    KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::{LegacyNumber, LegacyVector};

fn code() -> [KernelInstruction; 3] {
    [
        KernelInstruction::Constant(KernelValue::Vector(LegacyVector::from_components([
            f32::NAN,
            -0.0,
            f32::INFINITY,
        ]))),
        KernelInstruction::Builtin {
            function: KernelBuiltin::IsVector,
            arguments: 1,
        },
        KernelInstruction::Return,
    ]
}

#[test]
fn pure_builtins_share_exact_proof_and_execution_count_byte_and_work_limits() -> Result<(), String>
{
    let code = code();
    let proof = VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
        .expect("Predicate proof");
    let usage = proof.usage();
    let exact = KernelVerificationBudget {
        max_instructions: usage.instructions,
        max_steps: usage.verification_steps,
        max_stack_values: usage.max_stack_values,
        max_local_values: 0,
        max_metadata_bytes: usage.verification_metadata_bytes,
    };
    VerifiedKernelProgram::verify(&code, exact).expect("Exact proof admission");
    for resource in [
        KernelResource::Instructions,
        KernelResource::VerificationSteps,
        KernelResource::StackValues,
        KernelResource::VerificationMetadataBytes,
    ] {
        let mut budget = exact;
        match resource {
            KernelResource::Instructions => budget.max_instructions -= 1,
            KernelResource::VerificationSteps => budget.max_steps -= 1,
            KernelResource::StackValues => budget.max_stack_values -= 1,
            KernelResource::VerificationMetadataBytes => budget.max_metadata_bytes -= 1,
            _ => return Err("Unexpected proof resource".into()),
        }
        let error =
            VerifiedKernelProgram::verify(&code, budget).expect_err("One less proof resource");
        assert!(
            matches!(error.kind, KernelErrorKind::BudgetExceeded { resource: actual, requested, maximum } if actual == resource && requested > maximum)
        );
    }
    let exact = KernelExecutionBudget {
        max_steps: 3,
        max_stack_values: 1,
        max_stack_bytes: size_of::<KernelValue>() as u64,
        max_local_values: 0,
        max_local_bytes: 0,
    };
    let result = proof.execute(exact).expect("Exact pure execution");
    assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(1)));
    for resource in [
        KernelResource::ExecutionSteps,
        KernelResource::StackValues,
        KernelResource::ExecutionStackBytes,
    ] {
        let mut budget = exact;
        match resource {
            KernelResource::ExecutionSteps => budget.max_steps -= 1,
            KernelResource::StackValues => budget.max_stack_values -= 1,
            KernelResource::ExecutionStackBytes => budget.max_stack_bytes -= 1,
            _ => return Err("Unexpected execution resource".into()),
        }
        let error = proof
            .execute(budget)
            .expect_err("One less execution resource");
        assert!(
            matches!(error.kind, KernelErrorKind::BudgetExceeded { resource: actual, requested, maximum } if actual == resource && requested > maximum)
        );
        assert!(error.work_steps <= budget.max_steps);
    }
    assert_eq!(
        proof.execute(exact).expect("Fresh recovery after failures"),
        result
    );
    Ok(())
}

#[test]
fn huge_declared_builtin_counts_require_reachable_operands_without_static_arity_rejection() {
    let code = [
        KernelInstruction::Builtin {
            function: KernelBuiltin::IsInt,
            arguments: usize::MAX,
        },
        KernelInstruction::Return,
    ];
    assert_eq!(
        VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
            .expect_err("Reachable count needs real operands")
            .kind,
        KernelErrorKind::StackUnderflow {
            required: usize::MAX as u64,
            available: 0
        }
    );
    let code = [
        KernelInstruction::Jump(2),
        code[0],
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::Return,
    ];
    assert_eq!(
        VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
            .expect("Unselected huge wrong count")
            .execute(KernelExecutionBudget::default())
            .expect("Skipped builtin has no effect")
            .value,
        KernelValue::Undefined
    );
}
