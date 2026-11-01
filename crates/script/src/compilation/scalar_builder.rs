// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::scalar_compilation_ledger::ScalarCompilationLedger;
use crate::compilation::scalar_compile_error;
use crate::compilation::scalar_local_bindings::ScalarLocalBindings;
use crate::compilation::scalar_loop_transfer::ScalarWhileContext;
use crate::compilation::scalar_task::{
    PendingScalarInstruction, PendingScalarKind, ScalarPatchSite, ScalarTask,
};
use crate::compilation::{ScalarCompilationBudget, ScalarCompileError, ScalarCompileErrorKind};
use crate::execution::{KernelBranchCondition, KernelInstruction};

pub(super) struct ScalarBuilder<'tree, 'source> {
    pub(super) ledger: ScalarCompilationLedger,
    pub(super) locals: Option<ScalarLocalBindings<'source>>,
    pub(super) reachable: bool,
    pub(super) active_loop: Option<ScalarWhileContext>,
    tasks: Vec<ScalarTask<'tree, 'source>>,
    pub(super) instructions: Vec<PendingScalarInstruction>,
    active_task: bool,
}

impl<'tree, 'source> ScalarBuilder<'tree, 'source> {
    pub(super) fn new(budget: ScalarCompilationBudget) -> Self {
        Self {
            ledger: ScalarCompilationLedger::new(budget),
            locals: None,
            reachable: true,
            active_loop: None,
            tasks: Vec::new(),
            instructions: Vec::new(),
            active_task: false,
        }
    }

    pub(super) fn admit_binding(
        &mut self,
        count: u64,
        bytes: u64,
        span: Range<u64>,
    ) -> Result<(), ScalarCompileError> {
        self.ledger.bindings(count, bytes, span.clone())?;
        let live = self
            .tasks
            .len()
            .checked_add(usize::from(self.active_task))
            .ok_or_else(|| {
                scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::SizeOverflow)
            })?;
        self.ledger.admit(self.instructions.len(), live, span)
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

    pub(super) fn admit_control(
        &mut self,
        bytes: u64,
        span: Range<u64>,
    ) -> Result<(), ScalarCompileError> {
        self.ledger.control(bytes, span.clone())?;
        let live = self
            .tasks
            .len()
            .checked_add(usize::from(self.active_task))
            .ok_or_else(|| {
                scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::SizeOverflow)
            })?;
        self.ledger.admit(self.instructions.len(), live, span)
    }

    pub(super) fn next_task(&mut self) -> Option<ScalarTask<'tree, 'source>> {
        let task = self.tasks.pop();
        self.active_task = task.is_some();
        task
    }

    pub(super) fn instruction_position(&self) -> usize {
        self.instructions.len()
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

    pub(super) fn jump(&mut self, span: Range<u64>) -> Result<ScalarPatchSite, ScalarCompileError> {
        self.append(PendingScalarKind::Jump { target: None }, span)
            .map(ScalarPatchSite)
    }

    pub(super) fn break_jump(
        &mut self,
        next: Option<ScalarPatchSite>,
        span: Range<u64>,
    ) -> Result<ScalarPatchSite, ScalarCompileError> {
        self.append(PendingScalarKind::Break { target: None, next }, span)
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
}
