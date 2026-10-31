// SPDX-License-Identifier: AGPL-3.0-or-later
/// A legacy grammar entry point, not a modern language-version switch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacySyntaxMode {
    /// Complete script source, using the loader's virtual plus selector.
    Script,
    /// Structural fragment entry selected by the grammar's virtual star token.
    ///
    /// Accepts grammar-supported expressions and statements. This does not implement
    /// evaluation, debugger state, or host authorization.
    Fragment,
}

impl LegacySyntaxMode {
    pub(super) fn selector(self) -> u16 {
        match self {
            Self::Script => 0x118,
            Self::Fragment => 0x11a,
        }
    }
}
