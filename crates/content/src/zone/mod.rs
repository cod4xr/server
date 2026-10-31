// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded IW3 root metadata; asset bodies and encoded references remain unresolved.

mod prefix_cursor;
mod script_strings;
mod zone_address;
mod zone_asset_record;
mod zone_pointer;
mod zone_pointer_error;
mod zone_prefix;
mod zone_prefix_budget;
mod zone_prefix_error;
mod zone_string;

pub use zone_address::ZoneAddress;
pub use zone_asset_record::ZoneAssetRecord;
pub use zone_pointer::ZonePointer;
pub use zone_pointer_error::ZonePointerError;
pub use zone_prefix::ZonePrefix;
pub use zone_prefix_budget::ZonePrefixBudget;
pub use zone_prefix_error::{ZonePrefixError, ZonePrefixResource};
pub use zone_string::ZoneString;
