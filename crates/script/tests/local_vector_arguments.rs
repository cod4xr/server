// SPDX-License-Identifier: AGPL-3.0-or-later
//! Restricted source syntax can receive inline vector arguments through its owned kernel.
#![allow(
    clippy::expect_used,
    reason = "Tests invoke finite source with typed primitive arguments."
)]

use crate::scalar_compile_support;
use cod4x_script::compilation::{ScalarCompilationBudget, ScalarCompileErrorKind};
use cod4x_script::execution::{KernelErrorKind, KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::{LegacyNumber, LegacyVector, LegacyVectorError};
use cod4x_script::source::{SourceBudget, SourceText};

#[test]
fn parameterized_source_reuses_vector_operations_without_source_constructor_claims() {
    let source = b"PrivateMain(vec,scale){x=vec;x*=scale;return x;}";
    let compiled =
        scalar_compile_support::compile_local(source, ScalarCompilationBudget::default())
            .expect("Restricted named source");
    let args = [
        KernelValue::Vector(LegacyVector::from_components([1.0, 2.0, 3.0])),
        KernelValue::Number(LegacyNumber::Integer(2)),
    ];
    let expected = KernelValue::Vector(LegacyVector::from_components([2.0, 4.0, 6.0]));
    for _ in 0..2 {
        assert_eq!(
            compiled
                .program()
                .execute_with_arguments(&args, KernelExecutionBudget::default())
                .expect("Vector parameter operation")
                .value,
            expected
        );
    }
    assert_eq!(
        args[0],
        KernelValue::Vector(LegacyVector::from_components([1.0, 2.0, 3.0]))
    );
    assert_eq!(compiled.usage().validation_steps, None);
    for pure in [false, true] {
        let error = if pure {
            scalar_compile_support::compile(
                b"main(){return(1,2,3);}",
                ScalarCompilationBudget::default(),
            )
        } else {
            scalar_compile_support::compile_local(
                b"main(){return(1,2,3);}",
                ScalarCompilationBudget::default(),
            )
        }
        .expect_err("Source vector construction remains separate work");
        assert_eq!(error.kind, ScalarCompileErrorKind::UnsupportedGroupArity);
    }
}

#[test]
fn vector_argument_failures_keep_source_operator_locations_and_redaction() {
    for (text, expected, token) in [
        (
            "PrivateMain(vec){vec+=1;return vec;}",
            KernelErrorKind::Vector(LegacyVectorError::UnsupportedMixedAddition),
            "+=",
        ),
        (
            "PrivateMain(vec){return vec<1;}",
            KernelErrorKind::Vector(LegacyVectorError::UnsupportedOrdering),
            "<",
        ),
        (
            "PrivateMain(vec){if(vec)return 1;return 0;}",
            KernelErrorKind::ExpectedNumber,
            "if",
        ),
    ] {
        let compiled = scalar_compile_support::compile_local(
            text.as_bytes(),
            ScalarCompilationBudget::default(),
        )
        .expect("Argument-dependent source");
        let args = [KernelValue::Vector(LegacyVector::from_components([
            1.0, 2.0, 3.0,
        ]))];
        let error = compiled
            .program()
            .execute_with_arguments(&args, KernelExecutionBudget::default())
            .expect_err("Located vector type failure");
        assert_eq!(error.kind, expected);
        let source = SourceText::index(text.as_bytes(), SourceBudget::default()).expect("Source");
        assert_eq!(
            source
                .span(
                    compiled
                        .instruction_span(error.instruction.expect("PC"))
                        .expect("Map")
                )
                .expect("Operation token"),
            token.as_bytes()
        );
        assert_eq!(
            args[0],
            KernelValue::Vector(LegacyVector::from_components([1.0, 2.0, 3.0]))
        );
        assert!(!format!("{compiled:?} {error:?}").contains("Private"));
    }
}
