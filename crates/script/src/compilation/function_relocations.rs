// SPDX-License-Identifier: AGPL-3.0-or-later
use std::mem;
use std::ops::Range;

use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_task::{FunctionPatchSite, PendingFunctionKind};
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::KernelInstruction;

impl FunctionBuilder<'_, '_> {
    pub(super) fn bind_transfers(
        &mut self,
        mut chain: Option<FunctionPatchSite>,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let target = self.instructions.len();
        while let Some(site) = chain {
            self.ledger.charge(1, span.clone())?;
            if site.0 >= target {
                return Err(function_compile_error::at(
                    span,
                    FunctionCompileErrorKind::InvalidBranchBinding,
                ));
            }
            let instruction = self.instructions.get_mut(site.0).ok_or_else(|| {
                function_compile_error::at(
                    span.clone(),
                    FunctionCompileErrorKind::InvalidBranchBinding,
                )
            })?;
            match &mut instruction.kind {
                PendingFunctionKind::Transfer { target: slot, next }
                    if slot.is_none() && next.as_ref().is_none_or(|next| next.0 < site.0) =>
                {
                    *slot = Some(target);
                    chain = next.take();
                }
                _ => {
                    return Err(function_compile_error::at(
                        span,
                        FunctionCompileErrorKind::InvalidBranchBinding,
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
        site: FunctionPatchSite,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let target = self.instructions.len();
        let instruction = self.instructions.get_mut(site.0).ok_or_else(|| {
            function_compile_error::at(span.clone(), FunctionCompileErrorKind::InvalidBranchBinding)
        })?;
        match &mut instruction.kind {
            PendingFunctionKind::Branch { target: slot, .. }
            | PendingFunctionKind::Jump { target: slot }
                if slot.is_none() && target > site.0 =>
            {
                *slot = Some(target);
            }
            _ => {
                return Err(function_compile_error::at(
                    span,
                    FunctionCompileErrorKind::InvalidBranchBinding,
                ));
            }
        }
        Ok(())
    }

    pub(super) fn resolve(
        &mut self,
        span: Range<u64>,
    ) -> Result<(Vec<KernelInstruction>, Vec<Range<u64>>), FunctionCompileError> {
        self.ledger.resolution(span.clone())?;
        let count = self.instructions.len();
        let mut code = Vec::new();
        let mut spans = Vec::new();
        code.try_reserve_exact(count).map_err(|_| {
            function_compile_error::at(span.clone(), FunctionCompileErrorKind::AllocationFailed)
        })?;
        spans.try_reserve_exact(count).map_err(|_| {
            function_compile_error::at(span.clone(), FunctionCompileErrorKind::AllocationFailed)
        })?;
        for pending in mem::take(&mut self.instructions) {
            self.ledger.charge(1, pending.span.clone())?;
            let instruction = match pending.kind {
                PendingFunctionKind::Ready(instruction) => instruction,
                PendingFunctionKind::Branch {
                    condition,
                    target: Some(target),
                } => KernelInstruction::Branch { condition, target },
                PendingFunctionKind::Jump {
                    target: Some(target),
                }
                | PendingFunctionKind::Transfer {
                    target: Some(target),
                    next: None,
                } => KernelInstruction::Jump(target),
                PendingFunctionKind::Branch { target: None, .. }
                | PendingFunctionKind::Jump { target: None }
                | PendingFunctionKind::Transfer { .. } => {
                    return Err(function_compile_error::at(
                        pending.span,
                        FunctionCompileErrorKind::InvalidBranchBinding,
                    ));
                }
            };
            code.push(instruction);
            spans.push(pending.span);
        }
        Ok((code, spans))
    }
}
