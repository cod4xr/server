// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scalar/string compatibility suite; leaf files keep independent responsibilities.

mod string_support;

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
