// SPDX-License-Identifier: AGPL-3.0-or-later
//! Vector equality and mixed-add policy remain distinct from scalar conventions.
#![allow(
    clippy::expect_used,
    reason = "Tests supply finite typed vector domains and expected failures."
)]

use cod4x_script::legacy::{
    LegacyComparison, LegacyNumber, LegacyNumericPair, LegacyVector, LegacyVectorError,
    LegacyVectorOperand, LegacyVectorOperation, LegacyVectorPair,
};

fn vectors(left: [f32; 3], right: [f32; 3]) -> LegacyVectorPair {
    LegacyVectorPair::coerce(
        LegacyVectorOperand::Vector(LegacyVector::from_components(left)),
        LegacyVectorOperand::Vector(LegacyVector::from_components(right)),
    )
    .expect("Vector pair")
}

#[test]
fn vector_casts_preserve_bits_and_exact_equality_differs_from_scalar_tolerance() {
    let value = LegacyVector::from_numbers([
        LegacyNumber::Integer(16_777_217),
        LegacyNumber::Float(-0.0),
        LegacyNumber::Float(f32::from_bits(0x7f80_0001)),
    ]);
    assert_eq!(
        value.components().map(f32::to_bits),
        [0x4b80_0000, 0x8000_0000, 0x7f80_0001]
    );
    let near = f32::from_bits(1.0_f32.to_bits() + 1);
    assert!(
        !vectors([1.0; 3], [near; 3])
            .compare(LegacyComparison::Equal)
            .expect("Exact vector equality")
    );
    assert!(
        LegacyNumericPair::coerce(LegacyNumber::Float(1.0), LegacyNumber::Float(near))
            .compare(LegacyComparison::Equal)
            .as_bool()
    );
    assert!(
        vectors([f32::INFINITY; 3], [f32::INFINITY; 3])
            .compare(LegacyComparison::Equal)
            .expect("Exact equal infinities")
    );
    assert!(
        !LegacyNumericPair::coerce(
            LegacyNumber::Float(f32::INFINITY),
            LegacyNumber::Float(f32::INFINITY)
        )
        .compare(LegacyComparison::Equal)
        .as_bool()
    );
    assert!(
        vectors([0.0; 3], [-0.0; 3])
            .compare(LegacyComparison::Equal)
            .expect("Equal signed zeros")
    );
    assert!(
        !vectors([f32::NAN; 3], [f32::NAN; 3])
            .compare(LegacyComparison::Equal)
            .expect("NaN inequality")
    );
}

#[test]
fn mixed_addition_scalar_only_ordering_and_signed_zero_division_fail_without_mutation() {
    let original = LegacyVector::from_components([1.0, 2.0, 3.0]);
    for (left, right) in [
        (
            LegacyVectorOperand::Number(LegacyNumber::Integer(2)),
            LegacyVectorOperand::Vector(original),
        ),
        (
            LegacyVectorOperand::Vector(original),
            LegacyVectorOperand::Number(LegacyNumber::Integer(2)),
        ),
    ] {
        let pair = LegacyVectorPair::coerce(left, right).expect("Broadcast pair");
        assert_eq!(
            LegacyVectorOperation::Add
                .evaluate(pair)
                .expect_err("Addition uses a separate coercion policy"),
            LegacyVectorError::UnsupportedMixedAddition
        );
        assert_eq!(
            LegacyVectorOperation::Multiply
                .evaluate(pair)
                .expect("Component-wise multiplication")
                .components()
                .map(f32::to_bits),
            [2.0_f32, 4.0, 6.0].map(f32::to_bits)
        );
        assert!(!format!("{left:?} {right:?} {pair:?}").contains("2.0"));
    }
    for coordinate in 0..3 {
        let mut right = [1.0; 3];
        right[coordinate] = -0.0;
        assert_eq!(
            LegacyVectorOperation::Divide
                .evaluate(vectors([1.0; 3], right))
                .expect_err("Any zero denominator fails"),
            LegacyVectorError::DivisionByZero
        );
    }
    assert_eq!(
        LegacyVectorPair::coerce(
            LegacyVectorOperand::Number(LegacyNumber::Integer(1)),
            LegacyVectorOperand::Number(LegacyNumber::Float(1.0))
        )
        .expect_err("Scalar-only pairs use another API"),
        LegacyVectorError::ExpectedVectorPair
    );
    assert_eq!(
        vectors([1.0; 3], [2.0; 3])
            .compare(LegacyComparison::Less)
            .expect_err("No vector ordering"),
        LegacyVectorError::UnsupportedOrdering
    );
    assert_eq!(
        original.components().map(f32::to_bits),
        [1.0_f32, 2.0, 3.0].map(f32::to_bits)
    );
    assert_eq!(format!("{original:?}"), "LegacyVector(..)");
}
