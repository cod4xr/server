// SPDX-License-Identifier: AGPL-3.0-or-later
/// Logical scanner accounting without source contents; not elapsed CPU time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyLexUsage {
    /// Attempts admitted by the lexeme budget, including an attempt that failed.
    pub attempted_lexemes: u64,
    /// Transition/fallback attempts admitted by the work budget, including lookahead.
    pub steps: u64,
    /// Bytes committed to successfully returned lexemes; excludes lookahead/errors.
    pub consumed_bytes: u64,
}
