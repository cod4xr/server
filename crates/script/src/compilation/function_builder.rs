// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::function_compilation_ledger::FunctionCompilationLedger;
use crate::compilation::function_compile_error;
use crate::compilation::function_developer::FunctionDeveloperFrame;
use crate::compilation::function_local_bindings::FunctionLocalBindings;
use crate::compilation::function_loop_transfer::{FunctionBreakOwner, FunctionLoopContext};
use crate::compilation::function_lowering_mode::{FunctionLoweringMode, StandaloneFunctionMode};
use crate::compilation::function_scopes::{FunctionScopePlan, ScopeId};
use crate::compilation::function_switch::FunctionSwitchContext;
use crate::compilation::function_task::{
    FunctionPatchSite, FunctionTask, PendingFunctionInstruction, PendingFunctionKind,
};
use crate::compilation::{
    FunctionCompilationBudget, FunctionCompileError, FunctionCompileErrorKind,
    FunctionLoweringBudget, FunctionSourceProfile,
};
use crate::execution::{KernelBranchCondition, KernelInstruction};

pub(super) struct FunctionBuilder<'tree, 'source> {
    pub(super) ledger: FunctionCompilationLedger,
    pub(super) mode: FunctionLoweringMode<'tree, 'source>,
    pub(super) locals: Option<FunctionLocalBindings<'source>>,
    pub(super) reachable: bool,
    pub(super) active_loop: Option<FunctionLoopContext>,
    pub(super) break_owner: Option<FunctionBreakOwner>,
    pub(super) break_scope: Option<ScopeId>,
    pub(super) active_switch: Option<FunctionSwitchContext>,
    pub(super) active_developer: Option<FunctionDeveloperFrame>,
    pub(super) scopes: Option<FunctionScopePlan>,
    tasks: Vec<FunctionTask<'tree, 'source>>,
    pub(super) instructions: Vec<PendingFunctionInstruction>,
    pub(super) string_literals: Vec<Vec<u8>>,
    pub(super) field_names: Vec<Vec<u8>>,
    pub(super) switch_labels: Vec<Vec<u8>>,
    active_task: bool,
}

impl<'tree, 'source> FunctionBuilder<'tree, 'source> {
    pub(super) fn new(budget: FunctionCompilationBudget, profile: FunctionSourceProfile) -> Self {
        Self::with_mode(
            budget.into(),
            FunctionLoweringMode::Standalone(StandaloneFunctionMode {
                profile,
                verification: budget.verification,
                validation: budget.validation,
            }),
        )
    }

    pub(super) fn with_mode(
        budget: FunctionLoweringBudget,
        mode: FunctionLoweringMode<'tree, 'source>,
    ) -> Self {
        Self {
            ledger: FunctionCompilationLedger::new(budget),
            mode,
            locals: mode.primitive().then(FunctionLocalBindings::new),
            reachable: true,
            active_loop: None,
            break_owner: None,
            break_scope: None,
            active_switch: None,
            active_developer: None,
            scopes: None,
            tasks: Vec::new(),
            instructions: Vec::new(),
            string_literals: Vec::new(),
            field_names: Vec::new(),
            switch_labels: Vec::new(),
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

    pub(super) fn admit_literal(
        &mut self,
        length: u64,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        self.ledger.usage.string_literals = self
            .ledger
            .usage
            .string_literals
            .checked_add(1)
            .ok_or_else(|| {
                function_compile_error::at(span.clone(), FunctionCompileErrorKind::SizeOverflow)
            })?;
        self.ledger.usage.retained_string_bytes = self
            .ledger
            .usage
            .retained_string_bytes
            .checked_add(length)
            .and_then(|bytes| bytes.checked_add(size_of::<Vec<u8>>() as u64))
            .ok_or_else(|| {
                function_compile_error::at(span.clone(), FunctionCompileErrorKind::SizeOverflow)
            })?;
        self.admit_tables(span)
    }

    pub(super) fn admit_tables(&mut self, span: Range<u64>) -> Result<(), FunctionCompileError> {
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

    pub(super) fn append(
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
