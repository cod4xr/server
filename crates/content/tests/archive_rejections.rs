// SPDX-License-Identifier: AGPL-3.0-or-later
//! Adversarial archive metadata and admission budget tests.
#![allow(
    clippy::expect_used,
    reason = "Tests assert exact admission boundaries."
)]

mod common;

use cod4x_content::{ArchiveBudget, ArchiveError, ArchiveResource, IwdArchive};

#[test]
fn paths_encryption_symlinks_and_contradictory_headers_are_rejected() {
    for (label, error) in [
        ("traversal", ArchiveError::InvalidPath),
        ("absolute", ArchiveError::InvalidPath),
        ("colon", ArchiveError::InvalidPath),
        ("dot_component", ArchiveError::InvalidPath),
        ("repeated_separator", ArchiveError::InvalidPath),
        ("symlink", ArchiveError::UnsupportedEntry),
        ("encrypted", ArchiveError::UnsupportedEntry),
        ("unsupported", ArchiveError::UnsupportedEntry),
        ("bad_local_name", ArchiveError::InvalidArchive),
    ] {
        assert_eq!(
            IwdArchive::open(&common::fixture(label), ArchiveBudget::default()).err(),
            Some(error),
            "{label}"
        );
    }
}

#[test]
fn every_admission_budget_accepts_exact_limits_and_rejects_one_less() {
    let bytes = common::fixture("single");
    let usage = IwdArchive::open(&bytes, ArchiveBudget::default())
        .expect("Archive")
        .usage();
    let exact = ArchiveBudget {
        max_archive_bytes: usage.archive_bytes,
        max_compressed_bytes: usage.compressed_bytes,
        max_metadata_bytes: usage.metadata_bytes,
        max_entries: 1,
        max_path_bytes: b"images/SECRET.iwi".len(),
        max_name_bytes: usage.name_bytes,
        max_entry_bytes: usage.declared_bytes,
        max_total_bytes: usage.declared_bytes,
    };
    assert!(IwdArchive::open(&bytes, exact).is_ok());
    let variants = [
        (
            ArchiveResource::CompressedBytes,
            ArchiveBudget {
                max_compressed_bytes: exact.max_compressed_bytes - 1,
                ..exact
            },
        ),
        (
            ArchiveResource::ArchiveBytes,
            ArchiveBudget {
                max_archive_bytes: exact.max_archive_bytes - 1,
                ..exact
            },
        ),
        (
            ArchiveResource::MetadataBytes,
            ArchiveBudget {
                max_metadata_bytes: exact.max_metadata_bytes - 1,
                ..exact
            },
        ),
        (
            ArchiveResource::Entries,
            ArchiveBudget {
                max_entries: 0,
                ..exact
            },
        ),
        (
            ArchiveResource::PathBytes,
            ArchiveBudget {
                max_path_bytes: exact.max_path_bytes - 1,
                ..exact
            },
        ),
        (
            ArchiveResource::NameBytes,
            ArchiveBudget {
                max_name_bytes: exact.max_name_bytes - 1,
                ..exact
            },
        ),
        (
            ArchiveResource::EntryBytes,
            ArchiveBudget {
                max_entry_bytes: exact.max_entry_bytes - 1,
                ..exact
            },
        ),
        (
            ArchiveResource::TotalBytes,
            ArchiveBudget {
                max_total_bytes: exact.max_total_bytes - 1,
                ..exact
            },
        ),
    ];
    for (resource, budget) in variants {
        assert_eq!(
            IwdArchive::open(&bytes, budget).err(),
            Some(ArchiveError::BudgetExceeded { resource })
        );
    }
    assert!(
        IwdArchive::open(&bytes, exact).is_ok(),
        "Rejection preserves source storage"
    );
}

#[test]
fn duplicate_entries_count_toward_resource_budgets() {
    let bytes = common::fixture("baseline");
    let budget = ArchiveBudget {
        max_entries: 6,
        ..ArchiveBudget::default()
    };
    assert_eq!(
        IwdArchive::open(&bytes, budget).err(),
        Some(ArchiveError::BudgetExceeded {
            resource: ArchiveResource::Entries
        })
    );
}

#[test]
fn all_truncations_and_deterministic_metadata_mutations_are_bounded() {
    let bytes = common::fixture("single");
    for end in 0..bytes.len() {
        assert!(IwdArchive::open(&bytes[..end], ArchiveBudget::default()).is_err());
    }
    let path = common::path(b"images/secret.iwi");
    let budget = ArchiveBudget {
        max_archive_bytes: 4096,
        max_compressed_bytes: 4096,
        max_metadata_bytes: 4096,
        max_entries: 16,
        max_path_bytes: 256,
        max_name_bytes: 4096,
        max_entry_bytes: 4096,
        max_total_bytes: 4096,
    };
    for index in 0..bytes.len() {
        for value in [0, 1, 0x7f, 0xff] {
            let mut mutated = bytes.clone();
            mutated[index] = value;
            if let Ok(mut archive) = IwdArchive::open(&mutated, budget)
                && let Ok(output) = archive.read(&path)
            {
                assert!(output.len() <= 4096);
            }
        }
    }
}
