// SPDX-License-Identifier: AGPL-3.0-or-later
//! Expanded catalog counts and stale-handle-safe unload/reload behavior.
#![allow(
    clippy::expect_used,
    reason = "Catalog tests use explicit small payloads and budgets."
)]

use cod4x_assets::{AssetBudget, AssetCatalog, AssetKind};

#[test]
fn full_synthetic_character_and_prestige_catalog_exceeds_old_pool_counts() {
    let mut catalog = AssetCatalog::try_new(AssetBudget::default()).expect("Catalog namespace");
    let mut ids = Vec::new();
    for index in 0..5000 {
        let name = if index < 50 {
            format!("ui/prestige/tier_{index}")
        } else {
            format!("ui/characters/icon_{index}")
        };
        ids.push(
            catalog
                .register(AssetKind::Image, name.as_bytes(), b"img")
                .expect("Grow beyond the audited 2400-image default"),
        );
    }
    for index in 0..1500 {
        let name = format!("characters/player_{index}");
        ids.push(
            catalog
                .register(AssetKind::Model, name.as_bytes(), b"mdl")
                .expect("Grow beyond the audited 1000-model default"),
        );
    }
    assert_eq!(catalog.usage().assets, 6500);
    assert_eq!(catalog.usage().payload_bytes, 19_500);
    assert!(ids.iter().all(|&id| catalog.get(id).is_some()));
    for tier in 0..50 {
        assert!(
            catalog
                .find(
                    AssetKind::Image,
                    format!("ui/prestige/tier_{tier}").as_bytes()
                )
                .is_some()
        );
    }
    for character in 0..100 {
        assert!(
            catalog
                .find(
                    AssetKind::Model,
                    format!("characters/player_{character}").as_bytes()
                )
                .is_some()
        );
    }
}

#[test]
fn stale_and_cross_catalog_ids_never_resolve_to_reused_slots() {
    let budget = AssetBudget {
        max_assets: 1,
        ..AssetBudget::default()
    };
    let mut first = AssetCatalog::try_new(budget).expect("First catalog");
    let mut second = AssetCatalog::try_new(budget).expect("Second catalog");
    let original = first
        .register(AssetKind::Image, b"name", b"old")
        .expect("First asset");
    let other = second
        .register(AssetKind::Image, b"name", b"other")
        .expect("Other catalog asset");
    assert_ne!(original, other);
    assert!(first.get(other).is_none());
    assert!(second.get(original).is_none());
    assert!(first.remove(other).is_none());
    let removed = first.remove(original).expect("Remove owned asset");
    assert_eq!(removed.bytes(), b"old");
    assert_eq!(removed.name(), b"name");
    assert_eq!(first.usage().assets, 0);
    assert_eq!(first.usage().payload_bytes, 0);
    assert_eq!(first.usage().name_bytes, 0);
    let replacement = first
        .register(AssetKind::Image, b"name", b"new")
        .expect("Recycled slot");
    assert_ne!(original, replacement);
    assert!(first.get(original).is_none());
    assert!(first.remove(original).is_none());
    assert_eq!(
        first
            .get(replacement)
            .expect("Replacement remains live")
            .bytes(),
        b"new"
    );
    assert_eq!(first.usage().slots, 1);
}

#[test]
fn repeated_unload_reload_reuses_metadata_without_reusing_handles() {
    let budget = AssetBudget {
        max_assets: 1,
        ..AssetBudget::default()
    };
    let mut catalog = AssetCatalog::try_new(budget).expect("Catalog");
    let mut previous = None;
    for _ in 0..10_000 {
        let id = catalog
            .register(AssetKind::Script, b"script", b"source")
            .expect("Register");
        if let Some(previous) = previous {
            assert_ne!(id, previous);
            assert!(catalog.get(previous).is_none());
        }
        assert_eq!(catalog.remove(id).expect("Remove").bytes(), b"source");
        previous = Some(id);
        assert_eq!(catalog.usage().slots, 1);
        assert_eq!(catalog.usage().assets, 0);
    }
    assert_eq!(catalog.usage().peak_payload_bytes, 6);
}

#[test]
fn catalog_payloads_and_names_do_not_inherit_message_or_virtual_path_sizes() {
    let mut catalog = AssetCatalog::try_new(AssetBudget::default()).expect("Catalog");
    let name = vec![b'n'; 512];
    let payload = vec![0x42; 128 * 1024 + 1];
    let id = catalog
        .register(AssetKind::RawFile, &name, &payload)
        .expect("Independent storage budget");
    let asset = catalog.get(id).expect("Stored asset");
    assert_eq!(asset.name(), name);
    assert_eq!(asset.bytes(), payload);
    assert_eq!(catalog.usage().name_bytes, 1024);
    let large_budget = AssetBudget {
        max_payload_bytes: 8 * 1024 * 1024 * 1024,
        max_single_asset_bytes: 5 * 1024 * 1024 * 1024,
        ..AssetBudget::default()
    };
    let wide = AssetCatalog::try_new(large_budget).expect("Wide configurable byte budgets");
    assert!(wide.budget().max_payload_bytes > u64::from(u32::MAX));
}
