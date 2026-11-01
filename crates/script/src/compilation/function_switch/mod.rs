// SPDX-License-Identifier: AGPL-3.0-or-later
mod switch_context;
mod switch_dispatch;
mod switch_error;
mod switch_exit;
mod switch_inventory;
mod switch_labels;
mod switch_nodes;
mod switch_queue;
mod switch_sections;
mod switch_sort;
mod switch_task;

pub(super) use switch_context::FunctionSwitchContext;
pub(super) use switch_sections::record_break;
pub(super) use switch_task::{FunctionSwitchTask, lower};

pub(super) use switch_queue::queue;
