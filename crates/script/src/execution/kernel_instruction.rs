// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::{KernelBuiltin, KernelErrorKind, KernelValue};
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

/// Typed checked instruction profile, not native/serialized GSC bytecode or a full VM.
///
/// Jump targets index instructions in this program, independently of old short
/// offsets. Constants and stack operands do not appear in Debug output.
#[derive(Clone, Copy, Debug)]
pub enum KernelInstruction {
    /// Pushes copied data. Runtime string handles are rejected even when unreachable;
    /// immutable code cannot silently root references into a mutable heap.
    Constant(KernelValue),
    /// Pushes a checked immutable table row through the invocation's heap/cache.
    StringLiteral(usize),
    /// Pushes the same owned literal bytes with the distinct legacy localized tag.
    LocalizedStringLiteral(usize),
    /// Pushes an initialized local slot; the layout bounds every operand.
    LoadLocal(usize),
    /// Consumes one value and initializes/replaces a local slot, including Undefined.
    StoreLocal(usize),
    /// Calls a function in the complete verified module. Stack arguments run
    /// bottom-to-top last to first; parameter zero is popped first. Missing
    /// suffixes become Undefined. Not a native/source evaluation/serialized ABI.
    Call {
        /// Wide module-local function index, checked even in unreachable code.
        function: usize,
        /// Supplied operand count. Excess arguments fail when the call executes.
        arguments: usize,
    },
    /// Pushes an opaque reference to this module's checked function index.
    FunctionReference(usize),
    /// Pops the top function reference after bottom-to-top last/first arguments.
    /// The reference must belong to this module; arity checks occur at invocation.
    CallIndirect {
        /// Supplied argument count, excluding the top function reference.
        arguments: usize,
    },
    /// Calls a sealed deterministic builtin after all arguments have evaluated.
    /// Builtin-specific arity failures stay at this instruction; no frame is added.
    Builtin {
        /// Statically known builtin, without host/plugin dispatch.
        function: KernelBuiltin,
        /// Supplied arguments, bottom-to-top last to first.
        arguments: usize,
    },
    /// Consumes z/y/x in bottom-to-top stack order and pushes an x/y/z vector.
    /// Components must be numbers; this does not specify source evaluation order.
    MakeVector,
    /// Preserves an integer word and rejects other primitive categories, without coercion.
    RequireInteger,
    /// Consumes bottom-to-top index/object and pushes a copied indexed result.
    /// Vectors use strict integer component indices; plain strings return one byte
    /// before the first NUL and require an explicit heap context.
    Index,
    /// Replaces a plain string with its byte length before the first NUL, or a
    /// checked ordinary struct with integer one. Arrays/other categories reject.
    Size,
    /// Replaces a checked struct receiver with the named field value or Undefined.
    /// The operand indexes this function's immutable field-name table.
    LoadField(usize),
    /// Consumes bottom-to-top RHS/struct receiver and writes the named field.
    /// Undefined removes the field; no value is pushed. Names are code-owned bytes.
    StoreField(usize),
    /// Pops right then left operands. Plain-string addition requires a heap context;
    /// mixed string coercion and other string arithmetic remain unsupported.
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
    /// Duplicates the top copied value; heap snapshots retain their existing identity.
    Duplicate,
    /// Exchanges the top two copied values without allocation or changing height.
    Swap,
    /// Discards the top copied value without collecting its backing storage.
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
    pub(super) fn stack_effect(self) -> Result<(usize, usize), KernelErrorKind> {
        let effect = match self {
            Self::CallIndirect { arguments } => (
                arguments
                    .checked_add(1)
                    .ok_or(KernelErrorKind::SizeOverflow)?,
                1,
            ),
            Self::Call { arguments, .. } | Self::Builtin { arguments, .. } => (arguments, 1),
            Self::MakeVector => (3, 1),
            Self::Constant(_)
            | Self::LoadLocal(_)
            | Self::FunctionReference(_)
            | Self::StringLiteral(_)
            | Self::LocalizedStringLiteral(_) => (0, 1),
            Self::Arithmetic(_)
            | Self::Compare(_)
            | Self::Integer(_)
            | Self::Divide
            | Self::Index => (2, 1),
            Self::CastBool
            | Self::LogicalNot
            | Self::BitwiseNot
            | Self::IntegerUpdate(_)
            | Self::Size
            | Self::LoadField(_)
            | Self::RequireInteger => (1, 1),
            Self::Duplicate => (1, 2),
            Self::Swap => (2, 2),
            Self::StoreField(_) => (2, 0),
            Self::Discard | Self::StoreLocal(_) | Self::Branch { .. } | Self::Return => (1, 0),
            Self::Jump(_) => (0, 0),
        };
        Ok(effect)
    }
}
