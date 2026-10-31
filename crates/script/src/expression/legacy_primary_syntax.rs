// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::expression::legacy_syntax_profile;
use crate::expression::{
    LegacyExpressionError, LegacyExpressionKind, LegacyExpressionSyntax, LegacyLiteralPolarity,
    LegacyObjectRoot, LegacyPrimaryKind,
};
use crate::syntax::LegacySyntaxNode;

/// Constant-cost borrowed primary view, retaining the original supplied syntax.
#[derive(Clone, Copy, Debug)]
pub struct LegacyPrimarySyntax<'tree, 'source> {
    syntax: LegacySyntaxNode<'tree, 'source>,
    kind: LegacyPrimaryKind<'tree, 'source>,
}

impl<'tree, 'source> LegacyPrimarySyntax<'tree, 'source> {
    /// Classifies a primary locally, recognizing the expression view's wrappers.
    ///
    /// Reads a bounded number of children without allocation, recursion, literal
    /// conversion, descendant traversal, list admission, binding, or evaluation.
    /// Signed literal tokens and grouped lists retain their original syntax.
    /// Animation/debugger forms do not grant or enable runtime services.
    ///
    /// # Errors
    /// Rejects nonprimary expressions, nonexpression nodes, or a mismatched pinned
    /// production shape. Source/tree state remains unchanged on every outcome.
    pub fn from_node(
        syntax: LegacySyntaxNode<'tree, 'source>,
    ) -> Result<Self, LegacyExpressionError> {
        let expression = LegacyExpressionSyntax::from_node(syntax)?;
        let LegacyExpressionKind::Primary { syntax: primary } = expression.kind() else {
            return Err(LegacyExpressionError::NotPrimary);
        };
        let kind = classify(primary)?;
        Ok(Self { syntax, kind })
    }

    /// Returns the local primary role without validating any descendants.
    #[must_use]
    pub fn kind(self) -> LegacyPrimaryKind<'tree, 'source> {
        self.kind
    }

    /// Returns the original supplied scoped node, preserving any entry wrapper.
    #[must_use]
    pub fn syntax(self) -> LegacySyntaxNode<'tree, 'source> {
        self.syntax
    }

    /// Returns the original supplied node's half-open byte span.
    #[must_use]
    pub fn span(self) -> Range<u64> {
        self.syntax.span()
    }
}

fn classify<'tree, 'source>(
    node: LegacySyntaxNode<'tree, 'source>,
) -> Result<LegacyPrimaryKind<'tree, 'source>, LegacyExpressionError> {
    let rule = legacy_syntax_profile::production(node).ok_or(LegacyExpressionError::NotPrimary)?;
    let count = match rule {
        40 => 3,
        43 | 44 | 54 | 56 | 57 => 2,
        61 => 5,
        41..=60 => 1,
        _ => return Err(LegacyExpressionError::NotPrimary),
    };
    if node.children().len() != count {
        return Err(LegacyExpressionError::ProfileMismatch);
    }
    let child = |index, count| legacy_syntax_profile::child(node, index, count);
    Ok(match rule {
        40 => LegacyPrimaryKind::Parenthesized {
            expressions: child(1, 3)?,
        },
        41 | 42 | 45 | 46 | 49 | 58 | 59 => literal(node, 0, 1, LegacyLiteralPolarity::Unchanged)?,
        43 | 44 => literal(node, 1, 2, LegacyLiteralPolarity::Negated)?,
        47 => LegacyPrimaryKind::Call {
            syntax: child(0, 1)?,
        },
        48 => LegacyPrimaryKind::Reference {
            syntax: child(0, 1)?,
        },
        50 => LegacyPrimaryKind::Object(LegacyObjectRoot::CurrentObject),
        51 => LegacyPrimaryKind::Object(LegacyObjectRoot::Level),
        52 => LegacyPrimaryKind::Object(LegacyObjectRoot::Game),
        53 => LegacyPrimaryKind::Object(LegacyObjectRoot::Animation),
        54 => LegacyPrimaryKind::Size {
            object: child(0, 2)?,
        },
        55 => LegacyPrimaryKind::FunctionReference {
            syntax: child(0, 1)?,
        },
        56 => LegacyPrimaryKind::EmptyArray,
        57 => LegacyPrimaryKind::Animation {
            name: child(1, 2)?
                .lexeme()
                .ok_or(LegacyExpressionError::ProfileMismatch)?,
        },
        60 => LegacyPrimaryKind::AnimationTree,
        61 => LegacyPrimaryKind::BreakOn {
            object: child(0, 5)?,
            condition: child(3, 5)?,
        },
        _ => return Err(LegacyExpressionError::NotPrimary),
    })
}

fn literal<'tree, 'source>(
    node: LegacySyntaxNode<'tree, 'source>,
    index: usize,
    count: usize,
    polarity: LegacyLiteralPolarity,
) -> Result<LegacyPrimaryKind<'tree, 'source>, LegacyExpressionError> {
    let token = legacy_syntax_profile::child(node, index, count)?
        .lexeme()
        .ok_or(LegacyExpressionError::ProfileMismatch)?;
    Ok(LegacyPrimaryKind::Literal { token, polarity })
}
