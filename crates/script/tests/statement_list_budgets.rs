// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete body-list admission precedes allocation; failures keep actual work/counts.
#![allow(
    clippy::expect_used,
    reason = "Tests own finite valid bodies and exact admission limits."
)]

use cod4x_script::statement::{
    LegacyStatementListSyntax, StatementListBudget, StatementListErrorKind, StatementListResource,
};
use cod4x_script::syntax::LegacySyntaxMode;

use crate::expression_support;
use crate::statement_support;

#[test]
fn exact_and_one_less_limits_preflight_both_passes_and_complete_count_storage() {
    let tree = expression_support::parse(b"main(){x=1;y=2;return 3;}", LegacySyntaxMode::Script);
    let body = statement_support::body(&tree);
    let usage = LegacyStatementListSyntax::lower(body, StatementListBudget::default())
        .expect("Baseline")
        .usage();
    let exact = StatementListBudget {
        max_statements: usage.statements,
        max_steps: usage.steps,
        max_metadata_bytes: usage.metadata_bytes,
    };
    assert_eq!(
        LegacyStatementListSyntax::lower(body, exact)
            .expect("Exact")
            .usage(),
        usage
    );
    for (resource, budget) in [
        (
            StatementListResource::Statements,
            StatementListBudget {
                max_statements: exact.max_statements - 1,
                ..exact
            },
        ),
        (
            StatementListResource::Steps,
            StatementListBudget {
                max_steps: exact.max_steps - 1,
                ..exact
            },
        ),
        (
            StatementListResource::MetadataBytes,
            StatementListBudget {
                max_metadata_bytes: exact.max_metadata_bytes - 1,
                ..exact
            },
        ),
    ] {
        let error = LegacyStatementListSyntax::lower(body, budget).expect_err("One less");
        assert!(
            matches!(error.kind, StatementListErrorKind::BudgetExceeded { resource: actual, requested, maximum } if actual == resource && requested > maximum)
        );
        assert!(error.work_steps > 0 && error.work_steps <= usage.steps / 2);
        assert!(error.admitted_statements <= budget.max_statements);
        assert!(error.admitted_metadata_bytes <= budget.max_metadata_bytes);
        if resource == StatementListResource::Steps {
            assert_eq!(error.work_steps, 4);
            assert_eq!(error.admitted_statements, 3);
            assert_eq!(error.admitted_metadata_bytes, usage.metadata_bytes);
            assert_eq!(
                error.kind,
                StatementListErrorKind::BudgetExceeded {
                    resource,
                    requested: 8,
                    maximum: 7
                }
            );
        }
    }
    let zero = LegacyStatementListSyntax::lower(
        body,
        StatementListBudget {
            max_steps: 0,
            ..exact
        },
    )
    .expect_err("Zero work");
    assert_eq!(zero.work_steps, 0);
    assert_eq!(zero.admitted_statements, 0);
    assert_eq!(zero.admitted_metadata_bytes, 0);
}

#[test]
fn empty_bodies_need_two_visits_and_no_statement_or_index_storage() {
    let tree = expression_support::parse(b"main(){}", LegacySyntaxMode::Script);
    let body = statement_support::body(&tree);
    let exact = StatementListBudget {
        max_statements: 0,
        max_steps: 2,
        max_metadata_bytes: 0,
    };
    let list = LegacyStatementListSyntax::lower(body, exact).expect("Empty complete body");
    assert_eq!(list.usage().steps, 2);
    assert_eq!(list.usage().statements, 0);
    assert_eq!(list.usage().metadata_bytes, 0);
    assert_eq!(list.statements().len(), 0);
    let error = LegacyStatementListSyntax::lower(
        body,
        StatementListBudget {
            max_steps: 1,
            ..exact
        },
    )
    .expect_err("Second pass admission");
    assert_eq!(error.work_steps, 1);
    assert_eq!(
        error.kind,
        StatementListErrorKind::BudgetExceeded {
            resource: StatementListResource::Steps,
            requested: 2,
            maximum: 1
        }
    );
}
