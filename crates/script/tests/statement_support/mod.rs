// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Helpers consume valid synthetic single-function modules."
)]

use cod4x_script::declaration::{DeclarationBudget, LegacyModuleItem, LegacyModuleSyntax};
use cod4x_script::statement::{
    LegacyStatementListSyntax, LegacyStatementSyntax, StatementListBudget,
};
use cod4x_script::syntax::{LegacySyntaxNode, LegacySyntaxTree};

pub fn body<'tree, 'source>(
    tree: &'tree LegacySyntaxTree<'source>,
) -> LegacySyntaxNode<'tree, 'source> {
    LegacyModuleSyntax::lower(tree, DeclarationBudget::default())
        .expect("Module")
        .items()
        .iter()
        .find_map(|item| match item {
            LegacyModuleItem::Function(function) => Some(function.body()),
            _ => None,
        })
        .expect("Synthetic function")
}

pub fn first<'tree, 'source>(
    tree: &'tree LegacySyntaxTree<'source>,
) -> LegacyStatementSyntax<'tree, 'source> {
    let list = LegacyStatementListSyntax::lower(body(tree), StatementListBudget::default())
        .expect("Body list");
    LegacyStatementSyntax::from_node(list.statements().next().expect("First statement"))
        .expect("Statement")
}
