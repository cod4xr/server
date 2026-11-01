// SPDX-License-Identifier: AGPL-3.0-or-later
/// Immutable internal switch operand, never a native string ID or mutable heap handle.
/// Derived Rust equality compares operands; different label rows can hold equal bytes.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum KernelSwitchKey {
    /// Complete signed integer word, independent of native encoded-ID capacities.
    Integer(i32),
    /// Wide index into this function's separately owned immutable label-byte table.
    StringLabel(usize),
}

impl std::fmt::Debug for KernelSwitchKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Integer(_) => "KernelSwitchKey::Integer(..)",
            Self::StringLabel(_) => "KernelSwitchKey::StringLabel(..)",
        })
    }
}

/// Sealed predicates over the internal normalized switch-key order.
/// Integers precede plain strings; integer words are signed and strings compare
/// complete bytes. Categories never compare equal. This is not native ID ordering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelSwitchComparison {
    /// Exact same-category key identity.
    Equal,
    /// Strictly earlier in the documented normalized order.
    Less,
}
