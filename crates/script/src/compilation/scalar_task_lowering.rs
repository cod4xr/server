// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::ScalarCompileError;
use crate::compilation::scalar_builder::ScalarBuilder;
use crate::compilation::scalar_conditional;
use crate::compilation::scalar_expression_lowering;
use crate::compilation::scalar_for;
use crate::compilation::scalar_local_names;
use crate::compilation::scalar_local_statements;
use crate::compilation::scalar_task::ScalarTask;
use crate::compilation::scalar_while;
use crate::execution::KernelInstruction;

pub(super) fn drain(builder: &mut ScalarBuilder<'_, '_>) -> Result<(), ScalarCompileError> {
    while let Some(task) = builder.next_task() {
        builder.ledger.charge(1, task.span())?;
        match task {
            ScalarTask::Visit(node) => scalar_expression_lowering::visit(builder, node)?,
            ScalarTask::Statement { syntax, last } => {
                scalar_local_statements::lower(builder, syntax, last)?;
            }
            ScalarTask::FinishAssignment { name, slot, span } => {
                builder.emit(KernelInstruction::StoreLocal(slot), span)?;
                scalar_local_names::initialize(builder, name, slot)?;
            }
            ScalarTask::Emit { instruction, span } => builder.emit(instruction, span)?,
            ScalarTask::BeginLogical {
                operator,
                right,
                span,
            } => scalar_expression_lowering::logical(builder, operator, right, span)?,
            ScalarTask::EndLogical { site, span } => builder.bind(site, span)?,
            ScalarTask::BeginIf {
                consequence,
                alternative,
                span,
            } => {
                scalar_conditional::begin(builder, consequence, alternative, span)?;
            }
            ScalarTask::AfterThen {
                false_site,
                incoming,
                alternative,
                span,
            } => {
                scalar_conditional::after_then(builder, false_site, incoming, alternative, span)?;
            }
            ScalarTask::AfterElse {
                end_site,
                then_facts,
                span,
            } => {
                scalar_conditional::after_else(builder, end_site, then_facts, span)?;
            }
            ScalarTask::BeginWhileBody { head, body, span } => {
                scalar_while::begin_body(builder, head, body, span)?;
            }
            ScalarTask::AfterWhileBody {
                head,
                exit_site,
                incoming,
                outer_loop,
                span,
            } => {
                scalar_while::after_body(builder, head, exit_site, incoming, outer_loop, span)?;
            }
            ScalarTask::AfterForInit(plan) => scalar_for::after_init(builder, plan)?,
            ScalarTask::BeginForBody { head, plan } => scalar_for::begin_body(builder, head, plan)?,
            ScalarTask::AfterForBody { end, update } => {
                scalar_for::after_body(builder, end, update)?;
            }
            ScalarTask::AfterForUpdate(end) => scalar_for::after_update(builder, end)?,
        }
    }
    Ok(())
}
