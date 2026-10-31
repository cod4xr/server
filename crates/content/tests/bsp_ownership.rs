// SPDX-License-Identifier: AGPL-3.0-or-later
//! Expanded metadata, independent instances, and zero-copy payload ownership.
#![allow(
    clippy::expect_used,
    reason = "Tests construct explicit synthetic BSP containers."
)]

mod bsp_support;

use cod4x_content::bsp::{BspBudget, BspError, BspFile, BspResource, LumpId};

#[test]
fn metadata_exceeds_the_old_100_chunk_array_and_preserves_full_raw_kinds() {
    let mut source = b"IBSP".to_vec();
    source.extend_from_slice(&22_u32.to_le_bytes());
    source.extend_from_slice(&1000_u32.to_le_bytes());
    for index in 0..1000_u32 {
        source.extend_from_slice(&(0x0001_0000 + index).to_le_bytes());
        source.extend_from_slice(&1_u32.to_le_bytes());
    }
    for index in 0..1000_u32 {
        source.extend_from_slice(&[u8::try_from(index % 256).expect("Byte"), 0xa5, 0xa5, 0xa5]);
    }
    let budget = BspBudget {
        max_file_bytes: 1 << 34,
        max_lumps: 1000,
        max_lump_bytes: 1 << 32,
        max_total_lump_bytes: 3 << 32,
    };
    let file = BspFile::parse(&source, budget).expect("Wide budgets without narrowing");
    assert_eq!(file.lumps().len(), 1000);
    assert_eq!(
        file.lump(LumpId::from_raw(0x0001_0000))
            .expect("First")
            .as_bytes(),
        [0]
    );
    assert_eq!(
        file.lump(LumpId::from_raw(0x0001_0000 + 999))
            .expect("Last")
            .as_bytes(),
        [231]
    );
}

#[test]
fn independent_maps_and_copied_views_keep_their_original_immutable_backing_bytes() {
    let modern = bsp_support::source(22);
    let old = bsp_support::source(6);
    let first = BspFile::parse(&modern, BspBudget::default()).expect("Modern map");
    let second = BspFile::parse(&old, BspBudget::default()).expect("Old map");
    let view = first.lump(LumpId::from_raw(4)).expect("Modern planes");
    assert_ne!(
        view.as_bytes(),
        second
            .lump(LumpId::from_raw(4))
            .expect("Old planes")
            .as_bytes()
    );
    let offset = usize::try_from(view.offset()).expect("Small source");
    assert_eq!(
        view.as_bytes().as_ptr(),
        modern.as_ptr().wrapping_add(offset)
    );
    drop(first);
    assert_eq!(
        view.as_bytes().len(),
        8,
        "Directory ownership is independent of source lifetime"
    );
    assert_eq!(second.version(), 6);
    assert_eq!(second.lumps().len(), 41);
}

#[test]
fn repeated_old_payload_spans_consume_summed_budgets_and_empty_offsets_are_not_dereferenced() {
    let mut source = bsp_support::source(6);
    let directory_end = 8 + 41 * 8;
    bsp_support::word(&mut source, 8 + 8, 3);
    bsp_support::word(&mut source, 12 + 8, directory_end);
    bsp_support::word(&mut source, 12 + 2 * 8, u32::MAX);
    let budget = BspBudget {
        max_total_lump_bytes: 13,
        ..BspBudget::default()
    };
    assert_eq!(
        BspFile::parse(&source, budget).err(),
        Some(BspError::BudgetExceeded(BspResource::TotalLumpBytes))
    );
    let file = BspFile::parse(
        &source,
        BspBudget {
            max_total_lump_bytes: 14,
            ..budget
        },
    )
    .expect("Exact repeated-span allowance");
    assert_eq!(
        file.lump(LumpId::from_raw(0)).expect("First").as_bytes(),
        file.lump(LumpId::from_raw(1)).expect("Alias").as_bytes()
    );
    let empty = file.lump(LumpId::from_raw(2)).expect("Empty entry");
    assert_eq!(empty.offset(), u64::from(u32::MAX));
    assert_eq!(empty.records(1).expect("Empty records").count(), 0);
}
