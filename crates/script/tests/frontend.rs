// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source/frontend compatibility suite; leaf files retain focused tests and helpers.

mod expression_list_support;
mod expression_support;
mod scalar_compile_support;
mod statement_support;

#[path = "local_compound_source.rs"]
mod local_compound_source;
#[path = "local_source_budgets.rs"]
mod local_source_budgets;
#[path = "local_source_execution.rs"]
mod local_source_execution;
#[path = "local_source_rejections.rs"]
mod local_source_rejections;
#[path = "local_source_stress.rs"]
mod local_source_stress;
#[path = "scalar_budgets.rs"]
mod scalar_budgets;
#[path = "scalar_compilation.rs"]
mod scalar_compilation;
#[path = "scalar_kernel_limits.rs"]
mod scalar_kernel_limits;
#[path = "scalar_operators.rs"]
mod scalar_operators;
#[path = "scalar_rejections.rs"]
mod scalar_rejections;
#[path = "scalar_stress.rs"]
mod scalar_stress;
#[path = "statement_basic.rs"]
mod statement_basic;
#[path = "statement_control.rs"]
mod statement_control;
#[path = "statement_events.rs"]
mod statement_events;
#[path = "statement_list_budgets.rs"]
mod statement_list_budgets;
#[path = "statement_lists.rs"]
mod statement_lists;
#[path = "statement_views.rs"]
mod statement_views;

#[path = "call_views.rs"]
mod call_views;
#[path = "declaration_budgets.rs"]
mod declaration_budgets;
#[path = "declaration_views.rs"]
mod declaration_views;
#[path = "expression_list_budgets.rs"]
mod expression_list_budgets;
#[path = "expression_lists.rs"]
mod expression_lists;
#[path = "expression_operators.rs"]
mod expression_operators;
#[path = "expression_structure.rs"]
mod expression_structure;
#[path = "function_names.rs"]
mod function_names;
#[path = "identifier_identity.rs"]
mod identifier_identity;
#[path = "identifier_vectors.rs"]
mod identifier_vectors;
#[path = "lexical_boundaries.rs"]
mod lexical_boundaries;
#[path = "lexical_stress.rs"]
mod lexical_stress;
#[path = "lexical_vectors.rs"]
mod lexical_vectors;
#[path = "literal_boundaries.rs"]
mod literal_boundaries;
#[path = "literal_vectors.rs"]
mod literal_vectors;
#[path = "primary_forms.rs"]
mod primary_forms;
#[path = "primary_literals.rs"]
mod primary_literals;
#[path = "reference_views.rs"]
mod reference_views;
#[path = "source_boundaries.rs"]
mod source_boundaries;
#[path = "source_locations.rs"]
mod source_locations;
#[path = "symbol_budgets.rs"]
mod symbol_budgets;
#[path = "symbol_semantics.rs"]
mod symbol_semantics;
#[path = "syntax_budgets.rs"]
mod syntax_budgets;
#[path = "syntax_structure.rs"]
mod syntax_structure;
#[path = "syntax_vectors.rs"]
mod syntax_vectors;
