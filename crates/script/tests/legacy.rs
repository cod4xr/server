// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scalar/string compatibility suite; leaf files keep independent responsibilities.

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
#[path = "kernel_integer_admission.rs"]
mod kernel_integer_admission;
#[path = "kernel_module_hostile.rs"]
mod kernel_module_hostile;
#[path = "kernel_module_verification.rs"]
mod kernel_module_verification;
#[path = "kernel_reference_verification.rs"]
mod kernel_reference_verification;
#[path = "kernel_reference_width.rs"]
mod kernel_reference_width;
#[path = "legacy_vectors.rs"]
mod legacy_vectors;
#[path = "numeric_boundaries.rs"]
mod numeric_boundaries;
#[path = "numeric_vectors.rs"]
mod numeric_vectors;
#[path = "predicate_vectors.rs"]
mod predicate_vectors;
#[path = "string_operation_vectors.rs"]
mod string_operation_vectors;
#[path = "string_operations.rs"]
mod string_operations;
#[path = "undefined_vectors.rs"]
mod undefined_vectors;
#[path = "wait_boundaries.rs"]
mod wait_boundaries;
