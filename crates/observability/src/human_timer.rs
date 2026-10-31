// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::time::Instant;

use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;

pub(crate) struct HumanTimer {
    start: Instant,
}

impl Default for HumanTimer {
    fn default() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl FormatTime for HumanTimer {
    fn format_time(&self, writer: &mut Writer<'_>) -> fmt::Result {
        let elapsed = self.start.elapsed();
        let seconds = elapsed.as_secs();
        write!(
            writer,
            "+{:02}:{:02}:{:02}.{:03}",
            seconds / 3600,
            seconds / 60 % 60,
            seconds % 60,
            elapsed.subsec_millis()
        )
    }
}
