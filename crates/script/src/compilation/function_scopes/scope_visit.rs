// SPDX-License-Identifier: AGPL-3.0-or-later
use super::scope_tasks::{ScopeFlow, ScopeForSource, ScopePlanner, ScopeTaskKind};
use super::{ScopeAbort, ScopeId, ScopeLocal, invalid};
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_for_clause;
use crate::compilation::function_local_bindings::FunctionLocalDeclaration;
use crate::compilation::function_local_names;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind, function_compile_error};
use crate::expression::{LegacyReferenceKind, LegacyReferenceSyntax};
use crate::statement::{LegacyStatementKind, LegacyStatementSyntax};
use crate::syntax::LegacySyntaxNode;

impl<'tree, 'source> ScopePlanner<'tree, 'source> {
    pub(super) fn visit(
        &mut self,
        builder: &mut FunctionBuilder<'tree, 'source>,
        syntax: LegacySyntaxNode<'tree, 'source>,
        block: ScopeId,
        flow: ScopeFlow,
    ) -> Result<(), FunctionCompileError> {
        if self.plan.block(block, syntax.span())?.abort != ScopeAbort::Open {
            return Ok(());
        }
        let view = LegacyStatementSyntax::from_node(syntax).map_err(|_| invalid(syntax.span()))?;
        match view.kind() {
            LegacyStatementKind::Assignment { target, .. }
            | LegacyStatementKind::CompoundAssignment { target, .. }
            | LegacyStatementKind::Update { target, .. } => {
                let reference =
                    LegacyReferenceSyntax::from_node(target).map_err(|_| invalid(target.span()))?;
                if let LegacyReferenceKind::Name { name } = reference.kind() {
                    let (_, index) = function_local_names::declare(
                        builder,
                        name,
                        FunctionLocalDeclaration::Assignment,
                    )?;
                    self.plan
                        .ensure_name(builder, block, ScopeLocal(index), name.span())?;
                }
                Ok(())
            }
            LegacyStatementKind::Block { body } => self.queue_body(builder, body, block, flow),
            LegacyStatementKind::DeveloperBlock { body } => {
                self.begin_developer(builder, block, body, flow, view.statement_syntax().span())
            }
            LegacyStatementKind::Return { .. } => {
                self.plan.block_mut(block, syntax.span())?.abort = ScopeAbort::Return;
                Ok(())
            }
            LegacyStatementKind::Break => {
                self.record_transfer(builder, block, flow.breaking, false, syntax.span())
            }
            LegacyStatementKind::Continue => {
                self.record_transfer(builder, block, flow.continuation, true, syntax.span())
            }
            LegacyStatementKind::If {
                consequence,
                alternative,
                ..
            } => self.begin_if(
                builder,
                block,
                consequence,
                alternative,
                flow,
                view.statement_syntax().span(),
            ),
            LegacyStatementKind::While { condition, body } => self.begin_while(
                builder,
                block,
                condition,
                body,
                view.statement_syntax().span(),
            ),
            LegacyStatementKind::For {
                initializer,
                condition,
                update,
                body,
            } => {
                function_for_clause::validate(initializer)?;
                let condition = function_for_clause::condition(condition)?;
                let update = function_for_clause::update(update)?;
                self.push(
                    builder,
                    ScopeTaskKind::ForAfterInitializer {
                        parent: block,
                        source: ScopeForSource {
                            condition,
                            update,
                            body,
                        },
                    },
                    view.statement_syntax().span(),
                )?;
                self.visit_task(builder, initializer, block, flow)
            }
            LegacyStatementKind::Switch { body, .. } => {
                self.begin_switch(builder, block, body, flow, view.statement_syntax().span())
            }
            LegacyStatementKind::Case { .. } | LegacyStatementKind::Default => Err(
                function_compile_error::at(view.span(), FunctionCompileErrorKind::IllegalCase),
            ),
            LegacyStatementKind::Empty | LegacyStatementKind::Call { .. } => Ok(()),
            _ => Err(function_compile_error::at(
                view.span(),
                FunctionCompileErrorKind::UnsupportedBody,
            )),
        }
    }
}
