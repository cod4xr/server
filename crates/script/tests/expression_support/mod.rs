// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Helpers index and parse bounded synthetic expression sources."
)]

use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{
    LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxNode, LegacySyntaxTree,
};

pub fn parse(bytes: &[u8], mode: LegacySyntaxMode) -> LegacySyntaxTree<'_> {
    let source = SourceText::index(bytes, SourceBudget::default()).expect("Source");
    LegacySyntaxTree::parse_structure(&source, mode, LegacySyntaxBudget::default()).expect("Syntax")
}

pub fn bytes<'source>(
    tree: &LegacySyntaxTree<'source>,
    node: LegacySyntaxNode<'_, '_>,
) -> &'source [u8] {
    let span = node.span();
    let start = usize::try_from(span.start).expect("Start");
    let end = usize::try_from(span.end).expect("End");
    &tree.source_bytes()[start..end]
}
