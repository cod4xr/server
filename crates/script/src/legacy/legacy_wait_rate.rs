// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::legacy::{LegacyNumber, LegacyWaitError};

/// Legacy wait conversion under an explicit nearest-even binary32 profile.
///
/// This matches the source's SSE conversion when its rounding environment is set
/// to nearest, as the Windows bootstrap requests. It does not change the host FP
/// environment, schedule threads, or inherit the source's masked 24-bit clock.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyWaitRate {
    frames_per_second: u32,
}

impl LegacyWaitRate {
    /// Validates an explicit source-compatible frame rate.
    ///
    /// # Errors
    /// Rejects zero and rates outside the source's positive signed-integer domain.
    pub fn try_new(frames_per_second: u32) -> Result<Self, LegacyWaitError> {
        if frames_per_second == 0 || frames_per_second > i32::MAX as u32 {
            return Err(LegacyWaitError::InvalidFrameRate);
        }
        Ok(Self { frames_per_second })
    }

    /// Returns the validated rate, without implying an engine-global tick limit.
    #[must_use]
    pub fn frames_per_second(self) -> u32 {
        self.frames_per_second
    }

    /// Converts numeric seconds to relative ticks; exact zero remains zero.
    ///
    /// Float multiplication is binary32 and conversion rounds ties to even. A
    /// nonzero positive input rounded/underflowed to zero becomes one tick. Integer
    /// seconds multiply exactly. Valid results widen to u64 for a future wide clock.
    /// This does not decide same-frame versus next-frame resume ordering.
    ///
    /// # Errors
    /// Rejects negative/non-finite durations and conversions outside the source's
    /// valid signed-integer result domain instead of reproducing C overflow or
    /// invalid SSE conversion sentinels. Modern wait semantics remain separate.
    pub fn wait_ticks(self, seconds: LegacyNumber) -> Result<u64, LegacyWaitError> {
        match seconds {
            LegacyNumber::Integer(seconds) => {
                let seconds = u32::try_from(seconds).map_err(|_| LegacyWaitError::Negative)?;
                let ticks = u64::from(seconds) * u64::from(self.frames_per_second);
                if ticks > i32::MAX as u64 {
                    return Err(LegacyWaitError::OutOfRange);
                }
                Ok(ticks)
            }
            LegacyNumber::Float(seconds) => self.float_ticks(seconds),
        }
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "The legacy expression converts its integer rate to binary32 before multiplication."
    )]
    fn float_ticks(self, seconds: f32) -> Result<u64, LegacyWaitError> {
        if !seconds.is_finite() {
            return Err(LegacyWaitError::NonFinite);
        }
        if seconds < 0.0 {
            return Err(LegacyWaitError::Negative);
        }
        let rounded = (self.frames_per_second as f32 * seconds).round_ties_even();
        // 2^31 is exact in binary32; casting i32::MAX to f32 would round up to it.
        if rounded >= 2_147_483_648.0 {
            return Err(LegacyWaitError::OutOfRange);
        }
        let ticks = rounded_ticks(rounded);
        Ok(if ticks == 0 && seconds != 0.0 {
            1
        } else {
            ticks
        })
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "Caller proves a finite, nonnegative, integral value strictly below 2^31."
)]
fn rounded_ticks(value: f32) -> u64 {
    value as u64
}
