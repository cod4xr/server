// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exact admission for both passes and complete output metadata precedes allocation.
#![allow(
    clippy::expect_used,
    reason = "Tests own valid synthetic list inputs and exact successful accounting."
)]

use crate::expression_list_support;
use crate::expression_support;

use cod4x_script::expression::{
    ExpressionListBudget, ExpressionListErrorKind, ExpressionListResource,
    LegacyExpressionListSyntax,
};
use cod4x_script::syntax::LegacySyntaxMode;

#[test]
fn exact_budgets_succeed_and_each_one_less_failure_identifies_the_exhausted_resource()
-> Result<(), String> {
    let text = b"(private_a,private_b,private_c)";
    let tree = expression_support::parse(text, LegacySyntaxMode::Fragment);
    let node = expression_list_support::node(&tree)?;
    let list =
        LegacyExpressionListSyntax::lower(node, ExpressionListBudget::default()).expect("Baseline");
    let usage = list.usage();
    let exact = ExpressionListBudget {
        max_expressions: usage.expressions,
        max_steps: usage.steps,
        max_metadata_bytes: usage.metadata_bytes,
    };
    let repeated = LegacyExpressionListSyntax::lower(node, exact).expect("Exact");
    assert_eq!(repeated.usage(), usage);
    for (resource, requested, maximum, budget) in [
        (
            ExpressionListResource::Expressions,
            usage.expressions,
            usage.expressions - 1,
            ExpressionListBudget {
                max_expressions: usage.expressions - 1,
                ..exact
            },
        ),
        (
            ExpressionListResource::Steps,
            usage.steps,
            usage.steps - 1,
            ExpressionListBudget {
                max_steps: usage.steps - 1,
                ..exact
            },
        ),
        (
            ExpressionListResource::MetadataBytes,
            usage.metadata_bytes,
            usage.metadata_bytes - 1,
            ExpressionListBudget {
                max_metadata_bytes: usage.metadata_bytes - 1,
                ..exact
            },
        ),
    ] {
        let error = LegacyExpressionListSyntax::lower(node, budget).expect_err("One less");
        assert_eq!(
            error.kind,
            ExpressionListErrorKind::BudgetExceeded {
                resource,
                requested,
                maximum
            }
        );
        assert_eq!(error.offset, node.span().start);
        assert!(!format!("{error:?} {error}").contains("private"));
    }
    assert_eq!(
        expression_support::bytes(&tree, node),
        b"private_a,private_b,private_c"
    );
    assert_eq!(tree.source_bytes(), text);
    Ok(())
}

#[test]
fn zero_work_and_the_second_pass_are_admitted_before_any_output_is_published() -> Result<(), String>
{
    let tree = expression_support::parse(b"()", LegacySyntaxMode::Fragment);
    let node = expression_list_support::node(&tree)?;
    for (maximum, requested) in [(0, 1), (1, 2)] {
        let error = LegacyExpressionListSyntax::lower(
            node,
            ExpressionListBudget {
                max_steps: maximum,
                ..ExpressionListBudget::default()
            },
        )
        .expect_err("Work limit");
        assert_eq!(
            error.kind,
            ExpressionListErrorKind::BudgetExceeded {
                resource: ExpressionListResource::Steps,
                requested,
                maximum
            }
        );
    }
    assert!(expression_support::bytes(&tree, node).is_empty());
    Ok(())
}
