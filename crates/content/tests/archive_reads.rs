// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent archive content, integrity, and duplicate-precedence tests.
#![allow(
    clippy::expect_used,
    reason = "Tests assert expected independent archive behavior."
)]

mod common;

use cod4x_content::{ArchiveBudget, ArchiveError, IwdArchive};

#[test]
fn independent_stored_deflate_empty_duplicate_and_zip64_entries_read_correctly() {
    for label in ["baseline", "zip64"] {
        let bytes = common::fixture(label);
        let mut archive = IwdArchive::open(&bytes, ArchiveBudget::default()).expect("Archive");
        assert_eq!(archive.usage().entries, 7);
        assert_eq!(archive.usage().files, 4);
        archive
            .verify_all()
            .expect("Every original entry validates");
        let badge = common::path(b"IMAGES\\badge.IWI");
        assert!(archive.contains(&badge));
        assert_eq!(archive.read(&badge).expect("Last matching entry"), b"last");
        assert_eq!(
            archive.read(&common::path(b"empty.bin")).expect("Empty"),
            b""
        );
        assert_eq!(
            archive
                .read(&common::path(b"scripts/example.gsc"))
                .expect("Script"),
            b"print(\"synthetic\");\n".repeat(30)
        );
        let long = [b"long/".as_slice(), &[b'x'; 150]].concat();
        assert_eq!(
            archive.read(&common::path(&long)).expect("Long path"),
            (0..=255).collect::<Vec<u8>>()
        );
        assert_eq!(archive.read(&badge).expect("Repeated read"), b"last");
        assert_eq!(
            archive.read(&common::path(b"missing")),
            Err(ArchiveError::NotFound)
        );
    }
    let bytes = common::fixture("zip64_local");
    let mut archive = IwdArchive::open(&bytes, ArchiveBudget::default()).expect("ZIP64 local");
    assert_eq!(
        archive
            .read(&common::path(b"large-format.bin"))
            .expect("Small body"),
        b"small"
    );
}

#[test]
fn crc_and_declared_output_mismatches_never_return_partial_content() {
    for label in ["bad_crc", "short_output", "long_output"] {
        let bytes = common::fixture(label);
        let mut archive =
            IwdArchive::open(&bytes, ArchiveBudget::default()).expect("Valid metadata");
        for _ in 0..2 {
            assert_eq!(archive.verify_all(), Err(ArchiveError::InvalidArchive));
            assert_eq!(
                archive.read(&common::path(b"images/secret.iwi")),
                Err(ArchiveError::InvalidArchive),
                "{label}"
            );
        }
    }
}

#[test]
fn full_verification_checks_corrupt_hidden_duplicates_without_deleting_them() {
    let bytes = common::fixture("hidden_bad_crc");
    let mut archive = IwdArchive::open(&bytes, ArchiveBudget::default()).expect("Metadata");
    assert_eq!(archive.usage().entries, 2);
    assert_eq!(archive.usage().files, 1);
    let path = common::path(b"same.bin");
    assert_eq!(archive.read(&path).expect("Visible entry"), b"visible");
    assert_eq!(archive.verify_all(), Err(ArchiveError::InvalidArchive));
    assert_eq!(
        archive
            .read(&path)
            .expect("Read after verification failure"),
        b"visible"
    );
}

#[test]
fn empty_archives_are_valid_and_debug_output_excludes_names_and_data() {
    let bytes = common::fixture("empty");
    let archive = IwdArchive::open(&bytes, ArchiveBudget::default()).expect("Empty archive");
    assert_eq!(archive.usage().entries, 0);
    assert_eq!(archive.usage().files, 0);
    let bytes = common::fixture("single");
    let archive = IwdArchive::open(&bytes, ArchiveBudget::default()).expect("One file");
    let debug = format!("{archive:?} {:?}", common::path(b"secret-name"));
    assert!(!debug.to_lowercase().contains("secret"));
    assert!(!ArchiveError::InvalidArchive.to_string().contains("secret"));
}
