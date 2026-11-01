// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit legacy numeric/vector semantics, independent of engine IDs and modern types.

mod legacy_arithmetic;
mod legacy_comparison;
mod legacy_integer_operator;
mod legacy_integer_update;
mod legacy_number;
mod legacy_number_error;
mod legacy_numeric_pair;
mod legacy_vector;
mod legacy_vector_error;
mod legacy_vector_operand;
mod legacy_vector_operation;
mod legacy_vector_pair;
mod legacy_wait_error;
mod legacy_wait_rate;

pub use legacy_arithmetic::LegacyArithmetic;
pub use legacy_comparison::LegacyComparison;
pub use legacy_integer_operator::LegacyIntegerOperator;
pub use legacy_integer_update::LegacyIntegerUpdate;
pub use legacy_number::LegacyNumber;
pub use legacy_number_error::LegacyNumberError;
pub use legacy_numeric_pair::LegacyNumericPair;
pub use legacy_vector::LegacyVector;
pub use legacy_vector_error::LegacyVectorError;
pub use legacy_vector_operand::LegacyVectorOperand;
pub use legacy_vector_operation::LegacyVectorOperation;
pub use legacy_vector_pair::LegacyVectorPair;
pub use legacy_wait_error::LegacyWaitError;
pub use legacy_wait_rate::LegacyWaitRate;
