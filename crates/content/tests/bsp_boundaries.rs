// SPDX-License-Identifier: AGPL-3.0-or-later
//! BSP malformed-input, budget, element-size, and redaction boundaries.
#![allow(
    clippy::expect_used,
    reason = "Tests patch bounded synthetic fixture fields."
)]

mod bsp_support;

use cod4x_content::bsp::{BspBudget, BspError, BspFile, BspResource, LumpId};

#[test]
fn each_budget_accepts_exact_limits_and_rejects_one_less() {
    let source = bsp_support::source(22);
    let budget = BspBudget {
        max_file_bytes: source.len() as u64,
        max_lumps: 6,
        max_lump_bytes: 8,
        max_total_lump_bytes: 18,
    };
    assert!(BspFile::parse(&source, budget).is_ok());
    for (resource, limited) in [
        (
            BspResource::FileBytes,
            BspBudget {
                max_file_bytes: budget.max_file_bytes - 1,
                ..budget
            },
        ),
        (
            BspResource::Lumps,
            BspBudget {
                max_lumps: 5,
                ..budget
            },
        ),
        (
            BspResource::LumpBytes,
            BspBudget {
                max_lump_bytes: 7,
                ..budget
            },
        ),
        (
            BspResource::TotalLumpBytes,
            BspBudget {
                max_total_lump_bytes: 17,
                ..budget
            },
        ),
    ] {
        assert_eq!(
            BspFile::parse(&source, limited).err(),
            Some(BspError::BudgetExceeded(resource))
        );
    }
    assert!(BspFile::parse(&source, budget).is_ok());
}

#[test]
fn invalid_magic_versions_truncations_and_hostile_directory_counts_are_rejected() {
    let source = bsp_support::source(22);
    for end in 0..source.len() {
        assert!(BspFile::parse(&source[..end], BspBudget::default()).is_err());
    }
    let mut wrong = source.clone();
    wrong[0] = b'X';
    assert_eq!(
        BspFile::parse(&wrong, BspBudget::default()).err(),
        Some(BspError::InvalidMagic)
    );
    for version in [0, 5, 46, u32::MAX] {
        let mut wrong = source.clone();
        bsp_support::word(&mut wrong, 4, version);
        assert_eq!(
            BspFile::parse(&wrong, BspBudget::default()).err(),
            Some(BspError::UnsupportedVersion)
        );
    }
    let mut wrong = source;
    bsp_support::word(&mut wrong, 8, u32::MAX);
    assert_eq!(
        BspFile::parse(&wrong, BspBudget::default()).err(),
        Some(BspError::BudgetExceeded(BspResource::Lumps))
    );
    let budget = BspBudget {
        max_lumps: usize::MAX,
        ..BspBudget::default()
    };
    assert!(
        BspFile::parse(&wrong, budget).is_err(),
        "Directory bytes checked before allocation"
    );
}

#[test]
fn old_payloads_cannot_overlap_the_directory_or_wrap_through_32_bit_arithmetic() {
    let source = bsp_support::source(6);
    let mut wrong = source.clone();
    bsp_support::word(&mut wrong, 12, 8);
    assert_eq!(
        BspFile::parse(&wrong, BspBudget::default()).err(),
        Some(BspError::DirectoryOverlap)
    );
    let mut wrong = source;
    bsp_support::word(&mut wrong, 8, 16);
    bsp_support::word(&mut wrong, 12, u32::MAX - 7);
    assert_eq!(
        BspFile::parse(&wrong, BspBudget::default()).err(),
        Some(BspError::Truncated)
    );
}

#[test]
fn encoded_records_require_whole_nonzero_sizes_and_debug_omits_entity_text() {
    let payload = b"do-not-log-map-entity-text";
    let mut source = b"IBSP".to_vec();
    for word in [
        22_u32,
        1,
        39,
        u32::try_from(payload.len()).expect("Small payload"),
    ] {
        source.extend_from_slice(&word.to_le_bytes());
    }
    source.extend_from_slice(payload);
    let file = BspFile::parse(&source, BspBudget::default()).expect("Container");
    let lump = file.lump(LumpId::from_raw(39)).expect("Entity text");
    assert_eq!(lump.records(0).err(), Some(BspError::InvalidElementSize));
    assert_eq!(
        lump.records(payload.len() + 1).err(),
        Some(BspError::InvalidElementSize)
    );
    assert_eq!(
        lump.records(1).expect("Byte records").count(),
        payload.len()
    );
    assert!(!format!("{file:?} {lump:?}").contains("do-not-log"));
}

#[test]
fn deterministic_header_and_payload_mutations_keep_all_admitted_spans_bounded() {
    let source = bsp_support::source(22);
    let budget = BspBudget {
        max_file_bytes: 4096,
        max_lumps: 100,
        max_lump_bytes: 512,
        max_total_lump_bytes: 2048,
    };
    for offset in 0..source.len() {
        for value in [0, 1, 0x7f, 0xff] {
            let mut mutated = source.clone();
            mutated[offset] = value;
            if let Ok(file) = BspFile::parse(&mutated, budget) {
                assert!(file.lumps().len() <= 100);
                let total: usize = file.lumps().iter().map(|lump| lump.as_bytes().len()).sum();
                assert!(total <= 2048);
                for lump in file.lumps() {
                    if !lump.as_bytes().is_empty() {
                        assert!(
                            lump.offset() + lump.as_bytes().len() as u64 <= source.len() as u64
                        );
                    }
                }
            }
        }
    }
}
