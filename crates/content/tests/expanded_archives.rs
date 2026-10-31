// SPDX-License-Identifier: AGPL-3.0-or-later
//! ZIP64 entry counts and highly compressible content budget coverage.
#![allow(
    clippy::expect_used,
    reason = "Tests construct controlled synthetic ZIP archives."
)]

mod common;

use std::io::{Cursor, Write};

use cod4x_content::{ArchiveBudget, ArchiveError, ArchiveResource, IwdArchive};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

#[test]
fn configurable_zip64_catalog_exceeds_the_legacy_16_bit_entry_count() {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for index in 0..66_000 {
        writer
            .start_file(format!("images/{index}.iwi"), options)
            .expect("Entry");
    }
    let bytes = writer.finish().expect("ZIP64 directory").into_inner();
    let budget = ArchiveBudget {
        max_entries: 66_000,
        ..ArchiveBudget::default()
    };
    let mut archive = IwdArchive::open(&bytes, budget).expect("Expanded archive");
    assert_eq!(archive.usage().entries, 66_000);
    assert_eq!(archive.usage().files, 66_000);
    assert_eq!(
        archive
            .read(&common::path(b"images/65999.iwi"))
            .expect("Last"),
        b""
    );
    assert_eq!(
        archive.read(&common::path(b"images/0.iwi")).expect("First"),
        b""
    );
}

#[test]
fn declared_expansion_is_rejected_before_decompression_and_forged_sizes_fail() {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    writer.start_file("expansion.bin", options).expect("Entry");
    writer
        .write_all(&vec![b'x'; 1024 * 1024])
        .expect("Highly compressible data");
    let bytes = writer.finish().expect("Archive").into_inner();
    assert!(bytes.len() < 4096);
    let budget = ArchiveBudget {
        max_entry_bytes: 1024,
        ..ArchiveBudget::default()
    };
    assert_eq!(
        IwdArchive::open(&bytes, budget).err(),
        Some(ArchiveError::BudgetExceeded {
            resource: ArchiveResource::EntryBytes
        })
    );
    let mut forged = bytes;
    let central = forged
        .windows(4)
        .position(|part| part == b"PK\x01\x02")
        .expect("Directory");
    forged[22..26].copy_from_slice(&1_u32.to_le_bytes());
    forged[central + 24..central + 28].copy_from_slice(&1_u32.to_le_bytes());
    let mut archive = IwdArchive::open(&forged, budget).expect("Small declared size");
    assert_eq!(archive.verify_all(), Err(ArchiveError::InvalidArchive));
    assert_eq!(
        archive.read(&common::path(b"expansion.bin")),
        Err(ArchiveError::InvalidArchive)
    );
}
