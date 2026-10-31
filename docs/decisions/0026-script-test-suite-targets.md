# ADR 0026: Explicit script test suites with complete leaf registration

Date: 2026-10-31

Status: accepted; validation tracked separately.

## Context

The script crate had thirty Cargo integration-test targets, one per leaf file.
Frequent shared-library changes make each target a separate compile/link unit,
including release LTO. The user requires reusing the normal target directory and
keeping builds/storage manageable while preserving small, separated source files.
Disabling Cargo's automatic discovery without a completeness check risks silently
dropping newly added tests.

## Decision

Keep leaf files and fixture paths in place. Use two explicit Cargo suites: frontend
for source/lowering/binding and legacy for scalar/string behavior. Each short root
only declares modules; shared test helpers live once within the relevant suite.
Preserve ordinary #[test] cases, existing per-module lint scope, and library doctests.
Cargo still uses the existing normal target directory, pinned toolchain, lockfile,
and workspace lint policy. No new dependency or alternate build cache is added.

Repository policy checks that every file containing plain #[test] cases belongs
to exactly one unconditional suite registration. It rejects missing, duplicate,
extra, gated, or path-mismatched registrations, including cases in subfolders.
Helpers remain separate modules. New test leaves must be registered explicitly.

## Alternatives and consequences

Thirty individual binaries preserve per-file Cargo targets but multiply link units
when the library changes. One combined suite reduces targets further but merges
the useful frontend/value boundary. Large combined source files violate the code
structure contract. Two module-only roots preserve focused ownership and filters.

Use cargo test -p cod4x-script --test frontend -- expression_lists:: for a focused
module, or --test legacy for the value suite. Old standalone leaf target names are
replaced by module-qualified filters. A leaf-only change now rebuilds its suite.
Tests within a suite can run concurrently; they must retain independent state.
Actual build-time/memory improvements require separate measurements and are not
claimed from target counts. Existing stale artifacts are not deleted by this change.

## Evidence

The pre/post Cargo inventories contain the same 81 test leaf names with identical
multiplicities: 63 frontend and 18 legacy tests. All pass in the grouped debug run;
full workspace/release checks remain separately recorded. This inventory includes
the six new expression-list tests. The repository guard must also reject injected
missing/duplicate/gated/extra/nested-unregistered test cases in disposable fixtures.
Future added tests may increase the count; the policy checks completeness rather
than freezing a numeric target. No runtime or protocol behavior changes here.
