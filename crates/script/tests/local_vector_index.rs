// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source reads evaluate index before object and retain immutable vector values.
#![allow(
    clippy::expect_used,
    reason = "Tests supply finite valid primitive source and selected failures."
)]

use crate::function_compile_support;
use cod4x_script::compilation::{FunctionCompilationBudget, FunctionCompileErrorKind};
use cod4x_script::execution::{KernelErrorKind, KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::{LegacyNumber, LegacyNumberError, LegacyVector, LegacyVectorError};
use cod4x_script::source::{SourceBudget, SourceText};

#[test]
fn indexed_source_reads_support_loops_reconstruction_and_caller_isolation() {
    let compiled = function_compile_support::compile_primitive(
        b"main(a){v=(a,2,3);sum=0;for(i=0;i<3;i++){sum+=v[i];}return sum;}",
        FunctionCompilationBudget::default(),
    )
    .expect("Indexed loop source");
    assert_eq!(
        compiled
            .program()
            .execute_with_arguments(
                &[KernelValue::Number(LegacyNumber::Integer(4))],
                KernelExecutionBudget::default()
            )
            .expect("Indexed sum")
            .value,
        KernelValue::Number(LegacyNumber::Float(9.0))
    );
    let compiled = function_compile_support::compile_primitive(
        b"main(a){v=(a,2,3);w=v;v=(v[0],v[1],v[2]+5);return v-w;}",
        FunctionCompilationBudget::default(),
    )
    .expect("Owned reconstruction");
    assert_eq!(
        compiled
            .program()
            .execute_with_arguments(
                &[KernelValue::Number(LegacyNumber::Integer(4))],
                KernelExecutionBudget::default()
            )
            .expect("Immutable input vector alias")
            .value,
        KernelValue::Vector(LegacyVector::from_components([0.0, 0.0, 5.0]))
    );
    let compiled = function_compile_support::compile_primitive(
        b"main(){return(1,2,3)[true];}",
        FunctionCompilationBudget::default(),
    )
    .expect("Boolean retains legacy integer type");
    assert_eq!(
        compiled
            .program()
            .execute(KernelExecutionBudget::default())
            .expect("Direct constructor index")
            .value,
        KernelValue::Number(LegacyNumber::Float(2.0))
    );
}

#[test]
fn index_expression_failure_precedes_object_evaluation_and_retains_original_spans() {
    let text = b"PrivateMain(a){return(1/a,2/a,3/a)[1/a];}";
    let compiled =
        function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
            .expect("Dynamic index/object source");
    let error = compiled
        .program()
        .execute_with_arguments(
            &[KernelValue::Number(LegacyNumber::Integer(0))],
            KernelExecutionBudget::default(),
        )
        .expect_err("Index fails first");
    assert_eq!(
        error.kind,
        KernelErrorKind::Numeric(LegacyNumberError::DivisionByZero)
    );
    let offset = u64::try_from(
        text.iter()
            .rposition(|byte| *byte == b'/')
            .expect("Index operator"),
    )
    .expect("Offset");
    assert_eq!(
        compiled.instruction_span(error.instruction.expect("PC")),
        Some(offset..offset + 1)
    );
    for (text, kind) in [
        (
            "PrivateMain(){return(1,2,3)[3];}",
            KernelErrorKind::Vector(LegacyVectorError::IndexOutOfRange),
        ),
        (
            "PrivateMain(){return(1,2,3)[1.0];}",
            KernelErrorKind::Vector(LegacyVectorError::ExpectedIntegerIndex),
        ),
        (
            "PrivateMain(){return(1,2,3)[undefined];}",
            KernelErrorKind::Vector(LegacyVectorError::ExpectedIntegerIndex),
        ),
        (
            "PrivateMain(){return 1[0];}",
            KernelErrorKind::ExpectedIndexable,
        ),
    ] {
        let compiled = function_compile_support::compile_primitive(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect("Runtime index validation");
        let error = compiled
            .program()
            .execute(KernelExecutionBudget::default())
            .expect_err("Index failure");
        assert_eq!(error.kind, kind);
        let source = SourceText::index(text.as_bytes(), SourceBudget::default()).expect("Source");
        assert_eq!(
            source
                .span(
                    compiled
                        .instruction_span(error.instruction.expect("PC"))
                        .expect("Map")
                )
                .expect("Bracket"),
            b"["
        );
        assert!(!format!("{compiled:?} {error:?}").contains("Private"));
    }
}

#[test]
fn index_name_order_write_rejection_and_pure_profile_remain_explicit() {
    let text = b"main(){return PrivateObject[PrivateIndex];}";
    let error =
        function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
            .expect_err("Name ordering");
    assert_eq!(error.kind, FunctionCompileErrorKind::UninitializedLocal);
    let source = SourceText::index(text, SourceBudget::default()).expect("Source");
    assert_eq!(
        source.span(error.span).expect("First missing name"),
        b"PrivateIndex"
    );
    assert_eq!(
        function_compile_support::compile_primitive(
            b"main(){v=(1,2,3);v[0]=4;}",
            FunctionCompilationBudget::default()
        )
        .expect_err("Legacy vector writes stay unsupported")
        .kind,
        FunctionCompileErrorKind::UnsupportedLocalTarget
    );
    assert_eq!(
        function_compile_support::compile_scalar(
            b"main(){return(1,2,3)[0];}",
            FunctionCompilationBudget::default()
        )
        .expect_err("Pure scalar syntax stays restricted")
        .kind,
        FunctionCompileErrorKind::UnsupportedPrimary
    );
}
