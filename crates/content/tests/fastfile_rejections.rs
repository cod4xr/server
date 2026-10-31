// SPDX-License-Identifier: AGPL-3.0-or-later
//! Malformed envelope rejection, exact resource admission, and bounded expansion.
#![allow(clippy::expect_used, reason = "Tests use known independent fixtures.")]

mod fastfile_support;

use cod4x_content::fastfile::{FastFileBudget, FastFileError, FastFileResource, FastFileStream};

#[test]
fn invalid_envelopes_do_not_publish_partial_payloads() {
    for (name, expected) in [
        ("bad_magic", FastFileError::InvalidMagic),
        ("bad_version", FastFileError::UnsupportedVersion),
        ("signed", FastFileError::UnsupportedAuthentication),
        ("bad_checksum", FastFileError::Compression),
        ("bad_compression", FastFileError::Compression),
        ("short_header", FastFileError::SizeMismatch),
        ("short_payload", FastFileError::SizeMismatch),
        ("long_payload", FastFileError::SizeMismatch),
        ("trailer", FastFileError::TrailingData),
        ("concatenated", FastFileError::TrailingData),
    ] {
        assert_eq!(
            FastFileStream::decode(&fastfile_support::fixture(name), FastFileBudget::default())
                .expect_err("Invalid stream"),
            expected,
            "{name}"
        );
    }
    let budget = FastFileBudget {
        max_payload_bytes: u64::from(u32::MAX),
        ..FastFileBudget::default()
    };
    assert_eq!(
        FastFileStream::decode(&fastfile_support::fixture("huge_declaration"), budget)
            .expect_err("No backing payload"),
        FastFileError::SizeMismatch
    );
}

#[test]
fn every_truncation_and_deterministic_hostile_prefix_is_bounded() {
    let source = fastfile_support::fixture("small");
    for length in 0..source.len() {
        assert!(FastFileStream::decode(&source[..length], FastFileBudget::default()).is_err());
    }
    let budget = FastFileBudget {
        max_payload_bytes: 1024,
        max_file_bytes: 1024,
        max_external_bytes: 2_000_000,
        max_block_bytes: 1024,
        max_total_block_bytes: 1024,
    };
    for index in 0..source.len() {
        for replacement in [0, 1, 127, 255] {
            let mut mutated = source.clone();
            mutated[index] = replacement;
            let result = FastFileStream::decode(&mutated, budget);
            if let Ok(stream) = result {
                assert!(stream.payload().len() <= 1024);
            }
        }
    }
}

#[test]
fn each_budget_admits_exactly_its_boundary_and_reports_its_resource() {
    let source = fastfile_support::fixture("small");
    let exact = FastFileBudget {
        max_file_bytes: source.len() as u64,
        max_payload_bytes: 25,
        max_external_bytes: 1_223_352,
        max_block_bytes: 223,
        max_total_block_bytes: 387,
    };
    FastFileStream::decode(&source, exact).expect("All exact boundaries");
    for (budget, resource) in [
        (
            FastFileBudget {
                max_file_bytes: exact.max_file_bytes - 1,
                ..exact
            },
            FastFileResource::FileBytes,
        ),
        (
            FastFileBudget {
                max_payload_bytes: 24,
                ..exact
            },
            FastFileResource::PayloadBytes,
        ),
        (
            FastFileBudget {
                max_external_bytes: 1_223_351,
                ..exact
            },
            FastFileResource::ExternalBytes,
        ),
        (
            FastFileBudget {
                max_block_bytes: 222,
                ..exact
            },
            FastFileResource::BlockBytes,
        ),
        (
            FastFileBudget {
                max_total_block_bytes: 386,
                ..exact
            },
            FastFileResource::TotalBlockBytes,
        ),
    ] {
        assert_eq!(
            FastFileStream::decode(&source, budget).expect_err("Budget"),
            FastFileError::BudgetExceeded(resource)
        );
    }
    let source = fastfile_support::fixture("expansion");
    assert!(source.len() < 1024);
    let budget = FastFileBudget {
        max_payload_bytes: 200_002,
        ..FastFileBudget::default()
    };
    assert_eq!(
        FastFileStream::decode(&source, budget).expect_err("Expansion quota"),
        FastFileError::BudgetExceeded(FastFileResource::PayloadBytes)
    );
}
