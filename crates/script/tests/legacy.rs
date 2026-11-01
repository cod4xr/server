// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scalar/string compatibility suite; leaf files keep independent responsibilities.
#[path = "array_copy_budgets.rs"]
mod array_copy_budgets;
#[path = "array_copy_churn.rs"]
mod array_copy_churn;
#[path = "array_copy_index_work.rs"]
mod array_copy_index_work;
#[path = "array_copy_model.rs"]
mod array_copy_model;
#[path = "array_copy_roots.rs"]
mod array_copy_roots;
mod array_copy_support;
#[path = "array_copy_values.rs"]
mod array_copy_values;
#[path = "array_copy_vectors.rs"]
mod array_copy_vectors;
#[path = "array_copy_width.rs"]
mod array_copy_width;
#[path = "array_copy_work.rs"]
mod array_copy_work;
#[path = "array_entry_budgets.rs"]
mod array_entry_budgets;
#[path = "array_entry_hostile.rs"]
mod array_entry_hostile;
#[path = "array_entry_model.rs"]
mod array_entry_model;
#[path = "array_entry_roots.rs"]
mod array_entry_roots;
#[path = "array_entry_stress.rs"]
mod array_entry_stress;
#[path = "array_entry_values.rs"]
mod array_entry_values;
#[path = "array_entry_vectors.rs"]
mod array_entry_vectors;
#[path = "array_entry_work.rs"]
mod array_entry_work;
#[path = "array_value_churn.rs"]
mod array_value_churn;
#[path = "array_value_collection.rs"]
mod array_value_collection;
#[path = "array_value_model.rs"]
mod array_value_model;
#[path = "array_value_vectors.rs"]
mod array_value_vectors;
#[path = "defined_limits.rs"]
mod defined_limits;
#[path = "defined_vectors.rs"]
mod defined_vectors;
#[path = "field_tables.rs"]
mod field_tables;
#[path = "kernel_object_lifetime.rs"]
mod kernel_object_lifetime;
#[path = "kernel_object_limits.rs"]
mod kernel_object_limits;
#[path = "kernel_object_roots.rs"]
mod kernel_object_roots;
#[path = "kernel_object_values.rs"]
mod kernel_object_values;
#[path = "object_execution_boundary.rs"]
mod object_execution_boundary;
#[path = "object_heap_budgets.rs"]
mod object_heap_budgets;
#[path = "object_heap_collection_limits.rs"]
mod object_heap_collection_limits;
#[path = "object_heap_graphs.rs"]
mod object_heap_graphs;
#[path = "object_heap_growth.rs"]
mod object_heap_growth;
#[path = "object_heap_hostile.rs"]
mod object_heap_hostile;
#[path = "object_heap_ownership.rs"]
mod object_heap_ownership;
#[path = "object_heap_stress.rs"]
mod object_heap_stress;
mod object_heap_support;
#[path = "object_size_vectors.rs"]
mod object_size_vectors;
#[path = "string_access_limits.rs"]
mod string_access_limits;
#[path = "string_access_vectors.rs"]
mod string_access_vectors;
#[path = "struct_limits.rs"]
mod struct_limits;
#[path = "struct_pressure.rs"]
mod struct_pressure;
#[path = "struct_vectors.rs"]
mod struct_vectors;
#[path = "struct_width.rs"]
mod struct_width;

mod kernel_support;
mod kernel_vector_support;

#[path = "kernel_vector_index.rs"]
mod kernel_vector_index;

#[path = "vector_index_vectors.rs"]
mod vector_index_vectors;

#[path = "kernel_vector_execution.rs"]
mod kernel_vector_execution;
#[path = "kernel_vector_limits.rs"]
mod kernel_vector_limits;
mod string_support;
mod vector_support;

#[path = "kernel_vector_vectors.rs"]
mod kernel_vector_vectors;

#[path = "vector_boundaries.rs"]
mod vector_boundaries;
#[path = "vector_value_vectors.rs"]
mod vector_value_vectors;

#[path = "integer_updates.rs"]
mod integer_updates;
#[path = "kernel_budgets.rs"]
mod kernel_budgets;
#[path = "kernel_execution.rs"]
mod kernel_execution;
#[path = "kernel_hostile.rs"]
mod kernel_hostile;
#[path = "kernel_local_budgets.rs"]
mod kernel_local_budgets;
#[path = "kernel_local_hostile.rs"]
mod kernel_local_hostile;
#[path = "kernel_local_verification.rs"]
mod kernel_local_verification;
#[path = "kernel_locals.rs"]
mod kernel_locals;
#[path = "kernel_script_argument_budgets.rs"]
mod kernel_script_argument_budgets;
#[path = "kernel_script_arguments.rs"]
mod kernel_script_arguments;
#[path = "kernel_vectors.rs"]
mod kernel_vectors;
#[path = "kernel_verification.rs"]
mod kernel_verification;

#[path = "arithmetic_boundaries.rs"]
mod arithmetic_boundaries;
#[path = "arithmetic_vectors.rs"]
mod arithmetic_vectors;
#[path = "array_key_strings.rs"]
mod array_key_strings;
#[path = "array_key_vectors.rs"]
mod array_key_vectors;
#[path = "array_value_budgets.rs"]
mod array_value_budgets;
#[path = "array_value_ownership.rs"]
mod array_value_ownership;
#[path = "array_value_width.rs"]
mod array_value_width;
#[path = "array_value_work.rs"]
mod array_value_work;
#[path = "field_pressure.rs"]
mod field_pressure;
#[path = "field_vectors.rs"]
mod field_vectors;
#[path = "field_verification.rs"]
mod field_verification;
#[path = "function_reference_vectors.rs"]
mod function_reference_vectors;
#[path = "kernel_builtin_limits.rs"]
mod kernel_builtin_limits;
#[path = "kernel_call_arity.rs"]
mod kernel_call_arity;
#[path = "kernel_call_execution.rs"]
mod kernel_call_execution;
#[path = "kernel_call_limits.rs"]
mod kernel_call_limits;
#[path = "kernel_call_recursion.rs"]
mod kernel_call_recursion;
mod kernel_call_support;
#[path = "kernel_call_width.rs"]
mod kernel_call_width;
#[path = "kernel_function_references.rs"]
mod kernel_function_references;
#[path = "kernel_heap_churn.rs"]
mod kernel_heap_churn;
#[path = "kernel_heap_hostile.rs"]
mod kernel_heap_hostile;
#[path = "kernel_heap_lifetime.rs"]
mod kernel_heap_lifetime;
#[path = "kernel_heap_limits.rs"]
mod kernel_heap_limits;
#[path = "kernel_heap_pressure.rs"]
mod kernel_heap_pressure;
#[path = "kernel_heap_roots.rs"]
mod kernel_heap_roots;
mod kernel_heap_support;
#[path = "kernel_heap_values.rs"]
mod kernel_heap_values;
#[path = "kernel_heap_vectors.rs"]
mod kernel_heap_vectors;
#[path = "kernel_integer_admission.rs"]
mod kernel_integer_admission;
#[path = "kernel_literals_cache.rs"]
mod kernel_literals_cache;
#[path = "kernel_literals_failure.rs"]
mod kernel_literals_failure;
#[path = "kernel_literals_limits.rs"]
mod kernel_literals_limits;
#[path = "kernel_literals_proof.rs"]
mod kernel_literals_proof;
#[path = "kernel_literals_width.rs"]
mod kernel_literals_width;
#[path = "kernel_module_hostile.rs"]
mod kernel_module_hostile;
#[path = "kernel_module_verification.rs"]
mod kernel_module_verification;
#[path = "kernel_reference_verification.rs"]
mod kernel_reference_verification;
#[path = "kernel_reference_width.rs"]
mod kernel_reference_width;
#[path = "kernel_swap.rs"]
mod kernel_swap;
#[path = "legacy_vectors.rs"]
mod legacy_vectors;
#[path = "localized_roots.rs"]
mod localized_roots;
#[path = "localized_values.rs"]
mod localized_values;
#[path = "numeric_boundaries.rs"]
mod numeric_boundaries;
#[path = "numeric_vectors.rs"]
mod numeric_vectors;
#[path = "object_comparison_limits.rs"]
mod object_comparison_limits;
#[path = "object_comparison_vectors.rs"]
mod object_comparison_vectors;
#[path = "object_cycle_limits.rs"]
mod object_cycle_limits;
#[path = "object_cycle_model.rs"]
mod object_cycle_model;
#[path = "object_cycle_topology.rs"]
mod object_cycle_topology;
#[path = "object_cycle_vectors.rs"]
mod object_cycle_vectors;
#[path = "predicate_vectors.rs"]
mod predicate_vectors;
#[path = "string_equality_work.rs"]
mod string_equality_work;
#[path = "string_heap_budgets.rs"]
mod string_heap_budgets;
#[path = "string_heap_hostile.rs"]
mod string_heap_hostile;
#[path = "string_heap_ownership.rs"]
mod string_heap_ownership;
#[path = "string_heap_reclamation.rs"]
mod string_heap_reclamation;
#[path = "string_heap_stress.rs"]
mod string_heap_stress;
#[path = "string_heap_vectors.rs"]
mod string_heap_vectors;
#[path = "string_operation_vectors.rs"]
mod string_operation_vectors;
#[path = "string_operations.rs"]
mod string_operations;
#[path = "string_value_vectors.rs"]
mod string_value_vectors;
#[path = "undefined_vectors.rs"]
mod undefined_vectors;
#[path = "wait_boundaries.rs"]
mod wait_boundaries;
