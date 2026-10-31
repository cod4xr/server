// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::expression::{LegacyInfixOperator, LegacyLogicalOperator, LegacyPrefixOperator};
use crate::syntax::LegacySyntaxNode;

/// Immediate expression shape; operand descendants remain unvalidated syntax.
#[derive(Clone, Copy, Debug)]
pub enum LegacyExpressionKind<'tree, 'source> {
    /// A primitive production requiring separate literal/call/reference lowering.
    Primary {
        /// Exact primitive node, without assuming a value or removing parentheses.
        syntax: LegacySyntaxNode<'tree, 'source>,
    },
    /// A prefix operator and its original operand.
    Prefix {
        /// Operator identity without choosing value semantics.
        operator: LegacyPrefixOperator,
        /// Unevaluated expression node.
        operand: LegacySyntaxNode<'tree, 'source>,
    },
    /// An eager infix operator, distinct from short-circuit logic.
    Infix {
        /// Operator identity; addition does not imply numeric-only operands.
        operator: LegacyInfixOperator,
        /// Original left expression.
        left: LegacySyntaxNode<'tree, 'source>,
        /// Original right expression.
        right: LegacySyntaxNode<'tree, 'source>,
    },
    /// A logical branch requiring lazy right-operand lowering.
    Logical {
        /// Logical OR/AND identity.
        operator: LegacyLogicalOperator,
        /// Expression evaluated before deciding whether to evaluate the right.
        left: LegacySyntaxNode<'tree, 'source>,
        /// Conditionally evaluated expression, never eagerly folded here.
        right: LegacySyntaxNode<'tree, 'source>,
    },
}
