// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::syntax::legacy_syntax_error;
use crate::syntax::{LegacySyntaxBudget, LegacySyntaxErrorKind, LegacySyntaxResource};

pub(super) struct SyntaxLedger {
    pub(super) budget: LegacySyntaxBudget,
    pub(super) metadata: u64,
    pub(super) peak_metadata: u64,
    pub(super) peak_stack: u64,
    pub(super) steps: u64,
}

impl SyntaxLedger {
    pub(super) fn new(budget: LegacySyntaxBudget) -> Self {
        Self {
            budget,
            metadata: 0,
            peak_metadata: 0,
            peak_stack: 0,
            steps: 0,
        }
    }

    pub(super) fn admit_metadata(&mut self, bytes: usize) -> Result<(), LegacySyntaxErrorKind> {
        let size = self
            .metadata
            .checked_add(bytes as u64)
            .ok_or(LegacySyntaxErrorKind::SizeOverflow)?;
        legacy_syntax_error::check(
            LegacySyntaxResource::MetadataBytes,
            size,
            self.budget.max_metadata_bytes,
        )?;
        self.metadata = size;
        self.peak_metadata = self.peak_metadata.max(size);
        Ok(())
    }

    pub(super) fn remove_metadata(&mut self, bytes: usize) -> Result<(), LegacySyntaxErrorKind> {
        self.metadata = self
            .metadata
            .checked_sub(bytes as u64)
            .ok_or(LegacySyntaxErrorKind::InvalidTable)?;
        Ok(())
    }

    pub(super) fn step(&mut self) -> Result<(), LegacySyntaxErrorKind> {
        let steps = self
            .steps
            .checked_add(1)
            .ok_or(LegacySyntaxErrorKind::SizeOverflow)?;
        legacy_syntax_error::check(LegacySyntaxResource::Steps, steps, self.budget.max_steps)?;
        self.steps = steps;
        Ok(())
    }
}
