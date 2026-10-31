// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scalar/string compatibility suite; leaf files keep independent responsibilities.

mod kernel_support;
mod string_support;

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
#[path = "kernel_vectors.rs"]
mod kernel_vectors;
#[path = "kernel_verification.rs"]
mod kernel_verification;

#[path = "arithmetic_boundaries.rs"]
mod arithmetic_boundaries;
#[path = "arithmetic_vectors.rs"]
mod arithmetic_vectors;
#[path = "legacy_vectors.rs"]
mod legacy_vectors;
#[path = "numeric_boundaries.rs"]
mod numeric_boundaries;
#[path = "numeric_vectors.rs"]
mod numeric_vectors;
#[path = "string_operation_vectors.rs"]
mod string_operation_vectors;
#[path = "string_operations.rs"]
mod string_operations;
#[path = "wait_boundaries.rs"]
mod wait_boundaries;
