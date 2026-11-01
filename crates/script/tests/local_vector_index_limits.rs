// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source index tasks retain exact admission and flat deep failure cleanup.
#![allow(
    clippy::expect_used,
    reason = "Tests generate finite source within explicit budgets."
)]

use crate::function_compile_support;
use cod4x_script::compilation::{
    FunctionCompilationBudget, FunctionCompilationResource, FunctionCompileErrorKind,
};
use cod4x_script::execution::{KernelErrorKind, KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::{LegacyNumber, LegacyVector};

#[test]
fn source_index_compilation_accepts_exact_limits_and_rejects_one_less() {
    let text = b"main(v,i){return v[i];}";
    let usage =
        function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
            .expect("Index source accounting")
            .usage();
    let exact = FunctionCompilationBudget {
        max_steps: usage.compiler_steps,
        max_instructions: usage.instructions,
        max_task_frames: usage.peak_task_frames,
        max_metadata_bytes: usage.peak_metadata_bytes,
        max_local_bindings: usage.local_bindings,
        max_identifier_bytes: usage.identifier_bytes,
        max_control_bytes: 0,
        ..FunctionCompilationBudget::default()
    };
    assert_eq!(
        function_compile_support::compile_primitive(text, exact)
            .expect("Exact source limits")
            .usage(),
        usage
    );
    for resource in [
        FunctionCompilationResource::Steps,
        FunctionCompilationResource::Instructions,
        FunctionCompilationResource::TaskFrames,
        FunctionCompilationResource::MetadataBytes,
        FunctionCompilationResource::LocalBindings,
        FunctionCompilationResource::IdentifierBytes,
    ] {
        let mut budget = exact;
        match resource {
            FunctionCompilationResource::Steps => budget.max_steps -= 1,
            FunctionCompilationResource::Instructions => budget.max_instructions -= 1,
            FunctionCompilationResource::TaskFrames => budget.max_task_frames -= 1,
            FunctionCompilationResource::MetadataBytes => budget.max_metadata_bytes -= 1,
            FunctionCompilationResource::LocalBindings => budget.max_local_bindings -= 1,
            FunctionCompilationResource::IdentifierBytes => budget.max_identifier_bytes -= 1,
            FunctionCompilationResource::ControlBytes => budget.max_control_bytes = 0,
        }
        let error = function_compile_support::compile_primitive(text, budget)
            .expect_err("One less source resource");
        assert!(
            matches!(error.kind,FunctionCompileErrorKind::BudgetExceeded { resource:actual,requested,maximum } if actual==resource && requested>maximum)
        );
        assert!(error.work_steps > 0 && error.work_steps <= usage.compiler_steps);
        assert!(error.peak_metadata_bytes <= budget.max_metadata_bytes);
    }
}

#[test]
fn twelve_thousand_index_reads_verify_and_fail_with_flat_owned_tasks() {
    let source = format!("main(v){{return v{};}}", "[0]".repeat(12_000));
    let compiled = function_compile_support::compile_primitive(
        source.as_bytes(),
        FunctionCompilationBudget::default(),
    )
    .expect("Deep reference reads");
    assert_eq!(compiled.program().usage().max_stack_values, 12_001);
    let args = [KernelValue::Vector(LegacyVector::from_components([
        1.0, 2.0, 3.0,
    ]))];
    let error = compiled
        .program()
        .execute_with_arguments(&args, KernelExecutionBudget::default())
        .expect_err("The first coordinate is a float, not another indexable vector");
    assert_eq!(error.kind, KernelErrorKind::ExpectedIndexable);
    assert_eq!(error.peak_stack_values, 12_001);
    let second = u64::try_from(
        source
            .bytes()
            .enumerate()
            .filter(|(_, byte)| *byte == b'[')
            .nth(1)
            .expect("Second bracket")
            .0,
    )
    .expect("Offset");
    assert_eq!(
        compiled.instruction_span(error.instruction.expect("PC")),
        Some(second..second + 1)
    );
    let error = function_compile_support::compile_primitive(
        source.as_bytes(),
        FunctionCompilationBudget {
            max_task_frames: compiled.usage().peak_task_frames - 1,
            ..FunctionCompilationBudget::default()
        },
    )
    .expect_err("Deep pending tasks fail/drop flat");
    assert!(matches!(
        error.kind,
        FunctionCompileErrorKind::BudgetExceeded {
            resource: FunctionCompilationResource::TaskFrames,
            ..
        }
    ));
    assert_eq!(
        args[0],
        KernelValue::Vector(LegacyVector::from_components([1.0, 2.0, 3.0]))
    );
    let scalar_args = [KernelValue::Number(LegacyNumber::Integer(1))];
    assert_eq!(
        compiled
            .program()
            .execute_with_arguments(&scalar_args, KernelExecutionBudget::default())
            .expect_err("Unsupported object stays a type error")
            .kind,
        KernelErrorKind::ExpectedIndexable
    );
}
