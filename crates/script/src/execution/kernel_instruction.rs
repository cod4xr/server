// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::KernelValue;
use crate::legacy::{LegacyArithmetic, LegacyComparison, LegacyIntegerOperator};

/// Numeric branch predicate; consumes its condition value without evaluating a source tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelBranchCondition {
    /// Take the target if numeric truth is false.
    IfFalse,
    /// Take the target if numeric truth is true.
    IfTrue,
}

/// Typed scalar instruction profile, not native/serialized GSC bytecode or a full VM.
///
/// Jump targets index instructions in this program, independently of old short
/// offsets. Constants and stack operands do not appear in Debug output.
#[derive(Clone, Copy, Debug)]
pub enum KernelInstruction {
    /// Pushes an independently copied scalar value.
    Constant(KernelValue),
    /// Pops right then left numbers and pushes the coerced arithmetic result.
    Arithmetic(LegacyArithmetic),
    /// Pops right then left numbers and pushes the language comparison integer.
    Compare(LegacyComparison),
    /// Pops right then left numbers and applies an integer-only operator.
    Integer(LegacyIntegerOperator),
    /// Pops right then left numbers and pushes float-result division.
    Divide,
    /// Converts numeric truth to integer zero or one.
    CastBool,
    /// Applies numeric logical negation.
    LogicalNot,
    /// Applies integer-only bitwise complement.
    BitwiseNot,
    /// Duplicates the top scalar value.
    Duplicate,
    /// Discards the top scalar value.
    Discard,
    /// Continues at an absolute instruction index.
    Jump(usize),
    /// Consumes numeric truth and selects the target or following instruction.
    Branch {
        /// Numeric truth predicate.
        condition: KernelBranchCondition,
        /// Absolute instruction index in this owned program.
        target: usize,
    },
    /// Returns the sole live stack value; other return stack shapes are invalid.
    Return,
}

impl KernelInstruction {
    pub(super) fn stack_effect(self) -> (usize, usize) {
        match self {
            Self::Constant(_) => (0, 1),
            Self::Arithmetic(_) | Self::Compare(_) | Self::Integer(_) | Self::Divide => (2, 1),
            Self::CastBool | Self::LogicalNot | Self::BitwiseNot => (1, 1),
            Self::Duplicate => (1, 2),
            Self::Discard | Self::Branch { .. } | Self::Return => (1, 0),
            Self::Jump(_) => (0, 0),
        }
    }
}
