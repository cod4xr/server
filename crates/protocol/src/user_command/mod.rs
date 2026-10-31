// SPDX-License-Identifier: AGPL-3.0-or-later
//! Legacy client movement/action commands, separate from engine simulation state.
//!
//! Keys reproduce protocol obfuscation, not authenticated encryption. Applications
//! must validate command timing, authorization, and game rules after decoding.

mod command_error;
mod delta_reader;
mod keyed_fields;
mod legacy_user_command;
mod melee_angle;

pub use command_error::UserCommandError;
pub use delta_reader::read_delta;
pub use legacy_user_command::LegacyUserCommand;
