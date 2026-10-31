// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub(crate) struct TokenBucket {
    tokens: u32,
    capacity: u32,
    refill: Duration,
    last_refill: Duration,
}

impl TokenBucket {
    pub(crate) fn new(capacity: u32, refill: Duration, now: Duration) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill,
            last_refill: now,
        }
    }

    pub(crate) fn take(&mut self, now: Duration) -> bool {
        let Some(elapsed) = now.checked_sub(self.last_refill) else {
            return false;
        };
        let gained = elapsed.as_nanos() / self.refill.as_nanos();
        if gained != 0 {
            // Once full, all old refill credit is discarded, retaining fractional time.
            let remainder = elapsed.as_nanos() % self.refill.as_nanos();
            let Ok(remainder) = u64::try_from(remainder) else {
                return false;
            };
            self.last_refill = now.saturating_sub(Duration::from_nanos(remainder));
            let gained = u32::try_from(gained).unwrap_or(self.capacity);
            self.tokens = self.tokens.saturating_add(gained).min(self.capacity);
        }
        if self.tokens == 0 {
            return false;
        }
        self.tokens -= 1;
        true
    }
}
