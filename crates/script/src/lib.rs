// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source-grounded script semantics shared by future server/client runtimes.
//!
//! Current scope is legacy numeric/wait primitives, source views, lexing, literals,
//! and structural syntax trees, not a compiler or running VM.
//! Modern language features, scheduling, objects, bytecode, and host services remain
//! separate required implementation work; legacy representations do not cap them.

pub mod legacy;
pub mod lexical;
pub mod literal;
pub mod source;
pub mod syntax;
