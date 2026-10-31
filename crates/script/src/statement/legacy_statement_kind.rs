// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::expression::LegacyInfixOperator;
use crate::lexical::LegacyLexeme;
use crate::syntax::LegacySyntaxNode;

/// Syntactic update direction; no lvalue or numeric-type guarantee is implied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyUpdateKind {
    /// Postfix ++ spelling.
    Increment,
    /// Postfix -- spelling.
    Decrement,
}

/// Event statement family, each retaining its distinct unresolved argument profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyEventKind {
    /// waittill with the event/output-name list, productions 119/120.
    WaitTill,
    /// waittillmatch with its matching-argument list, productions 121/122.
    WaitTillMatch,
    /// notify with its event/argument list, productions 123/124.
    Notify,
}

/// Profiling source spelling; this does not grant runtime profiling authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyProfileKind {
    /// `prof_begin` spelling.
    Begin,
    /// `prof_end` spelling.
    End,
}

/// Local statement roles; all descendant syntax/labels remain unresolved.
#[derive(Clone, Copy, Debug)]
pub enum LegacyStatementKind<'tree, 'source> {
    /// Assignment; target is not yet a validated writable reference.
    Assignment {
        /// Original target syntax.
        target: LegacySyntaxNode<'tree, 'source>,
        /// Original value expression.
        value: LegacySyntaxNode<'tree, 'source>,
    },
    /// Compound assignment; operator does not imply numeric-only runtime semantics.
    CompoundAssignment {
        /// Original target syntax.
        target: LegacySyntaxNode<'tree, 'source>,
        /// Compound operation, preserving the eager operator identity.
        operator: LegacyInfixOperator,
        /// Original right expression.
        value: LegacySyntaxNode<'tree, 'source>,
    },
    /// Postfix increment/decrement statement.
    Update {
        /// Original target syntax.
        target: LegacySyntaxNode<'tree, 'source>,
        /// Update direction.
        kind: LegacyUpdateKind,
    },
    /// Return with an optional original expression.
    Return {
        /// None for a bare return.
        value: Option<LegacySyntaxNode<'tree, 'source>>,
    },
    /// Wait duration expression, without frame conversion or scheduling.
    Wait {
        /// Original duration expression.
        duration: LegacySyntaxNode<'tree, 'source>,
    },
    /// Function/method invocation statement, without linking or execution.
    Call {
        /// Original invocation node.
        invocation: LegacySyntaxNode<'tree, 'source>,
    },
    /// Event statement with a family-specific opaque argument list.
    Event {
        /// Event family, not an executable subscription.
        kind: LegacyEventKind,
        /// Original receiver primary.
        receiver: LegacySyntaxNode<'tree, 'source>,
        /// Specialized event argument syntax, not an ordinary expression-list node.
        arguments: LegacySyntaxNode<'tree, 'source>,
    },
    /// endon with one event expression, not an event argument-list profile.
    EndOn {
        /// Original receiver primary.
        receiver: LegacySyntaxNode<'tree, 'source>,
        /// Original event expression.
        event: LegacySyntaxNode<'tree, 'source>,
    },
    /// waittillframeend; no scheduler action is performed.
    WaitTillFrameEnd,
    /// break; enclosing-loop/switch legality is unresolved.
    Break,
    /// continue; enclosing-loop legality is unresolved.
    Continue,
    /// breakpoint; runtime/debugger availability is unresolved.
    Breakpoint,
    /// Profile marker with a raw label, without decoding or profiling side effects.
    Profile {
        /// Begin/end spelling.
        kind: LegacyProfileKind,
        /// Original label token.
        label: LegacyLexeme<'source>,
    },
    /// Ordinary block with a separate unflattened body list.
    Block {
        /// Original body-list production.
        body: LegacySyntaxNode<'tree, 'source>,
    },
    /// Developer block, without selecting its activation policy.
    DeveloperBlock {
        /// Original body-list production.
        body: LegacySyntaxNode<'tree, 'source>,
    },
    /// Conditional with original branches and optional else branch.
    If {
        /// Original condition expression.
        condition: LegacySyntaxNode<'tree, 'source>,
        /// Original then statement.
        consequence: LegacySyntaxNode<'tree, 'source>,
        /// Original else statement, when present.
        alternative: Option<LegacySyntaxNode<'tree, 'source>>,
    },
    /// while loop, without control-flow validation or execution.
    While {
        /// Original condition expression.
        condition: LegacySyntaxNode<'tree, 'source>,
        /// Original loop statement.
        body: LegacySyntaxNode<'tree, 'source>,
    },
    /// for loop retaining the distinct initializer/optional condition/update roles.
    For {
        /// Original initializer statement.
        initializer: LegacySyntaxNode<'tree, 'source>,
        /// Original optional condition, production 26/27.
        condition: LegacySyntaxNode<'tree, 'source>,
        /// Original optional update, production 95/96.
        update: LegacySyntaxNode<'tree, 'source>,
        /// Original loop statement.
        body: LegacySyntaxNode<'tree, 'source>,
    },
    /// Switch with a body list retaining labels/statements in source order.
    Switch {
        /// Original switched expression.
        expression: LegacySyntaxNode<'tree, 'source>,
        /// Original body-list production.
        body: LegacySyntaxNode<'tree, 'source>,
    },
    /// Case label; value and enclosing-switch legality are unresolved.
    Case {
        /// Original label primary.
        value: LegacySyntaxNode<'tree, 'source>,
    },
    /// Default label; uniqueness/enclosing-switch legality are unresolved.
    Default,
    /// A literal semicolon statement, distinct from an empty body list.
    Empty,
}
