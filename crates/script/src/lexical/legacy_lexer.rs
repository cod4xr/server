// SPDX-License-Identifier: AGPL-3.0-or-later
use std::iter::FusedIterator;

use crate::lexical::legacy_scanner::{self, ScanMode};
use crate::lexical::legacy_tables;
use crate::lexical::{
    LegacyLexBudget, LegacyLexError, LegacyLexErrorKind, LegacyLexResource, LegacyLexUsage,
    LegacyLexeme,
};
use crate::source::SourceText;

/// Bounded allocation-free legacy lexing, retaining trivia and exact source spans.
///
/// Data/action codes are recovered from the pinned source. This does not decode
/// literals/numbers, intern names, parse syntax, or execute bytecode. Modern keywords
/// are not enabled implicitly. NULs/invalid bytes fail explicitly rather than
/// truncating input. After the first error, iteration is permanently finished.
pub struct LegacyLexer<'source> {
    bytes: &'source [u8],
    position: usize,
    comment_start: Option<usize>,
    remaining_lexemes: u64,
    remaining_steps: u64,
    max_lexeme_bytes: u64,
    finished: bool,
    initial_lexemes: u64,
    initial_steps: u64,
}

impl<'source> LegacyLexer<'source> {
    /// Borrows already-admitted source and sets explicit scan/output budgets.
    ///
    /// No source bytes are copied, normalized, or logged. Zero budgets permit an
    /// empty source but fail when work/output is required. Errors carry byte offsets
    /// and fixed categories; no partial error lexeme is emitted.
    #[must_use]
    pub fn new(source: &SourceText<'source>, budget: LegacyLexBudget) -> Self {
        Self {
            bytes: source.bytes(),
            position: 0,
            comment_start: None,
            remaining_lexemes: budget.max_lexemes,
            remaining_steps: budget.max_steps,
            max_lexeme_bytes: budget.max_lexeme_bytes,
            finished: false,
            initial_lexemes: budget.max_lexemes,
            initial_steps: budget.max_steps,
        }
    }

    /// Returns logical work/output accounting, including unsuccessful admitted work.
    #[must_use]
    pub fn usage(&self) -> LegacyLexUsage {
        LegacyLexUsage {
            attempted_lexemes: self.initial_lexemes - self.remaining_lexemes,
            steps: self.initial_steps - self.remaining_steps,
            consumed_bytes: self.position as u64,
        }
    }

    fn read(&mut self) -> Result<LegacyLexeme<'source>, LegacyLexError> {
        self.remaining_lexemes = self
            .remaining_lexemes
            .checked_sub(1)
            .ok_or(LegacyLexError {
                kind: LegacyLexErrorKind::BudgetExceeded(LegacyLexResource::Lexemes),
                offset: self.position as u64,
            })?;
        let mode = if self.comment_start.is_some() {
            ScanMode::BlockComment
        } else {
            ScanMode::Code
        };
        let (rule, length) = legacy_scanner::first(
            &self.bytes[self.position..],
            mode,
            self.max_lexeme_bytes,
            &mut self.remaining_steps,
        )
        .map_err(|mut error| {
            error.offset += self.position as u64;
            error
        })?;
        let end = self.position.checked_add(length).ok_or(LegacyLexError {
            kind: LegacyLexErrorKind::InvalidTable,
            offset: self.position as u64,
        })?;
        let bytes = self.bytes.get(self.position..end).ok_or(LegacyLexError {
            kind: LegacyLexErrorKind::InvalidTable,
            offset: self.position as u64,
        })?;
        let lexeme = LegacyLexeme::new(rule, self.position as u64, bytes).map_err(|kind| {
            LegacyLexError {
                kind,
                offset: self.position as u64,
            }
        })?;
        let mode = legacy_tables::RULE_MODES
            .get(usize::from(rule))
            .copied()
            .ok_or(LegacyLexError {
                kind: LegacyLexErrorKind::InvalidTable,
                offset: self.position as u64,
            })?;
        match usize::from(mode) {
            0 => {}
            legacy_tables::COMMENT_STATE => {
                self.comment_start.get_or_insert(self.position);
            }
            legacy_tables::NORMAL_STATE => self.comment_start = None,
            _ => {
                return Err(LegacyLexError {
                    kind: LegacyLexErrorKind::InvalidTable,
                    offset: self.position as u64,
                });
            }
        }
        self.position = end;
        Ok(lexeme)
    }
}

impl<'source> Iterator for LegacyLexer<'source> {
    type Item = Result<LegacyLexeme<'source>, LegacyLexError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        if self.position == self.bytes.len() {
            self.finished = true;
            return self.comment_start.map(|offset| {
                Err(LegacyLexError {
                    kind: LegacyLexErrorKind::UnterminatedComment,
                    offset: offset as u64,
                })
            });
        }
        let result = self.read();
        if result.is_err() {
            self.finished = true;
        }
        Some(result)
    }
}

impl FusedIterator for LegacyLexer<'_> {}
