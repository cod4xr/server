// SPDX-License-Identifier: AGPL-3.0-or-later
//! Root parsing failures preserve input and identify specific exhausted resources.
#![allow(clippy::expect_used, reason = "Tests use independent known fixtures.")]

mod zone_support;

use cod4x_content::zone::{
    ZonePrefix, ZonePrefixBudget, ZonePrefixError, ZonePrefixResource, ZoneString,
};

#[test]
fn missing_negative_truncated_and_out_of_block_tables_are_rejected() {
    for (name, error) in [
        ("missing_strings", ZonePrefixError::MissingTable),
        ("missing_assets", ZonePrefixError::MissingTable),
        ("negative_count", ZonePrefixError::InvalidCount),
        ("short_string", ZonePrefixError::Truncated),
        ("short_table", ZonePrefixError::Truncated),
        ("short_assets", ZonePrefixError::Truncated),
        ("short_block", ZonePrefixError::BlockBounds),
    ] {
        assert_eq!(
            ZonePrefix::parse(&zone_support::fixture(name), ZonePrefixBudget::default())
                .expect_err("Invalid prefix"),
            error,
            "{name}"
        );
    }
}

#[test]
fn all_metadata_resources_admit_exactly_their_boundary() {
    let stream = zone_support::fixture("mixed");
    let metadata = 5 * size_of::<ZoneString<'_>>() as u64;
    let exact = ZonePrefixBudget {
        max_script_strings: 5,
        max_assets: 3,
        max_string_bytes: 9,
        max_total_string_bytes: 10,
        max_metadata_bytes: metadata,
    };
    ZonePrefix::parse(&stream, exact).expect("Exact boundaries");
    for (budget, resource) in [
        (
            ZonePrefixBudget {
                max_script_strings: 4,
                ..exact
            },
            ZonePrefixResource::ScriptStrings,
        ),
        (
            ZonePrefixBudget {
                max_assets: 2,
                ..exact
            },
            ZonePrefixResource::Assets,
        ),
        (
            ZonePrefixBudget {
                max_string_bytes: 8,
                ..exact
            },
            ZonePrefixResource::StringBytes,
        ),
        (
            ZonePrefixBudget {
                max_total_string_bytes: 9,
                ..exact
            },
            ZonePrefixResource::TotalStringBytes,
        ),
        (
            ZonePrefixBudget {
                max_metadata_bytes: metadata - 1,
                ..exact
            },
            ZonePrefixResource::MetadataBytes,
        ),
    ] {
        assert_eq!(
            ZonePrefix::parse(&stream, budget).expect_err("Budget"),
            ZonePrefixError::BudgetExceeded(resource)
        );
    }
}

#[test]
fn all_prefix_truncations_and_hostile_root_mutations_are_bounded() {
    let source = zone_support::fixture("mixed").into_payload();
    let prefix_bytes = source.len() - b"secret-body".len();
    for length in 0..prefix_bytes {
        let stream = zone_support::from_payload(&source[..length], 56);
        assert!(ZonePrefix::parse(&stream, ZonePrefixBudget::default()).is_err());
    }
    let budget = ZonePrefixBudget {
        max_script_strings: 16,
        max_assets: 16,
        max_string_bytes: 16,
        max_total_string_bytes: 64,
        max_metadata_bytes: 1024,
    };
    for index in 0..prefix_bytes {
        for replacement in [0, 1, 127, 255] {
            let mut bytes = source.clone();
            bytes[index] = replacement;
            let stream = zone_support::from_payload(&bytes, 64);
            if let Ok(prefix) = ZonePrefix::parse(&stream, budget) {
                assert!(prefix.strings().len() <= 16);
                assert!(prefix.assets().len() <= 16);
                assert!(prefix.virtual_bytes() <= 64);
            }
        }
    }
    assert_eq!(source, zone_support::fixture("mixed").payload());
}
