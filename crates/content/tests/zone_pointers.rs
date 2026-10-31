// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source-derived offsets and checked range classification without host pointers.
#![allow(
    clippy::expect_used,
    reason = "Tests decode known original C fixtures."
)]

mod zone_support;

use cod4x_content::zone::{
    ZoneAddress, ZoneAssetRecord, ZonePointer, ZonePointerError, ZonePrefix, ZonePrefixBudget,
};

fn address(raw: u32) -> ZoneAddress {
    let decoded = match ZonePointer::decode(raw).expect("Offset encoding") {
        ZonePointer::Offset(address) => Some(address),
        _ => None,
    };
    decoded.expect("Known non-sentinel fixture offset")
}

#[test]
fn original_pointer_conversion_preserves_each_block_and_relative_offset() {
    let mut count = 0;
    for line in include_str!("fixtures/zone_pointer_vectors.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
    {
        let row: Vec<u32> = line
            .split_ascii_whitespace()
            .map(|field| field.parse().expect("Number"))
            .collect();
        let decoded = address(row[0]);
        assert_eq!(u32::from(decoded.block()), row[1]);
        assert_eq!(decoded.offset(), u64::from(row[2]));
        count += 1;
    }
    assert_eq!(count, 90);
}

#[test]
fn sentinel_words_block_boundaries_and_invalid_tags_are_distinct() {
    assert_eq!(ZonePointer::decode(0), Ok(ZonePointer::Null));
    assert_eq!(ZonePointer::decode(u32::MAX), Ok(ZonePointer::Inline));
    assert_eq!(
        ZonePointer::decode(u32::MAX - 1),
        Ok(ZonePointer::InlineWithAlias)
    );
    for block in 0_u32..9 {
        let first = address((block << 28) + 1);
        let last = address((block << 28) + 0x1000_0000);
        assert_eq!(u32::from(first.block()), block);
        assert_eq!(first.offset(), 0);
        assert_eq!(u32::from(last.block()), block);
        assert_eq!(last.offset(), 0x0fff_ffff);
    }
    for raw in [0x9000_0001, 0xa000_0001, 0xf000_0001, u32::MAX - 2] {
        assert_eq!(
            ZonePointer::decode(raw),
            Err(ZonePointerError::InvalidBlock)
        );
    }
}

#[test]
fn span_admission_uses_checked_wide_sizes_without_claiming_target_residency() {
    let stream = zone_support::fixture("alignment");
    let header = stream.header(); // virtual block four has exactly sixteen bytes.
    let first = address(0x4000_0001);
    assert_eq!(first.validate_span(header, 16), Ok(()));
    assert_eq!(
        first.validate_span(header, 17),
        Err(ZonePointerError::BlockBounds)
    );
    let end = address(0x4000_0011);
    assert_eq!(end.validate_span(header, 0), Ok(()));
    assert_eq!(
        end.validate_span(header, 1),
        Err(ZonePointerError::BlockBounds)
    );
    assert_eq!(
        end.validate_span(header, u64::MAX),
        Err(ZonePointerError::SizeOverflow)
    );
    assert_eq!(
        address(0x4000_0012).validate_span(header, 0),
        Err(ZonePointerError::BlockBounds)
    );
    assert_eq!(address(1).validate_span(header, 0), Ok(()));
    assert_eq!(
        address(1).validate_span(header, 1),
        Err(ZonePointerError::BlockBounds)
    );
}

#[test]
fn asset_header_classification_is_lazy_and_preserves_unresolved_raw_words() {
    let stream = zone_support::fixture("mixed");
    let prefix = ZonePrefix::parse(&stream, ZonePrefixBudget::default()).expect("Prefix");
    let pointers: Vec<_> = prefix
        .assets()
        .map(ZoneAssetRecord::decode_header)
        .collect();
    assert_eq!(
        pointers,
        [
            Ok(ZonePointer::Inline),
            Ok(ZonePointer::Null),
            Ok(ZonePointer::Offset(address(0x4000_0001)))
        ]
    );
}
