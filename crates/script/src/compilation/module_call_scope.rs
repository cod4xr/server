// SPDX-License-Identifier: AGPL-3.0-or-later
use std::cmp::Ordering;
use std::ops::Range;

use crate::binding::{CanonicalScriptPath, LegacyScriptPath, ScriptPathBudget};
use crate::compilation::function_compilation_ledger::FunctionCompilationLedger;
use crate::compilation::function_compile_error;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::lexical::LegacyLexeme;

/// Borrowed sorted closed catalog; identifiers grant no host file access.
#[derive(Clone, Copy)]
pub(super) struct ModuleCallScope<'a> {
    pub(super) names: &'a [CanonicalScriptPath],
    pub(super) order: &'a [usize],
    pub(super) path_budget: ScriptPathBudget,
}

impl ModuleCallScope<'_> {
    pub(super) fn resolve(
        self,
        token: LegacyLexeme<'_>,
        ledger: &mut FunctionCompilationLedger,
    ) -> Result<usize, FunctionCompileError> {
        let raw = token.as_bytes();
        let span = token.span();
        let visits = (raw.len() as u64)
            .checked_mul(2)
            .ok_or_else(|| invalid(span.clone(), FunctionCompileErrorKind::SizeOverflow))?;
        // Prepay complete validation/counting before calling the binding helper.
        ledger.identifier(visits, span.clone())?;
        let path = LegacyScriptPath::new(raw, self.path_budget)
            .map_err(|cause| {
                invalid(
                    span.clone(),
                    FunctionCompileErrorKind::ScriptPath(cause.kind),
                )
            })?
            .value;
        let comparison = (raw.len() as u64)
            .checked_add(path.canonical_len())
            .and_then(|bytes| bytes.checked_add(1))
            .ok_or_else(|| invalid(span.clone(), FunctionCompileErrorKind::SizeOverflow))?;
        let (mut low, mut high) = (0, self.order.len());
        while low < high {
            ledger.identifier(comparison, span.clone())?;
            let middle = low + (high - low) / 2;
            let module = self.order[middle];
            let stored = self
                .names
                .get(module)
                .ok_or_else(|| invalid(span.clone(), FunctionCompileErrorKind::InvalidStructure))?;
            match path
                .canonical_bytes()
                .cmp(stored.as_bytes().iter().copied())
            {
                Ordering::Less => high = middle,
                Ordering::Greater => low = middle + 1,
                Ordering::Equal => return Ok(module),
            }
        }
        Err(invalid(span, FunctionCompileErrorKind::UnknownModule))
    }
}

fn invalid(span: Range<u64>, kind: FunctionCompileErrorKind) -> FunctionCompileError {
    function_compile_error::at(span, kind)
}
