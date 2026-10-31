// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::expression::LegacyInfixOperator;
use crate::statement::statement_profile;
use crate::statement::{
    LegacyEventKind, LegacyProfileKind, LegacyStatementError, LegacyStatementKind, LegacyUpdateKind,
};
use crate::syntax::LegacySyntaxNode;

/// Bounded allocation-free statement view, preserving supplied syntax and span.
#[derive(Clone, Copy, Debug)]
pub struct LegacyStatementSyntax<'tree, 'source> {
    syntax: LegacySyntaxNode<'tree, 'source>,
    statement: LegacySyntaxNode<'tree, 'source>,
    kind: LegacyStatementKind<'tree, 'source>,
}

impl<'tree, 'source> LegacyStatementSyntax<'tree, 'source> {
    /// Classifies a direct statement or pinned statement wrapper locally.
    ///
    /// Does not walk descendants, bind names, validate lvalues/control flow,
    /// activate developer blocks, interpret event lists, or perform execution.
    /// Optional for-condition/update nodes and body lists remain distinct syntax.
    /// Consumers must independently budget descendant traversal.
    ///
    /// # Errors
    /// Rejects unrelated nodes and mismatched child profiles. No input mutation,
    /// source copying, allocation, or partial runtime action occurs.
    pub fn from_node(
        syntax: LegacySyntaxNode<'tree, 'source>,
    ) -> Result<Self, LegacyStatementError> {
        let node = statement_profile::strip_wrappers(syntax)?;
        let kind = match statement_profile::production(node) {
            Some(rule @ 67..=93) => basic(node, rule)?,
            Some(rule @ 98..=107) => control(node, rule)?,
            _ => return Err(LegacyStatementError::NotStatement),
        };
        Ok(Self {
            syntax,
            statement: node,
            kind,
        })
    }

    /// Returns the local role without selecting runtime scope, values, or authority.
    #[must_use]
    pub fn kind(self) -> LegacyStatementKind<'tree, 'source> {
        self.kind
    }

    /// Returns the originally supplied scoped node, including any statement wrapper.
    #[must_use]
    pub fn syntax(self) -> LegacySyntaxNode<'tree, 'source> {
        self.syntax
    }

    /// Returns the direct statement node after bounded wrapper removal.
    ///
    /// This preserves the underlying operation span for code generation; it
    /// does not replace the original supplied node returned by `Self::syntax`.
    #[must_use]
    pub fn statement_syntax(self) -> LegacySyntaxNode<'tree, 'source> {
        self.statement
    }

    /// Returns the supplied node's original half-open source byte span.
    #[must_use]
    pub fn span(self) -> Range<u64> {
        self.syntax.span()
    }
}

fn basic<'tree, 'source>(
    node: LegacySyntaxNode<'tree, 'source>,
    rule: u16,
) -> Result<LegacyStatementKind<'tree, 'source>, LegacyStatementError> {
    let child = |index, count| statement_profile::child(node, index, count);
    Ok(match rule {
        67 => LegacyStatementKind::Assignment {
            target: child(0, 3)?,
            value: child(2, 3)?,
        },
        68 => LegacyStatementKind::Return {
            value: Some(child(1, 2)?),
        },
        69 => {
            child(0, 1)?;
            LegacyStatementKind::Return { value: None }
        }
        70 => LegacyStatementKind::Wait {
            duration: child(1, 2)?,
        },
        71 | 72 => LegacyStatementKind::Update {
            target: child(0, 2)?,
            kind: if rule == 71 {
                LegacyUpdateKind::Increment
            } else {
                LegacyUpdateKind::Decrement
            },
        },
        73..=82 => LegacyStatementKind::CompoundAssignment {
            target: child(0, 3)?,
            operator: compound(rule)?,
            value: child(2, 3)?,
        },
        83 | 84 | 86 => LegacyStatementKind::Event {
            kind: match rule {
                83 => LegacyEventKind::WaitTill,
                84 => LegacyEventKind::WaitTillMatch,
                _ => LegacyEventKind::Notify,
            },
            receiver: child(0, 5)?,
            arguments: child(3, 5)?,
        },
        85 => {
            child(0, 1)?;
            LegacyStatementKind::WaitTillFrameEnd
        }
        87 => LegacyStatementKind::EndOn {
            receiver: child(0, 5)?,
            event: child(3, 5)?,
        },
        88 => {
            child(0, 1)?;
            LegacyStatementKind::Break
        }
        89 => {
            child(0, 1)?;
            LegacyStatementKind::Continue
        }
        90 => {
            child(0, 1)?;
            LegacyStatementKind::Breakpoint
        }
        91 | 92 => LegacyStatementKind::Profile {
            kind: if rule == 91 {
                LegacyProfileKind::Begin
            } else {
                LegacyProfileKind::End
            },
            label: child(2, 4)?
                .lexeme()
                .ok_or(LegacyStatementError::ProfileMismatch)?,
        },
        93 => LegacyStatementKind::Call {
            invocation: child(0, 1)?,
        },
        _ => return Err(LegacyStatementError::NotStatement),
    })
}

fn control<'tree, 'source>(
    node: LegacySyntaxNode<'tree, 'source>,
    rule: u16,
) -> Result<LegacyStatementKind<'tree, 'source>, LegacyStatementError> {
    let child = |index, count| statement_profile::child(node, index, count);
    Ok(match rule {
        98 => LegacyStatementKind::Block { body: child(1, 3)? },
        99 => LegacyStatementKind::If {
            condition: child(2, 5)?,
            consequence: child(4, 5)?,
            alternative: None,
        },
        100 => LegacyStatementKind::If {
            condition: child(2, 7)?,
            consequence: child(4, 7)?,
            alternative: Some(child(6, 7)?),
        },
        101 => LegacyStatementKind::While {
            condition: child(2, 5)?,
            body: child(4, 5)?,
        },
        102 => LegacyStatementKind::For {
            initializer: child(2, 8)?,
            condition: child(3, 8)?,
            update: child(5, 8)?,
            body: child(7, 8)?,
        },
        103 => LegacyStatementKind::Switch {
            expression: child(2, 7)?,
            body: child(5, 7)?,
        },
        104 => LegacyStatementKind::DeveloperBlock { body: child(1, 3)? },
        105 => {
            child(0, 1)?;
            LegacyStatementKind::Empty
        }
        106 => LegacyStatementKind::Case {
            value: child(1, 3)?,
        },
        107 => {
            child(0, 2)?;
            LegacyStatementKind::Default
        }
        _ => return Err(LegacyStatementError::NotStatement),
    })
}

fn compound(rule: u16) -> Result<LegacyInfixOperator, LegacyStatementError> {
    Ok(match rule {
        73 => LegacyInfixOperator::BitwiseOr,
        74 => LegacyInfixOperator::BitwiseExclusiveOr,
        75 => LegacyInfixOperator::BitwiseAnd,
        76 => LegacyInfixOperator::ShiftLeft,
        77 => LegacyInfixOperator::ShiftRight,
        78 => LegacyInfixOperator::Add,
        79 => LegacyInfixOperator::Subtract,
        80 => LegacyInfixOperator::Multiply,
        81 => LegacyInfixOperator::Divide,
        82 => LegacyInfixOperator::Remainder,
        _ => return Err(LegacyStatementError::ProfileMismatch),
    })
}
