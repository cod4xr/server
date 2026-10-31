// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded immutable source views and byte locations for future frontend/tooling.

mod source_budget;
mod source_error;
mod source_line;
mod source_location;
mod source_text;

pub use source_budget::SourceBudget;
pub use source_error::{SourceError, SourceResource};
pub use source_line::{LineEnding, SourceLine};
pub use source_location::SourceLocation;
pub use source_text::SourceText;
