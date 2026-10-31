// SPDX-License-Identifier: AGPL-3.0-or-later
//! Old valid byte results and intentional expansion past the native scratch cap.
#![allow(
    clippy::expect_used,
    reason = "Tests decode known synthetic native string fixtures."
)]

use crate::string_support;

use cod4x_script::literal::{LegacyStringError, LegacyStringKind};

#[test]
fn plain_concatenation_preserves_native_results_and_expands_capacity_explicitly()
-> Result<(), String> {
    let mut parity = 0;
    let mut expanded = 0;
    for row in include_str!("fixtures/string_operation_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let fields: Vec<_> = row.split_ascii_whitespace().collect();
        let seed: usize = fields[0].parse().expect("Seed");
        let left_length: usize = fields[1].parse().expect("Left length");
        let right_length: usize = fields[2].parse().expect("Right length");
        let left_bytes: Vec<_> = (0..left_length)
            .map(|index| b"Ab_1\xff\x80"[(index + seed) % 6])
            .collect();
        let right_bytes: Vec<_> = (0..right_length)
            .map(|index| b"xY-7\xe9"[(index + seed) % 5])
            .collect();
        let left = string_support::decoded(&left_bytes, LegacyStringKind::Plain)?;
        let right = string_support::decoded(&right_bytes, LegacyStringKind::Plain)?;
        let length = (left_length + right_length) as u64;
        let result = left
            .concatenate(&right, length)
            .expect("Exact admitted result");
        assert_eq!(result.kind(), LegacyStringKind::Plain);
        assert_eq!(result.as_bytes().len() as u64, length);
        let hash = result
            .as_bytes()
            .iter()
            .fold(14_695_981_039_346_656_037_u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(1_099_511_628_211)
            });
        assert_eq!(
            hash,
            u64::from_str_radix(fields[5], 16).expect("Complete checksum")
        );
        if fields[3] == "0" {
            assert_eq!(fields[4], fields[5]);
            assert!(length < 8191);
            parity += 1;
        } else {
            assert_eq!(fields[3], "1");
            assert_eq!(fields[4], "~");
            assert!(length >= 8191);
            expanded += 1;
        }
        if length != 0 {
            assert_eq!(
                left.concatenate(&right, length - 1).expect_err("One less"),
                LegacyStringError::BudgetExceeded {
                    requested: length,
                    maximum: length - 1
                }
            );
        }
        assert_eq!(left.as_bytes(), left_bytes);
        assert_eq!(right.as_bytes(), right_bytes);
    }
    assert_eq!((parity, expanded), (54, 54));
    Ok(())
}
