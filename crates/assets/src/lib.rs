// SPDX-License-Identifier: AGPL-3.0-or-later
//! Growable engine-owned asset storage, independent of legacy wire namespaces.
//!
//! This catalog stores opaque encoded bytes and exact byte names. It does not
//! decode/render assets, access files, or claim a legacy client can display a
//! larger catalog. Format readers, validated filesystem paths, decoded/GPU
//! residency, dependencies, and per-peer projection are separate boundaries.

mod asset_budget;
mod asset_catalog;
mod asset_error;
mod asset_id;
mod asset_kind;
mod asset_registration;
mod catalog_usage;
mod stored_asset;

pub use asset_budget::AssetBudget;
pub use asset_catalog::AssetCatalog;
pub use asset_error::{AssetError, AssetResource};
pub use asset_id::AssetId;
pub use asset_kind::AssetKind;
pub use catalog_usage::CatalogUsage;
pub use stored_asset::StoredAsset;
