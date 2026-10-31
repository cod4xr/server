// SPDX-License-Identifier: AGPL-3.0-or-later
//! Plain-name compound source executes shared numeric operators with initialized targets.
#![allow(
    clippy::expect_used,
    reason = "Tests supply valid finite compound-assignment source."
)]

use cod4x_script::compilation::{
    ScalarCompilationBudget, ScalarCompilationResource, ScalarCompileErrorKind,
};
use cod4x_script::execution::{KernelErrorKind, KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::{LegacyNumber, LegacyNumberError};
use cod4x_script::source::{SourceBudget, SourceText};

use crate::scalar_compile_support;

#[test]
fn all_ten_compound_operators_preserve_numeric_domains_boundaries_and_alias_values() {
    for (expression, expected) in [
        ("x=8;x|=3", LegacyNumber::Integer(11)),
        ("x=8;x^=3", LegacyNumber::Integer(11)),
        ("x=8;x&=3", LegacyNumber::Integer(0)),
        ("x=3;x<<=2", LegacyNumber::Integer(12)),
        ("x=-12;x>>=2", LegacyNumber::Integer(-3)),
        ("x=2147483647;x+=1", LegacyNumber::Integer(i32::MIN)),
        ("x=-2147483648;x-=1", LegacyNumber::Integer(i32::MAX)),
        ("x=3;x*=4", LegacyNumber::Integer(12)),
        ("x=7;x/=2", LegacyNumber::Float(3.5)),
        ("x=-7;x%=3", LegacyNumber::Integer(-1)),
        ("x=1.5;x+=2", LegacyNumber::Float(3.5)),
        ("x=2;x+=x+1", LegacyNumber::Integer(5)),
    ] {
        let text = format!("main(){{{expression};return x;}}");
        let compiled = scalar_compile_support::compile_local(
            text.as_bytes(),
            ScalarCompilationBudget::default(),
        )
        .expect("Compound source compilation");
        assert_eq!(compiled.usage().local_bindings, 1);
        assert_eq!(
            compiled
                .program()
                .execute(KernelExecutionBudget::default())
                .expect("Compound source invocation")
                .value,
            KernelValue::Number(expected)
        );
    }
    let compiled = scalar_compile_support::compile_local(
        b"main(arg){arg+=2;return arg;}",
        ScalarCompilationBudget::default(),
    )
    .expect("Compound parameter assignment");
    let arguments = [KernelValue::Number(LegacyNumber::Integer(3))];
    assert_eq!(
        compiled
            .program()
            .execute_with_arguments(&arguments, KernelExecutionBudget::default())
            .expect("Private parameter mutation")
            .value,
        KernelValue::Number(LegacyNumber::Integer(5))
    );
    assert_eq!(arguments[0], KernelValue::Number(LegacyNumber::Integer(3)));
}

#[test]
fn compound_targets_require_initialization_and_numeric_failures_map_to_original_compound_tokens() {
    for (text, expected) in [
        (
            "main(){PrivateMissing+=1;}",
            ScalarCompileErrorKind::UninitializedLocal,
        ),
        (
            "main(){self.PrivateField+=1;}",
            ScalarCompileErrorKind::UnsupportedLocalTarget,
        ),
        (
            "main(){PrivateArray[0]+=1;}",
            ScalarCompileErrorKind::UnsupportedLocalTarget,
        ),
    ] {
        let error = scalar_compile_support::compile_local(
            text.as_bytes(),
            ScalarCompilationBudget::default(),
        )
        .expect_err("Unsupported/uninitialized target");
        assert_eq!(error.kind, expected);
        assert!(!format!("{error:?} {error}").contains("Private"));
    }
    for (expression, expected, token) in [
        (
            "x=1.0;x|=1",
            KernelErrorKind::Numeric(LegacyNumberError::ExpectedInteger),
            "|=",
        ),
        ("x=undefined;x+=1", KernelErrorKind::ExpectedNumber, "+="),
        (
            "x=1;x/=0",
            KernelErrorKind::Numeric(LegacyNumberError::DivisionByZero),
            "/=",
        ),
        (
            "x=-2147483648;x%=-1",
            KernelErrorKind::Numeric(LegacyNumberError::RemainderOverflow),
            "%=",
        ),
    ] {
        let text = format!("main(){{{expression};return x;}}");
        let compiled = scalar_compile_support::compile_local(
            text.as_bytes(),
            ScalarCompilationBudget::default(),
        )
        .expect("Located compound function");
        let error = compiled
            .program()
            .execute(KernelExecutionBudget::default())
            .expect_err("Runtime numeric error");
        assert_eq!(error.kind, expected);
        let span = compiled
            .instruction_span(error.instruction.expect("Located instruction"))
            .expect("Compound source map");
        let source = SourceText::index(text.as_bytes(), SourceBudget::default()).expect("Source");
        assert_eq!(source.span(span).expect("Original token"), token.as_bytes());
        assert!(error.work_steps > 0);
    }
}

#[test]
fn compound_name_visits_stores_and_instruction_tasks_keep_exact_compiler_admission() {
    let text = b"main(arg){x=arg;x+=x*2;return x;}";
    let usage = scalar_compile_support::compile_local(text, ScalarCompilationBudget::default())
        .expect("Baseline")
        .usage();
    assert_eq!(usage.local_bindings, 2);
    assert_eq!(usage.identifier_bytes, 12);
    let exact = ScalarCompilationBudget {
        max_steps: usage.compiler_steps,
        max_instructions: usage.instructions,
        max_task_frames: usage.peak_task_frames,
        max_metadata_bytes: usage.peak_metadata_bytes,
        max_local_bindings: usage.local_bindings,
        max_identifier_bytes: usage.identifier_bytes,
        ..ScalarCompilationBudget::default()
    };
    assert_eq!(
        scalar_compile_support::compile_local(text, exact)
            .expect("Complete exact admission")
            .usage(),
        usage
    );
    let error = scalar_compile_support::compile_local(
        text,
        ScalarCompilationBudget {
            max_identifier_bytes: 11,
            ..exact
        },
    )
    .expect_err("Final name visit stays charged");
    assert_eq!(
        error.kind,
        ScalarCompileErrorKind::BudgetExceeded {
            resource: ScalarCompilationResource::IdentifierBytes,
            requested: 12,
            maximum: 11
        }
    );
    assert!(error.work_steps > 0 && error.work_steps < usage.compiler_steps);
}
