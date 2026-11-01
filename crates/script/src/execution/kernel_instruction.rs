// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::KernelValue;
use crate::legacy::{
    LegacyArithmetic, LegacyComparison, LegacyIntegerOperator, LegacyIntegerUpdate,
};

/// Numeric branch predicate; consumes its condition value without evaluating a source tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelBranchCondition {
    /// Take the target if numeric truth is false.
    IfFalse,
    /// Take the target if numeric truth is true.
    IfTrue,
}

/// Typed primitive instruction profile, not native/serialized GSC bytecode or a full VM.
///
/// Jump targets index instructions in this program, independently of old short
/// offsets. Constants and stack operands do not appear in Debug output.
#[derive(Clone, Copy, Debug)]
pub enum KernelInstruction {
    /// Pushes an independently copied primitive value.
    Constant(KernelValue),
    /// Pushes an initialized local slot; the layout bounds every operand.
    LoadLocal(usize),
    /// Consumes one value and initializes/replaces a local slot, including Undefined.
    StoreLocal(usize),
    /// Consumes z/y/x in bottom-to-top stack order and pushes an x/y/z vector.
    /// Components must be numbers; this does not specify source evaluation order.
    MakeVector,
    /// Consumes bottom-to-top index/object and pushes a copied indexed result.
    /// The current primitive profile indexes vectors only, with strict integer indices.
    Index,
    /// Pops right then left numeric/vector operands and pushes the coerced result.
    Arithmetic(LegacyArithmetic),
    /// Pops right then left primitives and pushes the legacy comparison integer.
    /// Two Undefined values support equality only; mixed Undefined pairs fail.
    Compare(LegacyComparison),
    /// Pops right then left numbers and applies an integer-only operator.
    Integer(LegacyIntegerOperator),
    /// Applies one integer-only increment/decrement, without selecting a writable reference.
    IntegerUpdate(LegacyIntegerUpdate),
    /// Pops right then left numeric/vector operands and pushes float/vector division.
    Divide,
    /// Converts numeric truth to integer zero or one.
    CastBool,
    /// Applies numeric logical negation.
    LogicalNot,
    /// Applies integer-only bitwise complement.
    BitwiseNot,
    /// Duplicates the top primitive value.
    Duplicate,
    /// Discards the top primitive value.
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
            Self::MakeVector => (3, 1),
            Self::Constant(_) | Self::LoadLocal(_) => (0, 1),
            Self::Arithmetic(_)
            | Self::Compare(_)
            | Self::Integer(_)
            | Self::Divide
            | Self::Index => (2, 1),
            Self::CastBool | Self::LogicalNot | Self::BitwiseNot | Self::IntegerUpdate(_) => (1, 1),
            Self::Duplicate => (1, 2),
            Self::Discard | Self::StoreLocal(_) | Self::Branch { .. } | Self::Return => (1, 0),
            Self::Jump(_) => (0, 0),
        }
    }
}
