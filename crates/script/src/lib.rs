// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source-grounded script semantics shared by future server/client runtimes.
//!
//! Current scope is legacy numeric/wait primitives, source views, lexing, literals,
//! structural/operator views, typed declarations, and local function binding, not a compiler or VM.
//! Modern language features, scheduling, objects, bytecode, and host services remain
//! separate required implementation work; legacy representations do not cap them.

pub mod binding;
pub mod declaration;
pub mod expression;
pub mod legacy;
pub mod lexical;
pub mod literal;
pub mod source;
pub mod syntax;
