// SPDX-License-Identifier: AGPL-3.0-or-later
//! Wide vector bindings and deep source lowering retain configurable flat storage.
#![allow(
    clippy::expect_used,
    reason = "Tests generate finite source within explicit frontend/compiler limits."
)]

use crate::function_compile_support;
use cod4x_script::compilation::{
    FunctionCompilationBudget, FunctionCompilationResource, FunctionCompileErrorKind,
};
use cod4x_script::execution::{KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::{LegacyNumber, LegacyVector};
use cod4x_script::syntax::LegacySyntaxBudget;
use std::fmt::Write;

#[test]
fn seventy_thousand_vector_source_locals_use_wide_slots_and_complete_inline_values() {
    let mut source = String::from("main(a){");
    for index in 0..70_000 {
        write!(source, "v{index}=(a,2,3);").expect("Generated vector source");
    }
    source.push_str("return v69999;}");
    let compiled = function_compile_support::compile_primitive_with_syntax_budget(
        source.as_bytes(),
        FunctionCompilationBudget::default(),
        LegacySyntaxBudget {
            max_nodes: 3_000_000,
            max_child_edges: 4_000_000,
            max_metadata_bytes: 512 * 1024 * 1024,
            ..LegacySyntaxBudget::default()
        },
    )
    .expect("Separately admitted wide frontend");
    assert_eq!(compiled.usage().local_bindings, 70_001);
    let args = [KernelValue::Number(LegacyNumber::Integer(3))];
    let result = compiled
        .program()
        .execute_with_arguments(&args, KernelExecutionBudget::default())
        .expect("Wide vector source execution");
    assert_eq!(
        result.value,
        KernelValue::Vector(LegacyVector::from_components([3.0, 2.0, 3.0]))
    );
    assert_eq!(result.local_values, 70_001);
    assert_eq!(
        result.local_bytes,
        70_001 * size_of::<Option<KernelValue>>() as u64
    );
}

#[test]
fn twelve_thousand_vector_additions_lower_execute_and_fail_drop_without_recursion() {
    let source = format!("main(a){{return {}(a,2,3);}}", "(a,2,3)+".repeat(12_000));
    let compiled = function_compile_support::compile_primitive(
        source.as_bytes(),
        FunctionCompilationBudget::default(),
    )
    .expect("Deep vector expressions");
    let result = compiled
        .program()
        .execute_with_arguments(
            &[KernelValue::Number(LegacyNumber::Integer(1))],
            KernelExecutionBudget::default(),
        )
        .expect("Deep vector execution");
    assert_eq!(
        result.value,
        KernelValue::Vector(LegacyVector::from_components([12001.0, 24002.0, 36003.0]))
    );
    let error = function_compile_support::compile_primitive(
        source.as_bytes(),
        FunctionCompilationBudget {
            max_task_frames: compiled.usage().peak_task_frames - 1,
            ..FunctionCompilationBudget::default()
        },
    )
    .expect_err("Deep pending-task failure drops flat");
    assert!(matches!(
        error.kind,
        FunctionCompileErrorKind::BudgetExceeded {
            resource: FunctionCompilationResource::TaskFrames,
            ..
        }
    ));
    assert!(error.work_steps > 0);
}
