// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::function_compilation_ledger::FunctionCompilationLedger;
use crate::compilation::function_compile_error;
use crate::compilation::function_local_bindings::FunctionLocalBindings;
use crate::compilation::function_loop_transfer::FunctionLoopContext;
use crate::compilation::function_task::{
    FunctionPatchSite, FunctionTask, PendingFunctionInstruction, PendingFunctionKind,
};
use crate::compilation::{
    FunctionCompilationBudget, FunctionCompileError, FunctionCompileErrorKind,
    FunctionSourceProfile,
};
use crate::execution::{KernelBranchCondition, KernelInstruction};

pub(super) struct FunctionBuilder<'tree, 'source> {
    pub(super) ledger: FunctionCompilationLedger,
    pub(super) profile: FunctionSourceProfile,
    pub(super) locals: Option<FunctionLocalBindings<'source>>,
    pub(super) reachable: bool,
    pub(super) active_loop: Option<FunctionLoopContext>,
    tasks: Vec<FunctionTask<'tree, 'source>>,
    pub(super) instructions: Vec<PendingFunctionInstruction>,
    active_task: bool,
}

impl<'tree, 'source> FunctionBuilder<'tree, 'source> {
    pub(super) fn new(budget: FunctionCompilationBudget, profile: FunctionSourceProfile) -> Self {
        Self {
            ledger: FunctionCompilationLedger::new(budget),
            profile,
            locals: match profile {
                FunctionSourceProfile::PureScalarReturn => None,
                FunctionSourceProfile::PrimitiveStatements => Some(FunctionLocalBindings::new()),
            },
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
    ) -> Result<(), FunctionCompileError> {
        self.ledger.bindings(count, bytes, span.clone())?;
        let live = self
            .tasks
            .len()
            .checked_add(usize::from(self.active_task))
            .ok_or_else(|| {
                function_compile_error::at(span.clone(), FunctionCompileErrorKind::SizeOverflow)
            })?;
        self.ledger.admit(self.instructions.len(), live, span)
    }

    pub(super) fn push_task(
        &mut self,
        task: FunctionTask<'tree, 'source>,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let count = self.tasks.len().checked_add(1).ok_or_else(|| {
            function_compile_error::at(span.clone(), FunctionCompileErrorKind::SizeOverflow)
        })?;
        let live = count
            .checked_add(usize::from(self.active_task))
            .ok_or_else(|| {
                function_compile_error::at(span.clone(), FunctionCompileErrorKind::SizeOverflow)
            })?;
        self.ledger
            .admit(self.instructions.len(), live, span.clone())?;
        self.tasks.try_reserve(1).map_err(|_| {
            function_compile_error::at(span, FunctionCompileErrorKind::AllocationFailed)
        })?;
        self.tasks.push(task);
        Ok(())
    }

    pub(super) fn admit_control(
        &mut self,
        bytes: u64,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        self.ledger.control(bytes, span.clone())?;
        let live = self
            .tasks
            .len()
            .checked_add(usize::from(self.active_task))
            .ok_or_else(|| {
                function_compile_error::at(span.clone(), FunctionCompileErrorKind::SizeOverflow)
            })?;
        self.ledger.admit(self.instructions.len(), live, span)
    }

    pub(super) fn next_task(&mut self) -> Option<FunctionTask<'tree, 'source>> {
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
    ) -> Result<(), FunctionCompileError> {
        self.append(PendingFunctionKind::Ready(instruction), span)
            .map(|_| ())
    }

    pub(super) fn branch(
        &mut self,
        condition: KernelBranchCondition,
        span: Range<u64>,
    ) -> Result<FunctionPatchSite, FunctionCompileError> {
        self.append(
            PendingFunctionKind::Branch {
                condition,
                target: None,
            },
            span,
        )
        .map(FunctionPatchSite)
    }

    pub(super) fn jump(
        &mut self,
        span: Range<u64>,
    ) -> Result<FunctionPatchSite, FunctionCompileError> {
        self.append(PendingFunctionKind::Jump { target: None }, span)
            .map(FunctionPatchSite)
    }

    pub(super) fn transfer_jump(
        &mut self,
        next: Option<FunctionPatchSite>,
        span: Range<u64>,
    ) -> Result<FunctionPatchSite, FunctionCompileError> {
        self.append(PendingFunctionKind::Transfer { target: None, next }, span)
            .map(FunctionPatchSite)
    }

    fn append(
        &mut self,
        kind: PendingFunctionKind,
        span: Range<u64>,
    ) -> Result<usize, FunctionCompileError> {
        let index = self.instructions.len();
        let count = index.checked_add(1).ok_or_else(|| {
            function_compile_error::at(span.clone(), FunctionCompileErrorKind::SizeOverflow)
        })?;
        let live = self
            .tasks
            .len()
            .checked_add(usize::from(self.active_task))
            .ok_or_else(|| {
                function_compile_error::at(span.clone(), FunctionCompileErrorKind::SizeOverflow)
            })?;
        self.ledger.admit(count, live, span.clone())?;
        self.instructions.try_reserve(1).map_err(|_| {
            function_compile_error::at(span.clone(), FunctionCompileErrorKind::AllocationFailed)
        })?;
        self.instructions
            .push(PendingFunctionInstruction { kind, span });
        Ok(index)
    }
}
