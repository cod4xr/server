// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit parser limits, fail-fast ordering, malformed-input and privacy checks.
#![allow(
    clippy::expect_used,
    reason = "Tests construct synthetic malformed sources and budgets."
)]

use cod4x_script::lexical::{LegacyLexErrorKind, LegacyLexResource};
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{
    LegacySyntaxBudget, LegacySyntaxErrorKind, LegacySyntaxMode, LegacySyntaxResource,
    LegacySyntaxTree,
};

#[test]
fn each_structural_budget_accepts_exact_usage_and_rejects_one_less() {
    let source = SourceText::index(
        b"main(a) { if(a) return 1; else return 0; }",
        SourceBudget::default(),
    )
    .expect("Source");
    let usage = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )
    .expect("Tree")
    .usage();
    let exact = LegacySyntaxBudget {
        max_nodes: usage.nodes,
        max_child_edges: usage.child_edges,
        max_stack_entries: usage.peak_stack_entries,
        max_steps: usage.steps,
        max_metadata_bytes: usage.peak_metadata_bytes,
        ..LegacySyntaxBudget::default()
    };
    assert!(LegacySyntaxTree::parse_structure(&source, LegacySyntaxMode::Script, exact).is_ok());
    for resource in [
        LegacySyntaxResource::Nodes,
        LegacySyntaxResource::ChildEdges,
        LegacySyntaxResource::StackEntries,
        LegacySyntaxResource::Steps,
        LegacySyntaxResource::MetadataBytes,
    ] {
        let mut budget = exact;
        match resource {
            LegacySyntaxResource::Nodes => budget.max_nodes -= 1,
            LegacySyntaxResource::ChildEdges => budget.max_child_edges -= 1,
            LegacySyntaxResource::StackEntries => budget.max_stack_entries -= 1,
            LegacySyntaxResource::Steps => budget.max_steps -= 1,
            LegacySyntaxResource::MetadataBytes => budget.max_metadata_bytes -= 1,
        }
        let error = LegacySyntaxTree::parse_structure(&source, LegacySyntaxMode::Script, budget)
            .expect_err("One less");
        assert_eq!(error.kind, LegacySyntaxErrorKind::BudgetExceeded(resource));
        assert!(error.offset <= source.bytes().len() as u64);
    }
}

#[test]
fn scanner_budgets_and_exact_failure_offsets_propagate_without_source_disclosure() {
    let source = SourceText::index(
        b"main() { return \"private-secret\"; }",
        SourceBudget::default(),
    )
    .expect("Source");
    let mut budget = LegacySyntaxBudget::default();
    budget.lexical.max_steps = 0;
    let error = LegacySyntaxTree::parse_structure(&source, LegacySyntaxMode::Script, budget)
        .expect_err("Scanner budget");
    assert_eq!(
        error.kind,
        LegacySyntaxErrorKind::Lexical(LegacyLexErrorKind::BudgetExceeded(
            LegacyLexResource::Steps
        ))
    );
    for (bytes, expected, offset) in [
        (
            b"main( { } /* unfinished".as_slice(),
            LegacySyntaxErrorKind::UnexpectedToken,
            6,
        ),
        (
            b"main() {} /* unfinished".as_slice(),
            LegacySyntaxErrorKind::Lexical(LegacyLexErrorKind::UnterminatedComment),
            10,
        ),
        (
            b"main() {} \0private-secret".as_slice(),
            LegacySyntaxErrorKind::Lexical(LegacyLexErrorKind::NulByte),
            10,
        ),
    ] {
        let source = SourceText::index(bytes, SourceBudget::default()).expect("Source");
        let error = LegacySyntaxTree::parse_structure(
            &source,
            LegacySyntaxMode::Script,
            LegacySyntaxBudget::default(),
        )
        .expect_err("Invalid syntax");
        assert_eq!(error.kind, expected);
        assert_eq!(error.offset, offset);
        assert!(!format!("{error:?} {error}").contains("private"));
    }
}

#[test]
fn every_two_byte_input_fails_or_produces_bounded_original_spans() {
    for word in 0..=u16::MAX {
        let bytes = word.to_le_bytes();
        let source = SourceText::index(&bytes, SourceBudget::default()).expect("Source");
        match LegacySyntaxTree::parse_structure(
            &source,
            LegacySyntaxMode::Fragment,
            LegacySyntaxBudget::default(),
        ) {
            Ok(tree) => assert!(tree.nodes().all(|node| node.span().end <= 2)),
            Err(error) => {
                assert!(error.offset <= 2);
                assert_ne!(
                    error.kind,
                    LegacySyntaxErrorKind::InvalidTable,
                    "Input {word}"
                );
            }
        }
    }
}
