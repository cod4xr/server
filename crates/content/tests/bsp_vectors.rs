// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original C BSP container lookups, element counts, presence, and checksums.
#![allow(
    clippy::expect_used,
    reason = "Tests compare valid original C results."
)]

mod bsp_support;

use cod4x_content::bsp::{BspBudget, BspFile, BspLayout, LumpId};

#[test]
fn every_source_supported_container_version_matches_original_lump_and_checksum_functions() {
    let mut count = 0;
    for row in bsp_support::rows() {
        assert_eq!(row.len(), 10);
        let version: u32 = row[1].parse().expect("Version");
        let source = bsp_support::hex(row[2]);
        let file = BspFile::parse(&source, BspBudget::default()).expect("Container");
        assert_eq!(file.version(), version);
        assert_eq!(
            file.layout(),
            if version <= 18 {
                BspLayout::OffsetDirectory
            } else {
                BspLayout::SequentialChunks
            }
        );
        let kind = LumpId::from_raw(row[3].parse().expect("Kind"));
        let element_bytes: usize = row[4].parse().expect("Element size");
        let expected_count: usize = row[5].parse().expect("Count");
        let expected_bytes = bsp_support::hex(row[7]);
        let lump = file.lump(kind);
        let actual_bytes = lump.map_or([].as_slice(), |lump| lump.as_bytes());
        assert_eq!(actual_bytes, expected_bytes);
        assert_eq!(actual_bytes.len() / element_bytes, expected_count);
        assert_eq!(!actual_bytes.is_empty(), row[8] == "1");
        if let Some(lump) = lump {
            assert_eq!(
                lump.records(element_bytes).expect("Whole records").count(),
                expected_count
            );
            if row[6] != "~" {
                assert_eq!(lump.offset(), row[6].parse::<u64>().expect("Offset"));
            }
        }
        assert_eq!(
            file.legacy_checksum(0),
            row[9].parse::<u32>().expect("Checksum")
        );
        count += 1;
    }
    assert_eq!(count, 280);
}

#[test]
fn duplicate_unknown_and_empty_lumps_preserve_order_and_first_match() {
    let source = bsp_support::source(22);
    let file = BspFile::parse(&source, BspBudget::default()).expect("Container");
    assert_eq!(file.lumps().len(), 6);
    assert_eq!(
        file.lump(LumpId::from_raw(4))
            .expect("First plane lump")
            .as_bytes()
            .len(),
        8
    );
    assert_eq!(file.lumps()[3].id(), LumpId::from_raw(4));
    assert_eq!(file.lumps()[3].as_bytes().len(), 2);
    assert_eq!(
        file.lump(LumpId::from_raw(77))
            .expect("Unknown semantic kind")
            .as_bytes()
            .len(),
        5
    );
    let empty = file.lump(LumpId::from_raw(1)).expect("Present empty kind");
    assert!(empty.as_bytes().is_empty());
    assert!(empty.offset() > source.len() as u64);
}
