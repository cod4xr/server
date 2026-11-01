// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_call_arguments;
use crate::compilation::function_conditional;
use crate::compilation::function_expression_lowering;
use crate::compilation::function_for;
use crate::compilation::function_local_names;
use crate::compilation::function_local_statements;
use crate::compilation::function_scopes;
use crate::compilation::function_switch;
use crate::compilation::function_task::FunctionTask;
use crate::compilation::function_while;
use crate::execution::KernelInstruction;

pub(super) fn drain(builder: &mut FunctionBuilder<'_, '_>) -> Result<(), FunctionCompileError> {
    while let Some(task) = builder.next_task() {
        builder.ledger.charge(1, task.span())?;
        match task {
            FunctionTask::EndDeveloper { span } => {
                crate::compilation::function_developer::finish(builder, span)?;
            }
            FunctionTask::ValidationOnly { span } => {
                crate::compilation::function_developer::emit_validation(builder, span)?;
            }
            FunctionTask::Switch(task) => function_switch::lower(builder, task)?,
            FunctionTask::Visit(node) => function_expression_lowering::visit(builder, node)?,
            FunctionTask::CallArguments { cursor, span } => {
                function_call_arguments::next(builder, cursor, span)?;
            }
            FunctionTask::Statement { syntax, last } => {
                function_local_statements::lower(builder, syntax, last)?;
            }
            FunctionTask::FinishAssignment { name, slot, span } => {
                function_scopes::local_store(builder, slot, span.clone())?;
                builder.emit(KernelInstruction::StoreLocal(slot), span)?;
                function_local_names::initialize(builder, name, slot)?;
            }
            FunctionTask::Emit { instruction, span } => builder.emit(instruction, span)?,
            FunctionTask::BeginLogical {
                operator,
                right,
                span,
            } => function_expression_lowering::logical(builder, operator, right, span)?,
            FunctionTask::EndLogical { site, span } => builder.bind(site, span)?,
            FunctionTask::BeginIf {
                consequence,
                alternative,
                span,
            } => {
                function_conditional::begin(builder, consequence, alternative, span)?;
            }
            FunctionTask::AfterThen {
                false_site,
                incoming,
                alternative,
                span,
            } => {
                function_conditional::after_then(builder, false_site, incoming, alternative, span)?;
            }
            FunctionTask::AfterElse {
                end_site,
                then_facts,
                span,
            } => {
                function_conditional::after_else(builder, end_site, then_facts, span)?;
            }
            FunctionTask::BeginWhileBody { head, body, span } => {
                function_while::begin_body(builder, head, body, span)?;
            }
            FunctionTask::AfterWhileBody {
                head,
                exit_site,
                incoming,
                outer_loop,
                span,
            } => {
                function_while::after_body(builder, head, exit_site, incoming, outer_loop, span)?;
            }
            FunctionTask::AfterForInit(plan) => function_for::after_init(builder, plan)?,
            FunctionTask::BeginForBody { head, plan } => {
                function_for::begin_body(builder, head, plan)?;
            }
            FunctionTask::AfterForBody { end, update } => {
                function_for::after_body(builder, end, update)?;
            }
            FunctionTask::AfterForUpdate(end) => function_for::after_update(builder, end)?,
        }
    }
    Ok(())
}
