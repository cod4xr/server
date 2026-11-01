// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Validation markers must fail both outside discarded regions and at final code publication."
)]
use super::emit_validation;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_task::PendingFunctionKind;
use crate::compilation::{
    FunctionCompilationBudget, FunctionCompileErrorKind, FunctionSourceProfile,
};

#[test]
fn normal_lowering_cannot_emit_validation_markers() {
    let mut builder = FunctionBuilder::new(
        FunctionCompilationBudget::default(),
        FunctionSourceProfile::PrimitiveStatements,
    );
    let error = emit_validation(&mut builder, 1..2).expect_err("No active discarded region");
    assert_eq!(error.kind, FunctionCompileErrorKind::InvalidControlState);
    assert!(builder.instructions.is_empty());
}

#[test]
fn an_escaped_validation_marker_cannot_become_verified_executable_code() {
    let mut builder = FunctionBuilder::new(
        FunctionCompilationBudget::default(),
        FunctionSourceProfile::PrimitiveStatements,
    );
    builder
        .append(PendingFunctionKind::ValidationOnly, 1..2)
        .expect("Private injected marker");
    let error = builder
        .resolve(0..3)
        .expect_err("No executable representation");
    assert_eq!(error.kind, FunctionCompileErrorKind::InvalidControlState);
    assert_eq!(error.span, 1..2);
}
