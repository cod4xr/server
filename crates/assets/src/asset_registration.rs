// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashMap;

use crate::asset_catalog::AssetSlot;
use crate::{AssetCatalog, AssetError, AssetId, AssetKind, AssetResource, StoredAsset};

impl AssetCatalog {
    /// Copies and registers one complete opaque encoded asset.
    ///
    /// Caller input remains owned by the caller. All budget/name checks and
    /// fallible capacity preparations complete before an ID becomes visible.
    /// Failure preserves entries, handles, contents, and logical accounting;
    /// successful reservations can retain additional unused metadata capacity.
    /// Exact kind/name duplicates are rejected rather than replaced or evicted.
    ///
    /// # Errors
    /// Rejects empty/NUL names, duplicates, configured budget exhaustion, size
    /// overflow, and allocation/capacity reservation failures.
    pub fn register(
        &mut self,
        kind: AssetKind,
        name: &[u8],
        bytes: &[u8],
    ) -> Result<AssetId, AssetError> {
        let (payload_bytes, name_bytes) = self.validate_registration(kind, name, bytes)?;
        let owned_bytes = copy_bytes(bytes)?;
        let owned_name = copy_bytes(name)?;
        let indexed_name = copy_bytes(name)?;
        self.reserve_slot()?;
        let mut prepared_index = HashMap::new();
        if let Some(index) = self.names.get_mut(&kind) {
            index
                .try_reserve(1)
                .map_err(|_| AssetError::AllocationFailed)?;
        } else {
            self.names
                .try_reserve(1)
                .map_err(|_| AssetError::AllocationFailed)?;
            prepared_index
                .try_reserve(1)
                .map_err(|_| AssetError::AllocationFailed)?;
        }
        let index = self.names.entry(kind).or_insert(prepared_index);
        let asset = StoredAsset {
            kind,
            name: owned_name,
            bytes: owned_bytes,
        };
        let (slot_index, generation) = if let Some(slot_index) = self.free_head {
            let slot = &mut self.slots[slot_index];
            self.free_head = slot.next_free.take();
            slot.asset = Some(asset);
            (slot_index, slot.generation)
        } else {
            let slot_index = self.slots.len();
            self.slots.push(AssetSlot {
                generation: 1,
                asset: Some(asset),
                next_free: None,
            });
            (slot_index, 1)
        };
        let id = AssetId {
            catalog: self.catalog,
            slot: slot_index,
            generation,
        };
        let replaced = index.insert(indexed_name, id);
        debug_assert!(
            replaced.is_none(),
            "Validated registration must not replace another asset"
        );
        self.usage.assets += 1;
        self.usage.slots = self.slots.len();
        self.usage.payload_bytes += payload_bytes;
        self.usage.name_bytes += name_bytes;
        self.usage.peak_payload_bytes = self.usage.peak_payload_bytes.max(self.usage.payload_bytes);
        Ok(id)
    }

    fn validate_registration(
        &self,
        kind: AssetKind,
        name: &[u8],
        bytes: &[u8],
    ) -> Result<(u64, u64), AssetError> {
        if name.is_empty() {
            return Err(AssetError::EmptyName);
        }
        let name_length = u64::try_from(name.len()).map_err(|_| AssetError::SizeOverflow)?;
        check_budget(
            AssetResource::NameBytes,
            name_length,
            u64::try_from(self.budget.max_name_bytes).map_err(|_| AssetError::SizeOverflow)?,
        )?;
        if name.contains(&0) {
            return Err(AssetError::EmbeddedTerminator);
        }
        if self.find(kind, name).is_some() {
            return Err(AssetError::DuplicateName);
        }
        if self.free_head.is_none() && self.slots.len() >= self.budget.max_assets {
            return Err(AssetError::BudgetExceeded {
                resource: AssetResource::Slots,
                requested: 1,
                available: 0,
            });
        }
        let payload_bytes = u64::try_from(bytes.len()).map_err(|_| AssetError::SizeOverflow)?;
        let name_bytes = name_length.checked_mul(2).ok_or(AssetError::SizeOverflow)?;
        check_budget(
            AssetResource::SingleAssetBytes,
            payload_bytes,
            self.budget.max_single_asset_bytes,
        )?;
        check_budget(
            AssetResource::PayloadBytes,
            payload_bytes,
            self.budget
                .max_payload_bytes
                .saturating_sub(self.usage.payload_bytes),
        )?;
        check_budget(
            AssetResource::TotalNameBytes,
            name_bytes,
            self.budget
                .max_total_name_bytes
                .saturating_sub(self.usage.name_bytes),
        )?;
        Ok((payload_bytes, name_bytes))
    }

    fn reserve_slot(&mut self) -> Result<(), AssetError> {
        if self.free_head.is_some() || self.slots.len() < self.slots.capacity() {
            return Ok(());
        }
        let growth = self
            .slots
            .capacity()
            .max(64)
            .min(self.budget.max_assets - self.slots.len());
        self.slots
            .try_reserve_exact(growth)
            .map_err(|_| AssetError::AllocationFailed)
    }
}

fn check_budget(resource: AssetResource, requested: u64, available: u64) -> Result<(), AssetError> {
    if requested > available {
        return Err(AssetError::BudgetExceeded {
            resource,
            requested,
            available,
        });
    }
    Ok(())
}

fn copy_bytes(source: &[u8]) -> Result<Vec<u8>, AssetError> {
    let mut owned = Vec::new();
    owned
        .try_reserve_exact(source.len())
        .map_err(|_| AssetError::AllocationFailed)?;
    owned.extend_from_slice(source);
    Ok(owned)
}
