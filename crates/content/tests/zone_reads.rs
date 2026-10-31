// SPDX-License-Identifier: AGPL-3.0-or-later
//! Root metadata preserves complete slots and raw directory records without native pointers.
#![allow(clippy::expect_used, reason = "Tests use independent known fixtures.")]

mod zone_support;

use cod4x_content::zone::{ZonePrefix, ZonePrefixBudget, ZoneString};

#[test]
fn mixed_root_slots_and_unknown_asset_records_preserve_order_and_bytes() {
    let stream = zone_support::fixture("mixed");
    let prefix = ZonePrefix::parse(&stream, ZonePrefixBudget::default()).expect("Prefix");
    assert_eq!(
        prefix.strings(),
        &[
            ZoneString::Null,
            ZoneString::Inline(b""),
            ZoneString::Inline(b"secret\xff\x80"),
            ZoneString::Unresolved(0x4000_0001),
            ZoneString::Unresolved(0xffff_fffe)
        ]
    );
    let records: Vec<_> = prefix
        .assets()
        .map(|asset| (asset.kind(), asset.encoded_header()))
        .collect();
    assert_eq!(records, [(3, u32::MAX), (u32::MAX, 0), (3, 0x4000_0001)]);
    assert_eq!(prefix.assets().next_back().expect("Last").kind(), 3);
    assert_eq!(prefix.remaining(), b"secret-body");
    assert_eq!(prefix.virtual_bytes(), 56);
    assert!(!format!("{prefix:?}").contains("secret"));
    for string in prefix.strings() {
        assert!(!format!("{string:?}").contains("secret"));
    }
}

#[test]
fn virtual_alignment_consumes_block_space_without_skipping_serialized_bytes() {
    let stream = zone_support::fixture("alignment");
    let prefix = ZonePrefix::parse(&stream, ZonePrefixBudget::default()).expect("Prefix");
    assert_eq!(prefix.strings(), &[ZoneString::Inline(b"ab")]);
    assert_eq!(prefix.virtual_bytes(), 16);
    assert_eq!(prefix.assets().next().expect("Raw file").kind(), 31);
    assert_eq!(prefix.remaining(), b"tail");
    let tail_start = stream.payload().len() - 4;
    assert_eq!(
        prefix.remaining().as_ptr(),
        stream.payload()[tail_start..].as_ptr()
    );
}

#[test]
fn independent_instances_and_wide_catalogs_keep_every_serialized_slot() {
    let empty = zone_support::fixture("empty");
    let empty_prefix = ZonePrefix::parse(&empty, ZonePrefixBudget::default()).expect("Empty");
    let wide = zone_support::fixture("wide");
    let wide_prefix = ZonePrefix::parse(&wide, ZonePrefixBudget::default()).expect("Wide");
    assert_eq!(wide_prefix.strings().len(), 70_000);
    assert!(
        wide_prefix
            .strings()
            .iter()
            .all(|slot| *slot == ZoneString::Null)
    );
    assert_eq!(wide_prefix.assets().len(), 70_000);
    assert_eq!(wide_prefix.remaining(), b"tail");
    assert_eq!(empty_prefix.assets().len(), 0);
    assert!(empty_prefix.strings().is_empty());
    assert_eq!(empty_prefix.virtual_bytes(), 0);
}
