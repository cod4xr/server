// SPDX-License-Identifier: AGPL-3.0-or-later
/// Configurable scanner work/output limits, separate from source admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyLexBudget {
    /// Returned lexemes, including whitespace/comment pieces; no output is owned.
    pub max_lexemes: u64,
    /// DFA transition/fallback attempts across the complete scan, including retries.
    ///
    /// This is logical work accounting, not a wall-clock deadline or CPU cycles.
    pub max_steps: u64,
    /// A single borrowed lexeme's byte length, including any literal delimiters.
    pub max_lexeme_bytes: u64,
}

impl Default for LegacyLexBudget {
    fn default() -> Self {
        Self {
            max_lexemes: 1_000_000,
            max_steps: 256 * 1024 * 1024,
            max_lexeme_bytes: 64 * 1024 * 1024,
        }
    }
}
