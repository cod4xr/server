// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;
use std::ops::Range;

use crate::binding::LegacyIdentifierError;
use crate::execution::KernelError;
use crate::literal::LegacyLiteralError;
use crate::statement::StatementListError;

/// Restricted scalar compiler resource, independent of legacy parser/VM pools.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScalarCompilationResource {
    /// Compiler actions, literal/identifier-byte work, and instruction resolution.
    Steps,
    /// Emitted typed instructions.
    Instructions,
    /// Live iterative lowering frames.
    TaskFrames,
    /// Logical compiler-owned pending/task/code/map metadata.
    MetadataBytes,
    /// Complete bound names, including parameters.
    LocalBindings,
    /// Identifier bytes across name visits.
    IdentifierBytes,
    /// Live initialization snapshot flag payload.
    ControlBytes,
}

/// Fixed scalar compilation category, without source text, values, or names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScalarCompileErrorKind {
    /// This source profile accepts no parameters.
    UnsupportedParameters,
    /// A statement is outside the selected restricted source profile.
    UnsupportedBody,
    /// Strings, vectors, objects, references, calls, and other primaries are unsupported.
    UnsupportedPrimary,
    /// Parentheses must contain exactly one scalar expression in this profile.
    UnsupportedGroupArity,
    /// This profile permits only plain-name local assignment targets.
    UnsupportedLocalTarget,
    /// This restricted profile rejects repeated canonical parameter names.
    DuplicateParameter,
    /// A name is not initialized on every structural path reaching this read.
    UninitializedLocal,
    /// This restricted profile permits return only as the final immediate statement.
    ReturnNotLast,
    /// A prior structural return prevents this source statement from being reached.
    UnreachableStatement,
    /// A private initialization snapshot/accounting invariant did not hold.
    InvalidControlState,
    /// Break requires an active loop in this restricted profile.
    IllegalBreak,
    /// Continue requires an active loop in this restricted profile.
    IllegalContinue,
    /// A private loop destination/context invariant did not hold.
    InvalidLoopContext,
    /// A private local slot/table invariant did not hold.
    InvalidLocalBinding,
    /// Identifier admission failed without retaining its spelling.
    Identifier(LegacyIdentifierError),
    /// Separately admitted immediate body indexing failed.
    BodyList(StatementListError),
    /// Required private source production/child invariants did not hold.
    InvalidStructure,
    /// A compiler-owned branch relocation was unresolved, rebound, or invalid.
    InvalidBranchBinding,
    /// Checked count/storage/address arithmetic overflowed.
    SizeOverflow,
    /// Fallible compiler code/map/task allocation failed.
    AllocationFailed,
    /// A separately admitted literal conversion failed.
    Literal(LegacyLiteralError),
    /// Kernel verification or pure validation execution failed.
    Kernel(KernelError),
    /// A compiler-owned resource was exhausted.
    BudgetExceeded {
        /// Resource being admitted.
        resource: ScalarCompilationResource,
        /// Requested count or bytes at this check.
        requested: u64,
        /// Configured admission budget.
        maximum: u64,
    },
}

/// Failed scalar compilation at an original half-open source byte span.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarCompileError {
    /// Fixed category and sanitized nested failure.
    pub kind: ScalarCompileErrorKind,
    /// Original source bytes associated with the failing stage/operation.
    pub span: Range<u64>,
    /// Charged compiler work, separate from nested kernel work.
    pub work_steps: u64,
    /// Maximum admitted logical compiler metadata, excluding nested kernel scopes.
    pub peak_metadata_bytes: u64,
}

impl fmt::Display for ScalarCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "scalar compilation error at bytes {:?} after {} work steps: {:?}",
            self.span, self.work_steps, self.kind
        )
    }
}

impl Error for ScalarCompileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            ScalarCompileErrorKind::Identifier(error) => Some(error),
            ScalarCompileErrorKind::BodyList(error) => Some(error),
            ScalarCompileErrorKind::Literal(error) => Some(error),
            ScalarCompileErrorKind::Kernel(error) => Some(error),
            _ => None,
        }
    }
}

pub(super) fn at(span: Range<u64>, kind: ScalarCompileErrorKind) -> ScalarCompileError {
    ScalarCompileError {
        kind,
        span,
        work_steps: 0,
        peak_metadata_bytes: 0,
    }
}
