// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;

use crate::token_bucket::TokenBucket;

const MAX_ADDRESSES: usize = 4096;
const ADDRESS_IDLE: Duration = Duration::from_secs(30);

#[derive(Debug)]
struct AddressBudget {
    bucket: TokenBucket,
    last_seen: Duration,
}

#[derive(Debug)]
pub(crate) struct QueryBudget {
    addresses: HashMap<IpAddr, AddressBudget>,
    global: TokenBucket,
    last_time: Duration,
    next_cleanup: Duration,
}

impl Default for QueryBudget {
    fn default() -> Self {
        Self {
            addresses: HashMap::new(),
            global: TokenBucket::new(100, Duration::from_millis(10), Duration::ZERO),
            last_time: Duration::ZERO,
            next_cleanup: Duration::from_secs(1),
        }
    }
}

impl QueryBudget {
    pub(crate) fn allow(&mut self, address: IpAddr, now: Duration) -> bool {
        if now < self.last_time {
            return false;
        }
        self.last_time = now;
        if now >= self.next_cleanup {
            self.addresses
                .retain(|_, entry| now.saturating_sub(entry.last_seen) < ADDRESS_IDLE);
            self.next_cleanup = now.saturating_add(Duration::from_secs(1));
        }
        let address = match address {
            IpAddr::V6(ipv6) => ipv6.to_ipv4_mapped().map_or(address, IpAddr::V4),
            IpAddr::V4(_) => address,
        };
        if let Some(entry) = self.addresses.get_mut(&address) {
            entry.last_seen = now;
            return entry.bucket.take(now) && self.global.take(now);
        }
        if self.addresses.len() >= MAX_ADDRESSES || !self.global.take(now) {
            return false;
        }
        let mut bucket = TokenBucket::new(4, Duration::from_millis(250), now);
        let allowed = bucket.take(now);
        self.addresses.insert(
            address,
            AddressBudget {
                bucket,
                last_seen: now,
            },
        );
        allowed
    }
}
