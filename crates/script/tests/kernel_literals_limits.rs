// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests measure complete literal proof/cache/root overlap and one-less admission."
)]
use crate::kernel_call_support::definition;
use crate::kernel_heap_support::{self, allocate};
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelModuleVerificationBudget, KernelResource, KernelVerificationBudget,
    VerifiedKernelModule, VerifiedKernelProgram,
};
use cod4x_script::heap::HeapWorkBudget;
use cod4x_script::legacy::LegacyArithmetic;

#[test]
fn complete_module_literal_overlap_has_exact_global_proof_metadata_and_work_limits()
-> Result<(), String> {
    let code = [
        KernelInstruction::StringLiteral(0),
        KernelInstruction::Return,
    ];
    let first = vec![vec![1; 70], vec![2; 40]];
    let second = vec![vec![3; 90]];
    let mut definitions = [definition(&code, 0, 0), definition(&code, 0, 0)];
    definitions[0].string_literals = &first;
    definitions[1].string_literals = &second;
    let usage =
        VerifiedKernelModule::verify(&definitions, KernelModuleVerificationBudget::default())
            .expect("Measured module")
            .usage();
    assert_eq!(usage.string_literals, 3);
    assert_eq!(
        usage.retained_string_bytes,
        3 * size_of::<Vec<u8>>() as u64 + 200
    );
    let exact = KernelModuleVerificationBudget {
        max_steps: usage.verification_steps,
        max_metadata_bytes: usage.verification_metadata_bytes,
        ..KernelModuleVerificationBudget::default()
    };
    assert_eq!(
        VerifiedKernelModule::verify(&definitions, exact)
            .expect("Exact complete proof")
            .usage(),
        usage
    );
    for budget in [
        KernelModuleVerificationBudget {
            max_steps: exact.max_steps - 1,
            ..exact
        },
        KernelModuleVerificationBudget {
            max_metadata_bytes: exact.max_metadata_bytes - 1,
            ..exact
        },
    ] {
        let error = VerifiedKernelModule::verify(&definitions, budget)
            .expect_err("One less module proof resource");
        match error.cause.kind {
            KernelErrorKind::BudgetExceeded {
                resource: KernelResource::VerificationSteps,
                requested,
                maximum,
            } => {
                assert_eq!(maximum, budget.max_steps);
                assert!(requested > maximum);
            }
            KernelErrorKind::BudgetExceeded {
                resource: KernelResource::VerificationMetadataBytes,
                requested,
                maximum,
            } => {
                assert_eq!(maximum, budget.max_metadata_bytes);
                assert!(requested > maximum);
            }
            _ => return Err("Unexpected module literal proof failure".into()),
        }
        assert!(error.cause.work_steps <= budget.max_steps);
    }
    Ok(())
}

#[test]
fn literal_cache_initialization_and_payload_copy_share_one_exact_work_budget() -> Result<(), String>
{
    let literals = vec![b"abcdefgh".to_vec()];
    let code = [
        KernelInstruction::StringLiteral(0),
        KernelInstruction::Return,
    ];
    let program = VerifiedKernelProgram::verify_with_literals(
        &code,
        KernelFunctionLayout::default(),
        &literals,
        KernelVerificationBudget::default(),
    )
    .expect("Literal proof");
    let mut exact = None;
    for maximum in 0..=12 {
        let mut heap = kernel_heap_support::heap(1);
        let mut context = KernelHeapContext::new(
            &mut heap,
            &[],
            HeapWorkBudget {
                max_steps: maximum,
                ..HeapWorkBudget::default()
            },
        );
        let result = program.execute_with_heap(&[], KernelExecutionBudget::default(), &mut context);
        if maximum < 12 {
            let error = result.expect_err("Every smaller total work budget");
            match error.kind {
                KernelErrorKind::BudgetExceeded {
                    resource: KernelResource::HeapWorkSteps,
                    requested,
                    maximum: limit,
                } => {
                    assert_eq!(limit, maximum);
                    assert!(requested > limit);
                }
                _ => return Err("Unexpected literal work failure".into()),
            }
            assert_eq!(context.heap_usage().live_strings, 0);
        } else {
            result.expect("Exact copy and initialization work");
            assert_eq!(context.usage().steps, 12);
            exact = Some(context.usage().peak_scratch_bytes);
        }
    }
    let scratch = exact.ok_or("Missing cache observation")?;
    for maximum in [scratch - 1, scratch] {
        let mut heap = kernel_heap_support::heap(1);
        let mut context = KernelHeapContext::new(
            &mut heap,
            &[],
            HeapWorkBudget {
                max_scratch_bytes: maximum,
                ..HeapWorkBudget::default()
            },
        );
        let result = program.execute_with_heap(&[], KernelExecutionBudget::default(), &mut context);
        if maximum == scratch {
            result.expect("Exact cache metadata");
        } else {
            assert!(matches!(
                result.expect_err("One less cache byte").kind,
                KernelErrorKind::BudgetExceeded {
                    resource: KernelResource::HeapScratchBytes,
                    ..
                }
            ));
            assert_eq!(context.heap_usage().live_strings, 0);
            assert_eq!(context.usage().steps, 0);
        }
    }
    Ok(())
}

#[test]
fn simultaneous_cache_root_ids_and_mark_flags_share_scratch_admission() -> Result<(), String> {
    let literals = vec![b"cached".to_vec()];
    let code = [
        KernelInstruction::StringLiteral(0),
        KernelInstruction::Discard,
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        KernelInstruction::Return,
    ];
    let program = VerifiedKernelProgram::verify_with_literals(
        &code,
        KernelFunctionLayout::new(1, 1).expect("Layout"),
        &literals,
        KernelVerificationBudget::default(),
    )
    .expect("Cache/root program");
    let mut exact = None;
    for pass in 0..3 {
        let mut heap = kernel_heap_support::heap(3);
        let argument = allocate(&mut heap, b"x");
        let garbage = allocate(&mut heap, b"old");
        let mut budget = HeapWorkBudget::default();
        if let Some(bytes) = exact {
            budget.max_scratch_bytes = if pass == 2 { bytes - 1 } else { bytes };
        }
        let mut context = KernelHeapContext::new(&mut heap, &[], budget);
        let result = program.execute_with_heap(
            &[cod4x_script::execution::KernelValue::String(argument)],
            KernelExecutionBudget::default(),
            &mut context,
        );
        if pass == 2 {
            let error = result.expect_err("One less combined scratch byte");
            assert_eq!(error.instruction, Some(4));
            assert_eq!(
                context.get(garbage).expect("Failed collection is atomic"),
                b"old"
            );
            assert_eq!(context.usage().collections, 0);
            match error.kind {
                KernelErrorKind::BudgetExceeded {
                    resource: KernelResource::HeapScratchBytes,
                    requested,
                    maximum,
                } => {
                    assert_eq!(maximum, budget.max_scratch_bytes);
                    assert_eq!(requested, maximum + 1);
                }
                _ => return Err("Unexpected scratch failure".into()),
            }
        } else {
            result.expect("Full cache/root/mark overlap admitted");
            assert_eq!(context.usage().collections, 1);
            exact = Some(context.usage().peak_scratch_bytes);
        }
    }
    Ok(())
}
