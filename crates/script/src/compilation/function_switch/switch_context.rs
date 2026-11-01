// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::function_initialization::FunctionInitializationSnapshot;
use crate::compilation::function_loop_transfer::FunctionBreakOwner;
use crate::compilation::function_task::FunctionPatchSite;

pub(super) struct SwitchCase {
    pub(super) key: i32,
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
}

#[derive(Clone, Copy)]
pub(in crate::compilation) enum SwitchLabel {
    Case(usize),
    Default,
}
