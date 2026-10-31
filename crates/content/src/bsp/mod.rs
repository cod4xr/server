// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded IBSP container metadata, without decoded geometry or global map state.

mod bsp_budget;
mod bsp_error;
mod bsp_file;
mod bsp_index;
mod bsp_lump;
mod lump_id;

pub use bsp_budget::BspBudget;
pub use bsp_error::{BspError, BspResource};
pub use bsp_file::{BspFile, BspLayout};
pub use bsp_lump::BspLump;
pub use lump_id::LumpId;
