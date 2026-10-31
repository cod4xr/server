// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete-operation budget failures, exact identity, and diagnostic redaction.
#![allow(
    clippy::expect_used,
    reason = "Tests create small valid baseline assets."
)]

use cod4x_assets::{AssetBudget, AssetCatalog, AssetError, AssetKind, AssetResource};

#[test]
fn each_budget_reports_its_resource_and_preserves_existing_contents_and_usage() {
    let cases = [
        (
            AssetBudget {
                max_assets: 1,
                ..AssetBudget::default()
            },
            b"bb".as_slice(),
            b"456".as_slice(),
            AssetResource::Slots,
        ),
        (
            AssetBudget {
                max_payload_bytes: 5,
                ..AssetBudget::default()
            },
            b"bb",
            b"456",
            AssetResource::PayloadBytes,
        ),
        (
            AssetBudget {
                max_single_asset_bytes: 3,
                ..AssetBudget::default()
            },
            b"bb",
            b"4567",
            AssetResource::SingleAssetBytes,
        ),
        (
            AssetBudget {
                max_name_bytes: 1,
                ..AssetBudget::default()
            },
            b"bb",
            b"4",
            AssetResource::NameBytes,
        ),
        (
            AssetBudget {
                max_total_name_bytes: 4,
                ..AssetBudget::default()
            },
            b"bb",
            b"4",
            AssetResource::TotalNameBytes,
        ),
    ];
    for (budget, name, bytes, expected_resource) in cases {
        let mut catalog = AssetCatalog::try_new(budget).expect("Catalog");
        let baseline = catalog
            .register(AssetKind::Image, b"a", b"123")
            .expect("Baseline");
        let before = catalog.usage();
        let error = catalog
            .register(AssetKind::Image, name, bytes)
            .expect_err("Configured budget");
        assert!(matches!(error, AssetError::BudgetExceeded { resource, .. }
            if resource == expected_resource));
        assert_eq!(catalog.usage(), before);
        assert_eq!(
            catalog.get(baseline).expect("Baseline retained").bytes(),
            b"123"
        );
        assert!(catalog.find(AssetKind::Image, name).is_none());
    }
}

#[test]
fn exact_budgets_accept_complete_assets_and_removal_restores_available_units() {
    let budget = AssetBudget {
        max_assets: 2,
        max_payload_bytes: 6,
        max_single_asset_bytes: 3,
        max_name_bytes: 1,
        max_total_name_bytes: 4,
    };
    let mut catalog = AssetCatalog::try_new(budget).expect("Catalog");
    let first = catalog
        .register(AssetKind::Image, b"a", b"123")
        .expect("First exact asset");
    let second = catalog
        .register(AssetKind::Image, b"b", b"456")
        .expect("Second exact asset");
    assert_eq!(catalog.usage().assets, 2);
    assert_eq!(catalog.usage().payload_bytes, 6);
    assert_eq!(catalog.usage().name_bytes, 4);
    assert!(catalog.register(AssetKind::Image, b"c", b"789").is_err());
    let _removed = catalog.remove(first).expect("Free exact units");
    let third = catalog
        .register(AssetKind::Image, b"c", b"789")
        .expect("Reclaimed budgets");
    assert!(catalog.get(second).is_some());
    assert!(catalog.get(third).is_some());
    assert_eq!(catalog.usage().peak_payload_bytes, 6);
}

#[test]
fn duplicate_identity_is_nondestructive_and_names_remain_exact_raw_bytes() {
    let mut catalog = AssetCatalog::try_new(AssetBudget::default()).expect("Catalog");
    let first = catalog
        .register(AssetKind::Image, b"a\xff", b"original")
        .expect("High-byte name");
    let before = catalog.usage();
    assert_eq!(
        catalog.register(AssetKind::Image, b"a\xff", b"replacement"),
        Err(AssetError::DuplicateName)
    );
    assert_eq!(catalog.usage(), before);
    assert_eq!(catalog.get(first).expect("Unchanged").bytes(), b"original");
    let different_case = catalog
        .register(AssetKind::Image, b"A\xff", b"case")
        .expect("Exact case identity");
    let different_kind = catalog
        .register(AssetKind::Model, b"a\xff", b"kind")
        .expect("Separate kind");
    assert_ne!(first, different_case);
    assert_ne!(first, different_kind);
    assert_eq!(catalog.find(AssetKind::Image, b"a\xff"), Some(first));
}

#[test]
fn invalid_names_and_debug_output_never_disclose_asset_contents() {
    let mut catalog = AssetCatalog::try_new(AssetBudget::default()).expect("Catalog");
    assert_eq!(
        catalog.register(AssetKind::Image, b"", b"data"),
        Err(AssetError::EmptyName)
    );
    assert_eq!(
        catalog.register(AssetKind::Image, b"bad\0name", b"data"),
        Err(AssetError::EmbeddedTerminator)
    );
    let name = b"private-name-sentinel";
    let payload = b"private-payload-sentinel";
    let id = catalog
        .register(AssetKind::Script, name, payload)
        .expect("Asset");
    let debug = format!("{catalog:?} {:?}", catalog.get(id).expect("Stored asset"));
    assert!(!debug.contains("private-name-sentinel"));
    assert!(!debug.contains("private-payload-sentinel"));
}
