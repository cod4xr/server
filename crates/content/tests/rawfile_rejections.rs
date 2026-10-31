// SPDX-License-Identifier: AGPL-3.0-or-later
//! Resource and graph/body failures never publish partial raw files.
#![allow(clippy::expect_used, reason = "Tests use known synthetic fixtures.")]

mod zone_support;

use cod4x_content::zone::ZonePrefixBudget;

use cod4x_content::zone::rawfiles::{
    RawFileBudget, RawFileCollection, RawFileError, RawFileResource,
};

#[test]
fn unsupported_unbound_malformed_and_incomplete_collections_fail_completely() {
    for (label, expected) in [
        ("unsupported", RawFileError::UnsupportedAsset),
        ("unknown_alias", RawFileError::UnknownAlias),
        ("forward_alias", RawFileError::UnknownAlias),
        ("bad_temp", RawFileError::BlockBounds),
        ("bad_virtual", RawFileError::BlockBounds),
        ("negative_length", RawFileError::InvalidLength),
        ("invalid_name", RawFileError::InvalidNamePointer),
        ("missing_buffer", RawFileError::MissingBuffer),
        ("short_header", RawFileError::Truncated),
        ("short_name", RawFileError::Truncated),
        ("short_buffer", RawFileError::Truncated),
        ("bad_terminator", RawFileError::MissingTerminator),
        ("trailer", RawFileError::TrailingData),
    ] {
        let stream = zone_support::raw_fixture(label);
        assert_eq!(
            RawFileCollection::decode(&stream, RawFileBudget::default()).expect_err("Invalid"),
            expected,
            "{label}"
        );
    }
}

#[test]
fn all_body_truncations_and_deterministic_mutations_remain_bounded() {
    let source = zone_support::raw_fixture("inline").into_payload();
    let budget = RawFileBudget {
        prefix: ZonePrefixBudget {
            max_script_strings: 16,
            max_assets: 16,
            max_string_bytes: 32,
            max_total_string_bytes: 64,
            max_metadata_bytes: 1024,
        },
        max_name_bytes: 32,
        max_total_name_bytes: 64,
        max_buffer_bytes: 32,
        max_total_buffer_bytes: 64,
        max_metadata_bytes: 8192,
    };
    for length in 0..source.len() {
        let stream = zone_support::from_payload_with_temp(&source[..length], 128, 12);
        assert!(RawFileCollection::decode(&stream, budget).is_err());
    }
    for index in 0..source.len() {
        for replacement in [0, 1, 127, 255] {
            let mut mutated = source.clone();
            mutated[index] = replacement;
            let stream = zone_support::from_payload_with_temp(&mutated, 128, 12);
            if let Ok(collection) = RawFileCollection::decode(&stream, budget) {
                assert!(collection.entries().len() <= 16);
                assert!(collection.usage().virtual_bytes <= 128);
                for file in collection.entries().iter().flatten() {
                    assert!(file.name().len() < 32);
                    if let Some(bytes) = file.contents() {
                        assert!(bytes.len() < 32);
                    }
                }
            }
        }
    }
    assert_eq!(source, zone_support::raw_fixture("inline").payload());
}

#[test]
fn each_rawfile_budget_accepts_exact_usage_and_identifies_one_less() {
    let stream = zone_support::raw_fixture("inline");
    let usage = RawFileCollection::decode(&stream, RawFileBudget::default())
        .expect("Usage")
        .usage();
    let exact = RawFileBudget {
        max_name_bytes: 19,
        max_total_name_bytes: 19,
        max_buffer_bytes: 18,
        max_total_buffer_bytes: 18,
        max_metadata_bytes: usage.peak_metadata_bytes,
        ..RawFileBudget::default()
    };
    RawFileCollection::decode(&stream, exact).expect("Exact limits");
    for (budget, resource) in [
        (
            RawFileBudget {
                max_name_bytes: 18,
                ..exact
            },
            RawFileResource::NameBytes,
        ),
        (
            RawFileBudget {
                max_total_name_bytes: 18,
                ..exact
            },
            RawFileResource::TotalNameBytes,
        ),
        (
            RawFileBudget {
                max_buffer_bytes: 17,
                ..exact
            },
            RawFileResource::BufferBytes,
        ),
        (
            RawFileBudget {
                max_total_buffer_bytes: 17,
                ..exact
            },
            RawFileResource::TotalBufferBytes,
        ),
        (
            RawFileBudget {
                max_metadata_bytes: usage.peak_metadata_bytes - 1,
                ..exact
            },
            RawFileResource::MetadataBytes,
        ),
    ] {
        assert_eq!(
            RawFileCollection::decode(&stream, budget).expect_err("Quota"),
            RawFileError::BudgetExceeded(resource)
        );
    }
}
