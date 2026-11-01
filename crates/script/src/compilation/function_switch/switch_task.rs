// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_switch::switch_context::{FunctionSwitchContext, SwitchLabel};
use crate::compilation::function_switch::{switch_exit, switch_queue, switch_sections};
use crate::compilation::function_task::FunctionPatchSite;
use crate::syntax::LegacySyntaxNode;

pub(in crate::compilation) enum FunctionSwitchTask<'tree, 'source> {
    Begin {
        body: LegacySyntaxNode<'tree, 'source>,
        span: Range<u64>,
    },
    Label {
        label: SwitchLabel,
        span: Range<u64>,
    },
    End {
        head: usize,
        dispatch: FunctionPatchSite,
        outer: Option<FunctionSwitchContext>,
        span: Range<u64>,
    },
}

impl FunctionSwitchTask<'_, '_> {
    pub(in crate::compilation) fn span(&self) -> Range<u64> {
        match self {
            Self::Begin { span, .. } | Self::Label { span, .. } | Self::End { span, .. } => {
                span.clone()
            }
        }
    }
}

pub(in crate::compilation) fn lower<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    task: FunctionSwitchTask<'tree, 'source>,
) -> Result<(), FunctionCompileError> {
    match task {
        FunctionSwitchTask::Begin { body, span } => switch_queue::begin(builder, body, span),
        FunctionSwitchTask::Label { label, span } => switch_sections::label(builder, label, span),
        FunctionSwitchTask::End {
            head,
            dispatch,
            outer,
            span,
        } => switch_exit::finish(builder, head, dispatch, outer, span),
    }
}
