// SPDX-License-Identifier: AGPL-3.0-or-later
use std::mem;
use std::ops::Range;

use crate::compilation::scalar_builder::ScalarBuilder;
use crate::compilation::scalar_compile_error;
use crate::compilation::scalar_task::{PendingScalarKind, ScalarPatchSite};
use crate::compilation::{ScalarCompileError, ScalarCompileErrorKind};
use crate::execution::KernelInstruction;

impl ScalarBuilder<'_, '_> {
    pub(super) fn bind_transfers(
        &mut self,
        mut chain: Option<ScalarPatchSite>,
        span: Range<u64>,
    ) -> Result<(), ScalarCompileError> {
        let target = self.instructions.len();
        while let Some(site) = chain {
            self.ledger.charge(1, span.clone())?;
            if site.0 >= target {
                return Err(scalar_compile_error::at(
                    span,
                    ScalarCompileErrorKind::InvalidBranchBinding,
                ));
            }
            let instruction = self.instructions.get_mut(site.0).ok_or_else(|| {
                scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::InvalidBranchBinding)
            })?;
            match &mut instruction.kind {
                PendingScalarKind::Transfer { target: slot, next }
                    if slot.is_none() && next.as_ref().is_none_or(|next| next.0 < site.0) =>
                {
                    *slot = Some(target);
                    chain = next.take();
                }
                _ => {
                    return Err(scalar_compile_error::at(
                        span,
                        ScalarCompileErrorKind::InvalidBranchBinding,
                    ));
                }
            }
        }
        Ok(())
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "Binding consumes the private relocation token so its caller cannot reuse it."
    )]
    pub(super) fn bind(
        &mut self,
        site: ScalarPatchSite,
        span: Range<u64>,
    ) -> Result<(), ScalarCompileError> {
        let target = self.instructions.len();
        let instruction = self.instructions.get_mut(site.0).ok_or_else(|| {
            scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::InvalidBranchBinding)
        })?;
        match &mut instruction.kind {
            PendingScalarKind::Branch { target: slot, .. }
            | PendingScalarKind::Jump { target: slot }
                if slot.is_none() && target > site.0 =>
            {
                *slot = Some(target);
            }
            _ => {
                return Err(scalar_compile_error::at(
                    span,
                    ScalarCompileErrorKind::InvalidBranchBinding,
                ));
            }
        }
        Ok(())
    }

    pub(super) fn resolve(
        &mut self,
        span: Range<u64>,
    ) -> Result<(Vec<KernelInstruction>, Vec<Range<u64>>), ScalarCompileError> {
        self.ledger.resolution(span.clone())?;
        let count = self.instructions.len();
        let mut code = Vec::new();
        let mut spans = Vec::new();
        code.try_reserve_exact(count).map_err(|_| {
            scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::AllocationFailed)
        })?;
        spans.try_reserve_exact(count).map_err(|_| {
            scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::AllocationFailed)
        })?;
        for pending in mem::take(&mut self.instructions) {
            self.ledger.charge(1, pending.span.clone())?;
            let instruction = match pending.kind {
                PendingScalarKind::Ready(instruction) => instruction,
                PendingScalarKind::Branch {
                    condition,
                    target: Some(target),
                } => KernelInstruction::Branch { condition, target },
                PendingScalarKind::Jump {
                    target: Some(target),
                }
                | PendingScalarKind::Transfer {
                    target: Some(target),
                    next: None,
                } => KernelInstruction::Jump(target),
                PendingScalarKind::Branch { target: None, .. }
                | PendingScalarKind::Jump { target: None }
                | PendingScalarKind::Transfer { .. } => {
                    return Err(scalar_compile_error::at(
                        pending.span,
                        ScalarCompileErrorKind::InvalidBranchBinding,
                    ));
                }
            };
            code.push(instruction);
            spans.push(pending.span);
        }
        Ok((code, spans))
    }
}
