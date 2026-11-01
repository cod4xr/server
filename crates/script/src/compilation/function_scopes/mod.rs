// SPDX-License-Identifier: AGPL-3.0-or-later
//! Compiler-private metered physical local scopes and alias cleanup.

mod scope_branches;
mod scope_condition;
mod scope_developer;
mod scope_emission;
mod scope_fold_string;
mod scope_fold_value;
mod scope_for;
mod scope_if_emission;
mod scope_loop_emission;
mod scope_loops;
mod scope_metadata;
mod scope_names;
mod scope_planner;
mod scope_switch_emission;
mod scope_switches;
mod scope_tasks;
mod scope_transfers;
mod scope_unions;
mod scope_visit;

pub(in crate::compilation) use scope_developer::{enter_developer, leave_developer};
pub(in crate::compilation) use scope_emission::{break_edge, continue_edge, current, local_store};
pub(in crate::compilation) use scope_if_emission::{begin_else, begin_if, end_consequence, end_if};
pub(in crate::compilation) use scope_loop_emission::{
    enter_for_update, enter_loop_body, leave_loop, prepare_loop,
};
pub(in crate::compilation) use scope_planner::build;
pub(in crate::compilation) use scope_switch_emission::{enter_switch_label, leave_switch};

use std::collections::HashMap;
use std::ops::Range;

use crate::compilation::function_compile_error;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};

/// Scope identities never become published local operands or runtime heap IDs.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(in crate::compilation) struct ScopeId(usize);

/// A stable function-local operand, distinct from a block's physical name position.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct ScopeLocal(usize);

/// Native ordering determines structural abort propagation, independently of facts.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum ScopeAbort {
    Open,
    Continue,
    Break,
    Return,
}

struct ScopeBlock {
    parent: Option<ScopeId>,
    names: Vec<ScopeLocal>,
    positions: HashMap<ScopeLocal, usize>,
    public: usize,
    created: usize,
    abort: ScopeAbort,
}

#[derive(Clone, Copy)]
pub(in crate::compilation) enum ScopeControl {
    Developer {
        parent: ScopeId,
        body: ScopeId,
    },
    If {
        parent: ScopeId,
        consequence: ScopeId,
        alternative: Option<ScopeId>,
    },
    Loop {
        parent: ScopeId,
        body: ScopeId,
        update: Option<ScopeId>,
        declarations_after: usize,
        exits: bool,
    },
    Switch {
        parent: ScopeId,
        declarations_after: usize,
        has_default: bool,
        join: Option<ScopeId>,
    },
}

pub(in crate::compilation) struct FunctionScopePlan {
    blocks: Vec<ScopeBlock>,
    controls: HashMap<u64, ScopeControl>,
    labels: HashMap<u64, ScopeId>,
    current: ScopeId,
    retained_bytes: u64,
}

impl FunctionScopePlan {
    fn empty() -> Self {
        Self {
            blocks: Vec::new(),
            controls: HashMap::new(),
            labels: HashMap::new(),
            current: ScopeId(0),
            retained_bytes: 0,
        }
    }

    fn block(&self, id: ScopeId, span: Range<u64>) -> Result<&ScopeBlock, FunctionCompileError> {
        self.blocks.get(id.0).ok_or_else(|| invalid(span))
    }

    fn block_mut(
        &mut self,
        id: ScopeId,
        span: Range<u64>,
    ) -> Result<&mut ScopeBlock, FunctionCompileError> {
        self.blocks.get_mut(id.0).ok_or_else(|| invalid(span))
    }
}

fn invalid(span: Range<u64>) -> FunctionCompileError {
    function_compile_error::at(span, FunctionCompileErrorKind::InvalidControlState)
}

fn overflow(span: Range<u64>) -> FunctionCompileError {
    function_compile_error::at(span, FunctionCompileErrorKind::SizeOverflow)
}

fn allocation(span: Range<u64>) -> FunctionCompileError {
    function_compile_error::at(span, FunctionCompileErrorKind::AllocationFailed)
}
