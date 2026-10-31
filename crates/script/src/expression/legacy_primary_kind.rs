// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::lexical::LegacyLexeme;
use crate::syntax::LegacySyntaxNode;

/// Source literal polarity, before checked numeric conversion or value creation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyLiteralPolarity {
    /// Keep the literal token's value; used for all nonnumeric literals too.
    Unchanged,
    /// A separate minus terminal precedes an integer/float literal token.
    Negated,
}

/// Special object-root syntax, without resolving an object or assigning a handle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyObjectRoot {
    /// The source self root, whose actual value depends on execution context.
    CurrentObject,
    /// The source level root.
    Level,
    /// The source game root.
    Game,
    /// The source anim root.
    Animation,
}

/// Immediate primary shape, without validating or binding descendant syntax.
#[derive(Clone, Copy, Debug)]
pub enum LegacyPrimaryKind<'tree, 'source> {
    /// Raw literal spelling and source polarity, without decoding its value.
    Literal {
        /// Exact original token, excluding a separate minus terminal.
        token: LegacyLexeme<'source>,
        /// Whether the source uses a signed numeric-literal production.
        polarity: LegacyLiteralPolarity,
    },
    /// Original parenthesized list; group/vector arity is not decided here.
    Parenthesized {
        /// Structural expression list awaiting ordered list lowering.
        expressions: LegacySyntaxNode<'tree, 'source>,
    },
    /// Invocation subtree awaiting callee/receiver/argument lowering.
    Call {
        /// Complete original call production, including thread/method syntax.
        syntax: LegacySyntaxNode<'tree, 'source>,
    },
    /// Variable/field/index subtree awaiting reference lowering and binding.
    Reference {
        /// Original reference syntax, without interned names or object IDs.
        syntax: LegacySyntaxNode<'tree, 'source>,
    },
    /// Function-reference subtree awaiting path/name resolution.
    FunctionReference {
        /// Original local/external function-reference syntax.
        syntax: LegacySyntaxNode<'tree, 'source>,
    },
    /// A special object-root expression, without fetching its runtime value.
    Object(LegacyObjectRoot),
    /// Size access awaiting operand/value validation.
    Size {
        /// Original primitive operand before the .size terminal.
        object: LegacySyntaxNode<'tree, 'source>,
    },
    /// Empty-array syntax; creates no runtime allocation or object here.
    EmptyArray,
    /// Named animation syntax, without interning/resolving the animation name.
    Animation {
        /// Exact identifier token following the percent terminal.
        name: LegacyLexeme<'source>,
    },
    /// The #animtree form, without a loaded animation tree or runtime value.
    AnimationTree,
    /// Legacy breakon syntax; does not enable a debugger or execute a condition.
    BreakOn {
        /// Original primitive expression associated with the break condition.
        object: LegacySyntaxNode<'tree, 'source>,
        /// Original condition expression, left unevaluated.
        condition: LegacySyntaxNode<'tree, 'source>,
    },
}
