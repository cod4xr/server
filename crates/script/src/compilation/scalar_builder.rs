// SPDX-License-Identifier: AGPL-3.0-or-later
use std::mem;
use std::ops::Range;

use crate::compilation::scalar_compilation_ledger::ScalarCompilationLedger;
use crate::compilation::scalar_compile_error;
use crate::compilation::scalar_task::{
    PendingScalarInstruction, PendingScalarKind, ScalarPatchSite, ScalarTask,
};
use crate::compilation::{ScalarCompilationBudget, ScalarCompileError, ScalarCompileErrorKind};
use crate::execution::{KernelBranchCondition, KernelInstruction};

pub(super) struct ScalarBuilder<'tree, 'source> {
    pub(super) ledger: ScalarCompilationLedger,
    tasks: Vec<ScalarTask<'tree, 'source>>,
    instructions: Vec<PendingScalarInstruction>,
    active_task: bool,
}

impl<'tree, 'source> ScalarBuilder<'tree, 'source> {
    pub(super) fn new(budget: ScalarCompilationBudget) -> Self {
        Self {
            ledger: ScalarCompilationLedger::new(budget),
            tasks: Vec::new(),
            instructions: Vec::new(),
            active_task: false,
        }
    }

    pub(super) fn push_task(
        &mut self,
        task: ScalarTask<'tree, 'source>,
        span: Range<u64>,
    ) -> Result<(), ScalarCompileError> {
        let count = self.tasks.len().checked_add(1).ok_or_else(|| {
            scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::SizeOverflow)
        })?;
        let live = count
            .checked_add(usize::from(self.active_task))
            .ok_or_else(|| {
                scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::SizeOverflow)
            })?;
        self.ledger
            .admit(self.instructions.len(), live, span.clone())?;
        self.tasks.try_reserve(1).map_err(|_| {
            scalar_compile_error::at(span, ScalarCompileErrorKind::AllocationFailed)
        })?;
        self.tasks.push(task);
        Ok(())
    }

    pub(super) fn next_task(&mut self) -> Option<ScalarTask<'tree, 'source>> {
        let task = self.tasks.pop();
        self.active_task = task.is_some();
        task
    }

    pub(super) fn emit(
        &mut self,
        instruction: KernelInstruction,
        span: Range<u64>,
    ) -> Result<(), ScalarCompileError> {
        self.append(PendingScalarKind::Ready(instruction), span)
            .map(|_| ())
    }

    pub(super) fn branch(
        &mut self,
        condition: KernelBranchCondition,
        span: Range<u64>,
    ) -> Result<ScalarPatchSite, ScalarCompileError> {
        self.append(
            PendingScalarKind::Branch {
                condition,
                target: None,
            },
            span,
        )
        .map(ScalarPatchSite)
    }

    fn append(
        &mut self,
        kind: PendingScalarKind,
        span: Range<u64>,
    ) -> Result<usize, ScalarCompileError> {
        let index = self.instructions.len();
        let count = index.checked_add(1).ok_or_else(|| {
            scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::SizeOverflow)
        })?;
        let live = self
            .tasks
            .len()
            .checked_add(usize::from(self.active_task))
            .ok_or_else(|| {
                scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::SizeOverflow)
            })?;
        self.ledger.admit(count, live, span.clone())?;
        self.instructions.try_reserve(1).map_err(|_| {
            scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::AllocationFailed)
        })?;
        self.instructions
            .push(PendingScalarInstruction { kind, span });
        Ok(index)
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
            PendingScalarKind::Branch { target: slot, .. } if slot.is_none() && target > site.0 => {
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
                PendingScalarKind::Branch { target: None, .. } => {
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
