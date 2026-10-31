// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_script::expression::{LegacyPrimaryKind, LegacyPrimarySyntax};
use cod4x_script::syntax::{LegacySyntaxNode, LegacySyntaxTree};

pub fn node<'tree, 'source>(
    tree: &'tree LegacySyntaxTree<'source>,
) -> Result<LegacySyntaxNode<'tree, 'source>, String> {
    let primary = LegacyPrimarySyntax::from_node(tree.root()).map_err(|error| error.to_string())?;
    match primary.kind() {
        LegacyPrimaryKind::Parenthesized { expressions } => Ok(expressions),
        _ => Err("Expected parenthesized expression list".to_owned()),
    }
}
