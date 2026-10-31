// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original scalar helpers; nonnumeric/VM services are not exercised or emulated.
#![allow(
    clippy::expect_used,
    reason = "Tests decode known synthetic numeric fixtures."
)]

use cod4x_script::legacy::{LegacyComparison, LegacyNumber, LegacyNumberError, LegacyNumericPair};

fn number(kind: &str, bits: &str) -> Result<LegacyNumber, String> {
    let bits = u32::from_str_radix(bits, 16).map_err(|_| "Invalid scalar fixture bits")?;
    match kind {
        "I" => Ok(LegacyNumber::Integer(i32::from_le_bytes(
            bits.to_le_bytes(),
        ))),
        "F" => Ok(LegacyNumber::Float(f32::from_bits(bits))),
        _ => Err("Unknown scalar fixture type".to_owned()),
    }
}

fn raw(value: LegacyNumber) -> (&'static str, u32) {
    match value {
        LegacyNumber::Integer(value) => ("I", u32::from_le_bytes(value.to_le_bytes())),
        LegacyNumber::Float(value) => ("F", value.to_bits()),
    }
}

#[test]
fn original_coercion_comparison_and_division_match_the_selected_scalar_profile()
-> Result<(), String> {
    let comparisons = [
        LegacyComparison::Equal,
        LegacyComparison::NotEqual,
        LegacyComparison::Less,
        LegacyComparison::Greater,
        LegacyComparison::LessOrEqual,
        LegacyComparison::GreaterOrEqual,
    ];
    let mut count = 0;
    for row in include_str!("fixtures/numeric_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let fields: Vec<_> = row.split_ascii_whitespace().collect();
        let start = if fields[0] == "C" { 2 } else { 1 };
        let pair = LegacyNumericPair::coerce(
            number(fields[start], fields[start + 1])?,
            number(fields[start + 2], fields[start + 3])?,
        );
        match fields[0] {
            "P" => {
                let values = match pair {
                    LegacyNumericPair::Integers(left, right) => {
                        [LegacyNumber::Integer(left), LegacyNumber::Integer(right)]
                    }
                    LegacyNumericPair::Floats(left, right) => {
                        [LegacyNumber::Float(left), LegacyNumber::Float(right)]
                    }
                };
                for (index, value) in values.into_iter().enumerate() {
                    assert_eq!(
                        raw(value),
                        raw(number(fields[5 + index * 2], fields[6 + index * 2])?),
                        "Coercion case {count}"
                    );
                }
            }
            "C" => {
                let index: usize = fields[1].parse().map_err(|_| "Invalid comparison code")?;
                let comparison = comparisons.get(index).ok_or("Unknown comparison code")?;
                let expected = fields[6].parse().map_err(|_| "Invalid boolean result")?;
                assert_eq!(
                    pair.compare(*comparison),
                    LegacyNumber::Integer(expected),
                    "Comparison case {count}"
                );
            }
            "D" => {
                let result = pair.divide();
                if fields[5] == "1" {
                    assert_eq!(
                        result.expect_err("Source division error"),
                        LegacyNumberError::DivisionByZero
                    );
                } else {
                    let LegacyNumber::Float(value) = result.expect("Source division success")
                    else {
                        return Err("Expected binary32 division result".to_owned());
                    };
                    if fields[6] == "nan" {
                        assert!(value.is_nan());
                    } else {
                        assert_eq!(
                            value.to_bits(),
                            u32::from_str_radix(fields[6], 16).expect("Float bits"),
                            "Division case {count}"
                        );
                    }
                }
            }
            _ => return Err("Unknown original scalar fixture operation".to_owned()),
        }
        count += 1;
    }
    assert_eq!(count, 12_416);
    Ok(())
}
