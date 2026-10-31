// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit legacy scalar semantics, independent of engine IDs and modern types.

mod legacy_arithmetic;
mod legacy_comparison;
mod legacy_integer_operator;
mod legacy_integer_update;
mod legacy_number;
mod legacy_number_error;
mod legacy_numeric_pair;
mod legacy_wait_error;
mod legacy_wait_rate;

pub use legacy_arithmetic::LegacyArithmetic;
pub use legacy_comparison::LegacyComparison;
pub use legacy_integer_operator::LegacyIntegerOperator;
pub use legacy_integer_update::LegacyIntegerUpdate;
pub use legacy_number::LegacyNumber;
pub use legacy_number_error::LegacyNumberError;
pub use legacy_numeric_pair::LegacyNumericPair;
pub use legacy_wait_error::LegacyWaitError;
pub use legacy_wait_rate::LegacyWaitRate;
