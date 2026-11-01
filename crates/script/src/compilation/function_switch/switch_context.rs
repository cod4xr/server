// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::function_initialization::FunctionInitializationSnapshot;
use crate::compilation::function_loop_transfer::FunctionBreakOwner;
use crate::compilation::function_scopes::ScopeId;
use crate::compilation::function_task::FunctionPatchSite;
use crate::execution::KernelSwitchKey;

pub(super) struct SwitchCase {
    pub(super) key: KernelSwitchKey,
    pub(super) target: Option<usize>,
    pub(super) span: Range<u64>,
}

pub(in crate::compilation) struct FunctionSwitchContext {
    pub(super) head: usize,
    pub(super) cases: Vec<SwitchCase>,
    pub(super) case_bytes: u64,
    pub(super) default_span: Option<Range<u64>>,
    pub(super) default_target: Option<usize>,
    pub(super) incoming: Option<FunctionInitializationSnapshot>,
    pub(super) exit_facts: Option<FunctionInitializationSnapshot>,
    pub(super) breaks: Option<FunctionPatchSite>,
    pub(super) previous_break_owner: Option<FunctionBreakOwner>,
    pub(super) previous_break_scope: Option<ScopeId>,
    pub(super) scope_parent: Option<ScopeId>,
}

impl FunctionSwitchContext {
    pub(in crate::compilation) fn begins_before(&self, instruction: usize) -> bool {
        self.head < instruction
    }
}

#[derive(Clone, Copy)]
pub(in crate::compilation) enum SwitchLabel {
    Case(usize),
    Default,
}
