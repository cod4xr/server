// SPDX-License-Identifier: AGPL-3.0-or-later
//! Observable legacy float quirks and explicit scalar/error scope.
#![allow(
    clippy::expect_used,
    reason = "Tests exercise exact synthetic scalar domains."
)]

use cod4x_script::legacy::{LegacyComparison, LegacyNumber, LegacyNumberError, LegacyNumericPair};

#[test]
fn equality_uses_binary32_difference_against_the_original_double_threshold() {
    for (bits, equal) in [(0x3586_37bc, 1), (0x3586_37bd, 1), (0x3586_37be, 0)] {
        let pair = LegacyNumericPair::Floats(0.0, f32::from_bits(bits));
        assert_eq!(
            pair.compare(LegacyComparison::Equal),
            LegacyNumber::Integer(equal)
        );
        assert_eq!(
            pair.compare(LegacyComparison::NotEqual),
            LegacyNumber::Integer(1 - equal)
        );
    }
    let pair = LegacyNumericPair::Floats(1.0, f32::from_bits(1.0_f32.to_bits() + 1));
    assert_eq!(
        pair.compare(LegacyComparison::Equal),
        LegacyNumber::Integer(1)
    );
    assert_ne!(
        LegacyNumber::Float(1.0),
        LegacyNumber::Float(f32::from_bits(1.0_f32.to_bits() + 1))
    );
}

#[test]
fn unordered_comparisons_and_equal_infinities_follow_source_predicate_negation() {
    let pair = LegacyNumericPair::Floats(f32::NAN, 0.0);
    for (comparison, result) in [
        (LegacyComparison::Equal, 0),
        (LegacyComparison::NotEqual, 1),
        (LegacyComparison::Less, 0),
        (LegacyComparison::Greater, 0),
        (LegacyComparison::LessOrEqual, 1),
        (LegacyComparison::GreaterOrEqual, 1),
    ] {
        assert_eq!(pair.compare(comparison), LegacyNumber::Integer(result));
    }
    assert_eq!(
        LegacyNumericPair::Floats(f32::INFINITY, f32::INFINITY).compare(LegacyComparison::Equal),
        LegacyNumber::Integer(0)
    );
}

#[test]
fn integer_division_is_float_and_zero_errors_do_not_return_partial_values() -> Result<(), String> {
    for (pair, bits) in [
        (LegacyNumericPair::Integers(7, 2), 3.5_f32.to_bits()),
        (LegacyNumericPair::Integers(i32::MIN, -1), 0x4f00_0000),
        (LegacyNumericPair::Floats(-0.0, 1.0), 0x8000_0000),
    ] {
        let LegacyNumber::Float(value) = pair.divide().expect("Defined division") else {
            return Err("Expected binary32 result".to_owned());
        };
        assert_eq!(value.to_bits(), bits);
    }
    for pair in [
        LegacyNumericPair::Integers(1, 0),
        LegacyNumericPair::Floats(1.0, 0.0),
        LegacyNumericPair::Floats(1.0, -0.0),
    ] {
        let error = pair.divide().expect_err("Zero denominator");
        assert_eq!(error, LegacyNumberError::DivisionByZero);
        assert!(!format!("{pair:?}").contains("1.0"));
    }
    let pair = LegacyNumericPair::coerce(
        LegacyNumber::Integer(16_777_217),
        LegacyNumber::Float(16_777_216.0),
    );
    assert_eq!(
        pair.compare(LegacyComparison::Equal),
        LegacyNumber::Integer(1)
    );
    Ok(())
}
