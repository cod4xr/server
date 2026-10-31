// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit legacy scalar semantics, independent of engine IDs and modern types.

mod legacy_number;
mod legacy_number_error;
mod legacy_wait_error;
mod legacy_wait_rate;

pub use legacy_number::LegacyNumber;
pub use legacy_number_error::LegacyNumberError;
pub use legacy_wait_error::LegacyWaitError;
pub use legacy_wait_rate::LegacyWaitRate;
