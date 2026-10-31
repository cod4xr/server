// SPDX-License-Identifier: AGPL-3.0-or-later
//! Checked admission and exhaustive byte-location comparisons for bounded sources.
#![allow(clippy::expect_used, reason = "Tests own small valid source buffers.")]

use cod4x_script::source::{SourceBudget, SourceError, SourceResource, SourceText};

#[test]
fn every_resource_admits_exactly_its_limit_and_reports_one_less() {
    let bytes = b"a\r\n\nb\n";
    let exact = SourceBudget {
        max_source_bytes: bytes.len() as u64,
        max_lines: 4,
        max_metadata_bytes: 4 * size_of::<usize>() as u64,
    };
    let source = SourceText::index(bytes, exact).expect("Exact");
    assert_eq!(source.metadata_bytes(), exact.max_metadata_bytes);
    for (budget, resource) in [
        (
            SourceBudget {
                max_source_bytes: exact.max_source_bytes - 1,
                ..exact
            },
            SourceResource::SourceBytes,
        ),
        (
            SourceBudget {
                max_lines: 3,
                ..exact
            },
            SourceResource::Lines,
        ),
        (
            SourceBudget {
                max_metadata_bytes: exact.max_metadata_bytes - 1,
                ..exact
            },
            SourceResource::MetadataBytes,
        ),
    ] {
        assert_eq!(
            SourceText::index(bytes, budget).expect_err("Budget"),
            SourceError::BudgetExceeded(resource)
        );
    }
    let empty = SourceText::index(
        b"",
        SourceBudget {
            max_source_bytes: 0,
            max_lines: 1,
            max_metadata_bytes: size_of::<usize>() as u64,
        },
    )
    .expect("Empty exact");
    assert_eq!(empty.line_count(), 1);
}

#[test]
fn invalid_offsets_lines_and_spans_fail_without_clamping() {
    let source = SourceText::index(b"a\n", SourceBudget::default()).expect("Source");
    for offset in [3, u64::MAX] {
        assert_eq!(source.location(offset), Err(SourceError::InvalidOffset));
    }
    for line in [0, 3, u64::MAX] {
        assert_eq!(
            source.line(line).expect_err("Invalid line"),
            SourceError::InvalidLine
        );
    }
    for (start, end) in [(2, 1), (0, 3), (u64::MAX, u64::MAX)] {
        assert_eq!(source.span(start..end), Err(SourceError::InvalidSpan));
    }
    assert_eq!(source.bytes(), b"a\n");
}

#[test]
fn indexed_lookup_matches_a_simple_scan_for_every_offset_of_many_byte_patterns() {
    let mut state = 0x61c8_72d9_u32;
    for length in 0..256 {
        let bytes: Vec<u8> = (0..length)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                match state % 6 {
                    0 => b'\n',
                    1 => b'\r',
                    2 => 0,
                    _ => state.to_le_bytes()[0],
                }
            })
            .collect();
        let source = SourceText::index(&bytes, SourceBudget::default()).expect("Index");
        let mut line = 1;
        let mut column = 1;
        for offset in 0..=length {
            let actual = source.location(offset as u64).expect("Location");
            assert_eq!((actual.line, actual.byte_column), (line, column));
            if bytes.get(offset) == Some(&b'\n') {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
    }
}
