// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source/frontend compatibility suite; leaf files retain focused tests and helpers.
#[path = "package_calls.rs"]
mod package_calls;
mod package_compile_support;
#[path = "package_heap.rs"]
mod package_heap;
#[path = "package_includes_calls.rs"]
mod package_includes_calls;
#[path = "package_includes_heap.rs"]
mod package_includes_heap;
#[path = "package_includes_limits.rs"]
mod package_includes_limits;
#[path = "package_includes_visibility.rs"]
mod package_includes_visibility;
#[path = "package_includes_width.rs"]
mod package_includes_width;
#[path = "package_limits.rs"]
mod package_limits;
#[path = "package_names.rs"]
mod package_names;
#[path = "package_root_context.rs"]
mod package_root_context;
#[path = "package_visibility.rs"]
mod package_visibility;
#[path = "package_width_depth.rs"]
mod package_width_depth;
#[path = "script_path_budgets.rs"]
mod script_path_budgets;
#[path = "script_path_model.rs"]
mod script_path_model;
#[path = "script_path_names.rs"]
mod script_path_names;
#[path = "script_path_vectors.rs"]
mod script_path_vectors;
mod script_root_support;
#[path = "source_root_context.rs"]
mod source_root_context;
#[path = "source_root_depth.rs"]
mod source_root_depth;
#[path = "source_root_execution.rs"]
mod source_root_execution;
#[path = "source_root_hostile.rs"]
mod source_root_hostile;
#[path = "source_root_limits.rs"]
mod source_root_limits;
#[path = "source_root_pressure.rs"]
mod source_root_pressure;
#[path = "source_root_pressure_limits.rs"]
mod source_root_pressure_limits;
#[path = "source_root_width.rs"]
mod source_root_width;

#[path = "bound_object_arguments.rs"]
mod bound_object_arguments;
mod expression_list_support;
mod expression_support;
mod function_compile_support;
#[path = "source_fields.rs"]
mod source_fields;
#[path = "source_struct_construction.rs"]
mod source_struct_construction;
#[path = "source_struct_queries.rs"]
mod source_struct_queries;
mod statement_support;

#[path = "local_vector_index_limits.rs"]
mod local_vector_index_limits;

#[path = "local_vector_index.rs"]
mod local_vector_index;

#[path = "local_vector_source_limits.rs"]
mod local_vector_source_limits;
#[path = "local_vector_source_stress.rs"]
mod local_vector_source_stress;

#[path = "local_vector_source_errors.rs"]
mod local_vector_source_errors;
#[path = "local_vector_source_execution.rs"]
mod local_vector_source_execution;

#[path = "local_vector_arguments.rs"]
mod local_vector_arguments;

#[path = "local_for_budgets.rs"]
mod local_for_budgets;
#[path = "local_for_errors.rs"]
mod local_for_errors;
#[path = "local_for_stress.rs"]
mod local_for_stress;

#[path = "local_for_execution.rs"]
mod local_for_execution;
#[path = "local_for_initialization.rs"]
mod local_for_initialization;

#[path = "local_transfer_budgets.rs"]
mod local_transfer_budgets;
#[path = "local_transfer_stress.rs"]
mod local_transfer_stress;

#[path = "local_transfer_execution.rs"]
mod local_transfer_execution;
#[path = "local_transfer_rejections.rs"]
mod local_transfer_rejections;

#[path = "local_while_budgets.rs"]
mod local_while_budgets;
#[path = "local_while_stress.rs"]
mod local_while_stress;

#[path = "local_while_execution.rs"]
mod local_while_execution;
#[path = "local_while_initialization.rs"]
mod local_while_initialization;

#[path = "local_conditional_budgets.rs"]
mod local_conditional_budgets;
#[path = "local_conditional_stress.rs"]
mod local_conditional_stress;

#[path = "local_conditional_execution.rs"]
mod local_conditional_execution;
#[path = "local_conditional_initialization.rs"]
mod local_conditional_initialization;

#[path = "local_compound_source.rs"]
mod local_compound_source;
#[path = "local_postfix_source.rs"]
mod local_postfix_source;
#[path = "local_script_arguments.rs"]
mod local_script_arguments;
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

#[path = "bound_module_depth.rs"]
mod bound_module_depth;
#[path = "bound_module_execution.rs"]
mod bound_module_execution;
#[path = "bound_module_heap.rs"]
mod bound_module_heap;
#[path = "bound_module_limits.rs"]
mod bound_module_limits;
#[path = "bound_module_names.rs"]
mod bound_module_names;
#[path = "bound_module_order.rs"]
mod bound_module_order;
#[path = "bound_module_references.rs"]
mod bound_module_references;
#[path = "bound_module_visibility.rs"]
mod bound_module_visibility;
#[path = "bound_module_width.rs"]
mod bound_module_width;
#[path = "bound_reference_contracts.rs"]
mod bound_reference_contracts;
#[path = "builtin_defined_source.rs"]
mod builtin_defined_source;
#[path = "builtin_source.rs"]
mod builtin_source;
#[path = "builtin_source_contracts.rs"]
mod builtin_source_contracts;
#[path = "builtin_string_arguments.rs"]
mod builtin_string_arguments;
#[path = "builtin_string_source.rs"]
mod builtin_string_source;
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
#[path = "function_switch_budgets.rs"]
mod function_switch_budgets;
#[path = "function_switch_execution.rs"]
mod function_switch_execution;
#[path = "function_switch_initialization.rs"]
mod function_switch_initialization;
#[path = "function_switch_nesting.rs"]
mod function_switch_nesting;
#[path = "function_switch_rejections.rs"]
mod function_switch_rejections;
#[path = "function_switch_stress.rs"]
mod function_switch_stress;
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
#[path = "localized_source_pressure.rs"]
mod localized_source_pressure;
mod module_compile_support;
#[path = "primary_forms.rs"]
mod primary_forms;
#[path = "primary_literals.rs"]
mod primary_literals;
#[path = "reference_views.rs"]
mod reference_views;
#[path = "source_array_gate.rs"]
mod source_array_gate;
#[path = "source_array_transport_calls.rs"]
mod source_array_transport_calls;
#[path = "source_array_transport_growth.rs"]
mod source_array_transport_growth;
#[path = "source_array_transport_limits.rs"]
mod source_array_transport_limits;
#[path = "source_array_transport_pressure.rs"]
mod source_array_transport_pressure;
#[path = "source_array_transport_stress.rs"]
mod source_array_transport_stress;
#[path = "source_boundaries.rs"]
mod source_boundaries;
#[path = "source_field_collection.rs"]
mod source_field_collection;
#[path = "source_field_limits.rs"]
mod source_field_limits;
#[path = "source_field_update_depth.rs"]
mod source_field_update_depth;
#[path = "source_field_update_failures.rs"]
mod source_field_update_failures;
#[path = "source_field_update_limits.rs"]
mod source_field_update_limits;
#[path = "source_field_updates.rs"]
mod source_field_updates;
#[path = "source_field_width.rs"]
mod source_field_width;
#[path = "source_localized_strings.rs"]
mod source_localized_strings;
#[path = "source_locations.rs"]
mod source_locations;
#[path = "source_object_comparison.rs"]
mod source_object_comparison;
#[path = "source_string_access.rs"]
mod source_string_access;
#[path = "source_strings_diagnostics.rs"]
mod source_strings_diagnostics;
#[path = "source_strings_execution.rs"]
mod source_strings_execution;
#[path = "source_strings_limits.rs"]
mod source_strings_limits;
#[path = "source_strings_width.rs"]
mod source_strings_width;
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
#[path = "undefined_source.rs"]
mod undefined_source;
