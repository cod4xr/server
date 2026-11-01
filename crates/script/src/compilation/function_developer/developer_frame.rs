// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::binding::LegacyDeveloperMode;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_initialization::{self, FunctionInitializationSnapshot};
use crate::compilation::function_task::FunctionTask;
use crate::compilation::{
    FunctionCompilationUsage, FunctionCompileError, FunctionCompileErrorKind,
    function_compile_error,
};
use crate::compilation::{function_scopes, function_statement_body};
use crate::syntax::LegacySyntaxNode;
use std::ops::Range;

pub(in crate::compilation) struct FunctionDeveloperFrame {
    pub(in crate::compilation) start: usize,
    pub(in crate::compilation) discarded: bool,
    incoming: FunctionInitializationSnapshot,
    before: FunctionCompilationUsage,
    tables: [usize; 3],
}

pub(in crate::compilation) fn begin<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    body: LegacySyntaxNode<'tree, 'source>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    if !builder.mode.bound() {
        return Err(function_compile_error::at(
            span,
            FunctionCompileErrorKind::UnsupportedBody,
        ));
    }
    if builder.active_developer.is_some() || builder.mode.developer_function() {
        return Err(function_compile_error::at(
            span,
            FunctionCompileErrorKind::NestedDeveloperBlock,
        ));
    }
    let incoming = function_initialization::capture(builder, span.clone())?;
    function_scopes::enter_developer(builder, span.clone())?;
    builder.active_developer = Some(FunctionDeveloperFrame {
        start: builder.instructions.len(),
        discarded: builder.mode.developer() == LegacyDeveloperMode::Disabled,
        incoming,
        before: builder.ledger.usage,
        tables: [
            builder.string_literals.len(),
            builder.field_names.len(),
            builder.switch_labels.len(),
        ],
    });
    builder.push_task(FunctionTask::EndDeveloper { span: span.clone() }, span)?;
    function_statement_body::queue(builder, body)
}

pub(in crate::compilation) fn finish(
    builder: &mut FunctionBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    let frame = builder
        .active_developer
        .take()
        .ok_or_else(|| invalid(span.clone()))?;
    function_scopes::leave_developer(builder, frame.discarded, span.clone())?;
    if frame.discarded {
        discard(builder, &frame, span.clone())?;
    }
    function_initialization::restore(builder, frame.incoming, span)
}

fn discard(
    builder: &mut FunctionBuilder<'_, '_>,
    frame: &FunctionDeveloperFrame,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    let lengths = [
        builder.string_literals.len(),
        builder.field_names.len(),
        builder.switch_labels.len(),
    ];
    let mut work = builder
        .instructions
        .len()
        .checked_sub(frame.start)
        .ok_or_else(|| invalid(span.clone()))?;
    for (length, before) in lengths.into_iter().zip(frame.tables) {
        work = work
            .checked_add(
                length
                    .checked_sub(before)
                    .ok_or_else(|| invalid(span.clone()))?,
            )
            .ok_or_else(|| {
                function_compile_error::at(span.clone(), FunctionCompileErrorKind::SizeOverflow)
            })?;
    }
    // Truncation drops flat descriptors. Peaks and work remain charged; only
    // retained code/table counts reset. No reference to this region may publish.
    builder.ledger.charge(work as u64, span.clone())?;
    builder.instructions.truncate(frame.start);
    builder.string_literals.truncate(frame.tables[0]);
    builder.field_names.truncate(frame.tables[1]);
    builder.switch_labels.truncate(frame.tables[2]);
    let usage = &mut builder.ledger.usage;
    usage.instructions = frame.before.instructions;
    usage.string_literals = frame.before.string_literals;
    usage.retained_string_bytes = frame.before.retained_string_bytes;
    usage.field_names = frame.before.field_names;
    usage.retained_field_bytes = frame.before.retained_field_bytes;
    usage.switch_labels = frame.before.switch_labels;
    usage.retained_switch_bytes = frame.before.retained_switch_bytes;
    builder.admit_tables(span)
}

fn invalid(span: Range<u64>) -> FunctionCompileError {
    function_compile_error::at(span, FunctionCompileErrorKind::InvalidControlState)
}
