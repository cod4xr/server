// SPDX-License-Identifier: AGPL-3.0-or-later

/// Eager legacy infix syntax, without choosing overloaded value/coercion semantics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyInfixOperator {
    /// Bitwise inclusive OR.
    BitwiseOr,
    /// Bitwise exclusive OR.
    BitwiseExclusiveOr,
    /// Bitwise AND.
    BitwiseAnd,
    /// Language equality, rather than Rust data equality.
    Equal,
    /// Language inequality.
    NotEqual,
    /// Less than.
    Less,
    /// Greater than.
    Greater,
    /// Less than or equal to.
    LessOrEqual,
    /// Greater than or equal to.
    GreaterOrEqual,
    /// Left shift.
    ShiftLeft,
    /// Right shift.
    ShiftRight,
    /// Addition, including any supported overloaded string/vector semantics.
    Add,
    /// Subtraction.
    Subtract,
    /// Multiplication.
    Multiply,
    /// Division.
    Divide,
    /// Signed remainder.
    Remainder,
}

impl LegacyInfixOperator {
    pub(super) fn from_production(rule: u16) -> Option<Self> {
        let index = usize::from(rule.checked_sub(8)?);
        [
            Self::BitwiseOr,
            Self::BitwiseExclusiveOr,
            Self::BitwiseAnd,
            Self::Equal,
            Self::NotEqual,
            Self::Less,
            Self::Greater,
            Self::LessOrEqual,
            Self::GreaterOrEqual,
            Self::ShiftLeft,
            Self::ShiftRight,
            Self::Add,
            Self::Subtract,
            Self::Multiply,
            Self::Divide,
            Self::Remainder,
        ]
        .get(index)
        .copied()
    }
}

/// Logical infix syntax requiring short-circuit control-flow lowering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyLogicalOperator {
    /// Evaluate the right operand only if the left operand is false.
    Or,
    /// Evaluate the right operand only if the left operand is true.
    And,
}

/// Legacy prefix syntax, independent of literal decoding or value coercion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyPrefixOperator {
    /// Logical negation.
    LogicalNot,
    /// Integer bitwise complement.
    BitwiseNot,
}
