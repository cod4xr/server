// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original numeric functions and SSE nearest-even waits, without VM/ABI claims.
#![allow(clippy::expect_used, reason = "Tests decode known original C results.")]

use cod4x_script::legacy::{LegacyNumber, LegacyNumberError, LegacyWaitError, LegacyWaitRate};

fn number(kind: &str, bits: &str) -> Result<LegacyNumber, &'static str> {
    let bits = u32::from_str_radix(bits, 16).expect("Raw bits");
    match kind {
        "5" => Ok(LegacyNumber::Float(f32::from_bits(bits))),
        "6" => Ok(LegacyNumber::Integer(i32::from_le_bytes(
            bits.to_le_bytes(),
        ))),
        _ => Err("Unknown source numeric tag"),
    }
}

#[test]
fn numeric_truth_negation_and_complement_match_original_functions() {
    let mut count = 0;
    for row in include_str!("fixtures/legacy_vectors.txt")
        .lines()
        .filter(|line| line.starts_with("N "))
    {
        let row: Vec<_> = row.split_ascii_whitespace().collect();
        let value = number(row[1], row[2]).expect("Known numeric tag");
        assert_eq!(value.as_bool(), row[3] == "1");
        assert_eq!(
            value.logical_not(),
            LegacyNumber::Integer(row[4].parse().expect("Negation"))
        );
        let complement = value.bitwise_not();
        if row[5] == "1" {
            assert_eq!(complement, Err(LegacyNumberError::ExpectedInteger));
        } else {
            let bits = u32::from_str_radix(row[6], 16).expect("Complement");
            assert_eq!(
                complement,
                Ok(LegacyNumber::Integer(i32::from_le_bytes(
                    bits.to_le_bytes()
                )))
            );
        }
        count += 1;
    }
    assert_eq!(count, 572);
}

#[test]
fn valid_and_source_rejected_negative_waits_match_original_conversion() {
    let mut count = 0;
    for row in include_str!("fixtures/legacy_vectors.txt")
        .lines()
        .filter(|line| line.starts_with("W "))
    {
        let row: Vec<_> = row.split_ascii_whitespace().collect();
        let rate = LegacyWaitRate::try_new(row[1].parse().expect("Frame rate")).expect("Rate");
        let actual = rate.wait_ticks(number(row[2], row[3]).expect("Known numeric tag"));
        if row[4] == "1" {
            assert_eq!(actual, Err(LegacyWaitError::Negative));
        } else {
            assert_eq!(actual, Ok(row[5].parse().expect("Ticks")), "{row:?}");
        }
        count += 1;
    }
    assert_eq!(count, 255);
}
