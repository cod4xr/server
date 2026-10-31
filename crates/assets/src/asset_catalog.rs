// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashMap;
use std::fmt;

use crate::asset_id;
use crate::{AssetBudget, AssetError, AssetId, AssetKind, CatalogUsage, StoredAsset};

pub(super) type NameIndex = HashMap<Vec<u8>, AssetId>;

pub(super) struct AssetSlot {
    pub(super) generation: u64,
    pub(super) asset: Option<StoredAsset>,
    pub(super) next_free: Option<usize>,
}

/// Growable, generation-checked catalog with configurable ownership budgets.
///
/// Names are exact bytes within each kind, not filesystem paths or legacy
/// normalized names. Import/compatibility adapters must validate aliases. No
/// registration automatically evicts, truncates, or replaces existing content.
pub struct AssetCatalog {
    pub(super) catalog: u64,
    pub(super) budget: AssetBudget,
    pub(super) slots: Vec<AssetSlot>,
    pub(super) names: HashMap<AssetKind, NameIndex>,
    pub(super) free_head: Option<usize>,
    pub(super) usage: CatalogUsage,
}

impl AssetCatalog {
    /// Creates an empty catalog without allocating asset/metadata capacity.
    ///
    /// # Errors
    /// Returns [`AssetError::IdentityExhausted`] instead of reusing a namespace.
    pub fn try_new(budget: AssetBudget) -> Result<Self, AssetError> {
        Ok(Self {
            catalog: asset_id::allocate_catalog()?,
            budget,
            slots: Vec::new(),
            names: HashMap::new(),
            free_head: None,
            usage: CatalogUsage::default(),
        })
    }

    /// Returns the configured logical ownership budgets.
    #[must_use]
    pub fn budget(&self) -> AssetBudget {
        self.budget
    }

    /// Returns current and high-water catalog ownership statistics.
    #[must_use]
    pub fn usage(&self) -> CatalogUsage {
        self.usage
    }

    /// Looks up an exact kind/name without allocating or changing the catalog.
    #[must_use]
    pub fn find(&self, kind: AssetKind, name: &[u8]) -> Option<AssetId> {
        if name.is_empty() || name.len() > self.budget.max_name_bytes {
            return None;
        }
        self.names.get(&kind)?.get(name).copied()
    }

    /// Resolves only a live handle belonging to this catalog and generation.
    #[must_use]
    pub fn get(&self, id: AssetId) -> Option<&StoredAsset> {
        if id.catalog != self.catalog {
            return None;
        }
        let slot = self.slots.get(id.slot)?;
        if slot.generation != id.generation {
            return None;
        }
        slot.asset.as_ref()
    }

    /// Removes a live asset, returns its owned bytes, and invalidates the handle.
    ///
    /// Stale/cross-catalog handles are no-ops. Removal allocates no memory. A
    /// generation that cannot advance retires its metadata slot instead of
    /// wrapping. Returned data is caller-owned and no longer counted as resident
    /// catalog data. Metadata capacity remains available for future registration.
    #[must_use]
    pub fn remove(&mut self, id: AssetId) -> Option<StoredAsset> {
        if id.catalog != self.catalog {
            return None;
        }
        let slot = self.slots.get_mut(id.slot)?;
        if slot.generation != id.generation {
            return None;
        }
        let asset = slot.asset.as_ref()?;
        let payload_bytes = u64::try_from(asset.bytes.len()).ok()?;
        let name_bytes = u64::try_from(asset.name.len()).ok()?.checked_mul(2)?;
        let asset = slot.asset.take()?;
        if let Some(index) = self.names.get_mut(&asset.kind) {
            let removed = index.remove(asset.name.as_slice());
            debug_assert_eq!(
                removed,
                Some(id),
                "Name index must identify the removed live asset"
            );
        }
        self.usage.assets -= 1;
        self.usage.payload_bytes -= payload_bytes;
        self.usage.name_bytes -= name_bytes;
        if let Some(generation) = slot.generation.checked_add(1) {
            slot.generation = generation;
            slot.next_free = self.free_head;
            self.free_head = Some(id.slot);
        }
        Some(asset)
    }
}

impl fmt::Debug for AssetCatalog {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AssetCatalog")
            .field("usage", &self.usage)
            .field("budget", &self.budget)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::expect_used,
        reason = "Tests set up a mature generation without global mutation."
    )]

    use crate::{AssetBudget, AssetCatalog, AssetError, AssetKind, AssetResource};

    #[test]
    fn exhausted_generation_retires_the_slot_without_wrapping_or_reusing_the_id() {
        let budget = AssetBudget {
            max_assets: 1,
            ..AssetBudget::default()
        };
        let mut catalog = AssetCatalog::try_new(budget).expect("Catalog");
        let mut id = catalog
            .register(AssetKind::Image, b"a", b"x")
            .expect("Asset");
        catalog.slots[id.slot].generation = u64::MAX;
        id.generation = u64::MAX;
        *catalog
            .names
            .get_mut(&AssetKind::Image)
            .expect("Kind index")
            .get_mut(b"a".as_slice())
            .expect("Name index") = id;
        assert!(catalog.remove(id).is_some());
        assert!(catalog.get(id).is_none());
        assert_eq!(catalog.usage().assets, 0);
        assert_eq!(catalog.usage().slots, 1);
        assert!(matches!(
            catalog.register(AssetKind::Image, b"b", b"y"),
            Err(AssetError::BudgetExceeded {
                resource: AssetResource::Slots,
                ..
            })
        ));
    }
}
