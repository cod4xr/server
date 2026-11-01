# ADR 0084: Compact checked script suite declarations

Date: 2026-11-01

Status: accepted. No runtime behavior, test cases or Cargo target count changes.

## Problem

ADR 0026 keeps focused test leaves inside two Cargo suites to avoid multiplying
compile/link units. Every same-name root leaf also carried a redundant path
attribute. As coverage grew, the frontend/legacy entry files reached 303/275
lines, despite containing only declarations. More test coverage should not push
those entry files toward the source-size ceiling or require more test binaries.

## Decision

Use ordinary Rust `mod name;` declarations for same-name root leaf files.
Keep explicit path attributes where nested leaf paths need them. Preserve all
leaf files, module names, helper folders, filters, lint scopes, and the same two
explicit Cargo test targets with automatic test discovery disabled.

Extend the registration guard to parse a small module-only suite grammar rather
than depending on redundant attributes. Permit line comments, blank lines,
ordinary declarations and canonical matching relative leaf paths. Root helpers
resolve to separate module folders and cannot hide tests. Reject missing or
ambiguous file/folder modules, unexpected root code/attributes/macros, dangling
or repeated path attributes, duplicate module identities or leaf registrations,
unregistered/extra/gated leaves and mismatched/traversing/absolute paths. Check
Windows drive syntax on every host, and reject duplicate Cargo target entries
even if their set of names would otherwise match.

The bounded grammar is a repository policy, not a general Rust parser or a
filesystem security sandbox. Cargo/rustc still validate Rust semantics. Nested
test leaves must remain explicitly registered in one suite; helpers cannot
silently introduce another test-discovery mechanism. Do not replace these roots
with generated discovery, include macros, blanket lint suppression, or a large
combined implementation file.

## Alternatives and consequences

Keeping every redundant attribute eventually consumes the file-size allowance.
Additional Cargo suites/binaries would add compile/link units. Recursive discovery
or delegated unverified sub-suites can silently drop or gate cases. Standard Rust
declarations preserve direct ownership and keep the small checked suite format.

Entry files now contain 164 frontend and 147 legacy lines. The guard preserves all
139 frontend and 128 legacy leaf registrations. Root-file helper modules remain
separate folders. Existing explicit same-name paths are accepted for compatibility,
but new same-name root leaves should use ordinary module declarations.

## Validation

Twelve disposable Python regression tests cover valid ordinary/explicit/nested
registrations and helper folders; missing suites/leaves, duplicates within/across
suites and from distinct paths, cfg/ignore/macros/inline code, ambiguous modules,
hidden helper tests, invalid paths/identities, dangling attributes, auto-discovery,
and extra/duplicate Cargo targets. Fixtures use validated temporary directories,
invoke no Cargo builds, and create no target cache. Run
`python tools/test_script_test_targets.py`; both quality CI platforms run it.

Before/after per-suite leaf Counters are identical. Full Windows debug and release
logs also preserve the exact sorted sets of 323 frontend and 236 legacy test
names. All workspace tests, 15 private script units/two doctests, formatting,
denied-warning all-target Clippy/rustdoc, repository policy and diff checks pass.
Policy checks 942 sources/eight crates, with the largest file back at 287 lines.
Linux/hosted CI remains unrun. The normal target directory is reused, and no
stale artifact deletion or build-time/performance claim is made.
