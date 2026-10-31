// SPDX-License-Identifier: AGPL-3.0-or-later
//! Defined wait domains are explicit; old masked clock limits do not enter storage.
#![allow(
    clippy::expect_used,
    reason = "Tests construct known valid frame rates."
)]

use cod4x_script::legacy::{LegacyNumber, LegacyWaitError, LegacyWaitRate};

#[test]
fn nearest_even_positive_minimum_and_exact_zero_are_separate_rules() {
    let rate = LegacyWaitRate::try_new(20).expect("20 Hz");
    assert_eq!(rate.frames_per_second(), 20);
    for value in [0.0, -0.0] {
        assert_eq!(rate.wait_ticks(LegacyNumber::Float(value)), Ok(0));
    }
    for value in [f32::from_bits(1), 0.001, 0.025] {
        assert_eq!(rate.wait_ticks(LegacyNumber::Float(value)), Ok(1));
    }
    assert_eq!(rate.wait_ticks(LegacyNumber::Float(0.125)), Ok(2)); // 2.5 -> even 2
    assert_eq!(rate.wait_ticks(LegacyNumber::Float(0.175)), Ok(4)); // 3.5 -> even 4
    assert_eq!(rate.wait_ticks(LegacyNumber::Integer(0)), Ok(0));
    assert_eq!(rate.wait_ticks(LegacyNumber::Integer(1)), Ok(20));
}

#[test]
fn invalid_rates_nonfinite_inputs_and_undefined_source_conversions_are_rejected() {
    for fps in [0, 0x8000_0000, u32::MAX] {
        assert_eq!(
            LegacyWaitRate::try_new(fps),
            Err(LegacyWaitError::InvalidFrameRate)
        );
    }
    let rate = LegacyWaitRate::try_new(1).expect("Rate");
    for value in [
        f32::NAN,
        f32::from_bits(0x7f80_0001),
        f32::INFINITY,
        f32::NEG_INFINITY,
    ] {
        assert_eq!(
            rate.wait_ticks(LegacyNumber::Float(value)),
            Err(LegacyWaitError::NonFinite)
        );
    }
    assert_eq!(
        rate.wait_ticks(LegacyNumber::Integer(-1)),
        Err(LegacyWaitError::Negative)
    );
    assert_eq!(
        rate.wait_ticks(LegacyNumber::Float(-f32::from_bits(1))),
        Err(LegacyWaitError::Negative)
    );
    assert_eq!(
        rate.wait_ticks(LegacyNumber::Float(f32::from_bits(0x4eff_ffff))),
        Ok(2_147_483_520)
    );
    assert_eq!(
        rate.wait_ticks(LegacyNumber::Float(2_147_483_648.0)),
        Err(LegacyWaitError::OutOfRange)
    );
    assert_eq!(
        rate.wait_ticks(LegacyNumber::Integer(i32::MAX)),
        Ok(2_147_483_647)
    );
    let rate = LegacyWaitRate::try_new(20).expect("Rate");
    assert_eq!(
        rate.wait_ticks(LegacyNumber::Integer(i32::MAX)),
        Err(LegacyWaitError::OutOfRange)
    );
    assert_eq!(
        rate.wait_ticks(LegacyNumber::Float(f32::MAX)),
        Err(LegacyWaitError::OutOfRange)
    );
}

#[test]
fn wide_relative_results_do_not_wrap_through_the_old_24_bit_clock() {
    let rate = LegacyWaitRate::try_new(20).expect("Rate");
    assert_eq!(
        rate.wait_ticks(LegacyNumber::Integer(1_000_000)),
        Ok(20_000_000)
    );
}
