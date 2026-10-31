// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::syntax::legacy_syntax_error;
use crate::syntax::syntax_ledger::SyntaxLedger;
use crate::syntax::{LegacySyntaxErrorKind, LegacySyntaxResource};

pub(super) struct StackEntry {
    pub(super) state: usize,
    pub(super) node: Option<usize>,
}

pub(super) struct SyntaxStack {
    entries: Vec<StackEntry>,
}

impl SyntaxStack {
    pub(super) fn new(ledger: &mut SyntaxLedger) -> Result<Self, LegacySyntaxErrorKind> {
        let mut stack = Self {
            entries: Vec::new(),
        };
        stack.push(0, None, ledger)?;
        Ok(stack)
    }

    pub(super) fn push(
        &mut self,
        state: usize,
        node: Option<usize>,
        ledger: &mut SyntaxLedger,
    ) -> Result<(), LegacySyntaxErrorKind> {
        let count = self
            .entries
            .len()
            .checked_add(1)
            .ok_or(LegacySyntaxErrorKind::SizeOverflow)?;
        legacy_syntax_error::check(
            LegacySyntaxResource::StackEntries,
            count as u64,
            ledger.budget.max_stack_entries,
        )?;
        ledger.admit_metadata(size_of::<StackEntry>())?;
        self.entries
            .try_reserve(1)
            .map_err(|_| LegacySyntaxErrorKind::AllocationFailed)?;
        self.entries.push(StackEntry { state, node });
        ledger.peak_stack = ledger.peak_stack.max(count as u64);
        Ok(())
    }

    pub(super) fn state(&self) -> Result<usize, LegacySyntaxErrorKind> {
        self.entries
            .last()
            .map(|entry| entry.state)
            .ok_or(LegacySyntaxErrorKind::InvalidTable)
    }

    pub(super) fn node(&self) -> Result<usize, LegacySyntaxErrorKind> {
        self.entries
            .last()
            .and_then(|entry| entry.node)
            .ok_or(LegacySyntaxErrorKind::InvalidTable)
    }

    pub(super) fn rhs(&self, count: usize) -> Result<&[StackEntry], LegacySyntaxErrorKind> {
        if count >= self.entries.len() {
            return Err(LegacySyntaxErrorKind::InvalidTable);
        }
        Ok(&self.entries[self.entries.len() - count..])
    }

    pub(super) fn pop(
        &mut self,
        count: usize,
        ledger: &mut SyntaxLedger,
    ) -> Result<(), LegacySyntaxErrorKind> {
        self.rhs(count)?;
        let bytes = count
            .checked_mul(size_of::<StackEntry>())
            .ok_or(LegacySyntaxErrorKind::SizeOverflow)?;
        self.entries.truncate(self.entries.len() - count);
        ledger.remove_metadata(bytes)
    }
}
