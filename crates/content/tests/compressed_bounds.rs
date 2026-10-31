// SPDX-License-Identifier: AGPL-3.0-or-later
//! Compressed span and repeated-work budget admission tests.
#![allow(
    clippy::expect_used,
    reason = "Tests patch explicit synthetic ZIP directory fields."
)]

mod common;

use cod4x_content::{ArchiveBudget, ArchiveError, ArchiveResource, IwdArchive};

fn duplicate_reference() -> Vec<u8> {
    let single = common::fixture("single");
    let central = single
        .windows(4)
        .position(|part| part == b"PK\x01\x02")
        .expect("Directory");
    let end = single.len() - 22;
    let mut duplicated = single[..end].to_vec();
    duplicated.extend_from_slice(&single[central..end]);
    let directory_size = u32::try_from((end - central) * 2).expect("Small fixture");
    let mut footer = single[end..].to_vec();
    footer[8..10].copy_from_slice(&2_u16.to_le_bytes());
    footer[10..12].copy_from_slice(&2_u16.to_le_bytes());
    footer[12..16].copy_from_slice(&directory_size.to_le_bytes());
    duplicated.extend_from_slice(&footer);
    duplicated
}

#[test]
fn repeated_references_to_one_payload_consume_the_compressed_work_budget() {
    let bytes = duplicate_reference();
    let budget = ArchiveBudget {
        max_compressed_bytes: 17,
        ..ArchiveBudget::default()
    };
    assert_eq!(
        IwdArchive::open(&bytes, budget).err(),
        Some(ArchiveError::BudgetExceeded {
            resource: ArchiveResource::CompressedBytes,
        })
    );
    let budget = ArchiveBudget {
        max_compressed_bytes: 34,
        ..budget
    };
    let archive = IwdArchive::open(&bytes, budget).expect("Exact duplicate work allowance");
    assert_eq!(archive.usage().entries, 2);
    assert_eq!(archive.usage().files, 1);
    assert_eq!(archive.usage().compressed_bytes, 34);
    archive
        .verify_all()
        .expect("Both references verify within the budget");
}

#[test]
fn payload_spans_cannot_extend_into_directory_or_beyond_archive_storage() {
    let original = common::fixture("single");
    let central = original
        .windows(4)
        .position(|part| part == b"PK\x01\x02")
        .expect("Directory");
    for size in [18_u32, u32::MAX] {
        let mut bytes = original.clone();
        bytes[18..22].copy_from_slice(&size.to_le_bytes());
        bytes[central + 20..central + 24].copy_from_slice(&size.to_le_bytes());
        assert_eq!(
            IwdArchive::open(&bytes, ArchiveBudget::default()).err(),
            Some(ArchiveError::InvalidArchive)
        );
    }
}
