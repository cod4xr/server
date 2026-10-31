// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source-grounded script semantics shared by future server/client runtimes.
//!
//! Current scope is legacy numeric/wait primitives, source views, lexing, literals,
//! structural/operator views, typed declarations, local binding, and a verified
//! scalar instruction kernel with restricted scalar source compilation. Full GSC
//! compilation/VM integration, modern
//! language features, scheduling, objects, and host services remain required work.
//! Legacy representations do not cap those future runtime resources.

pub mod binding;
pub mod compilation;
pub mod declaration;
pub mod execution;
pub mod expression;
pub mod legacy;
pub mod lexical;
pub mod literal;
pub mod source;
pub mod syntax;
