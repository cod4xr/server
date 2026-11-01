# ADR 0056: Owned code literals and invocation caches

Date: 2026-10-31

Status: accepted for typed code literals/cache; source lowering and CLI output remain open.

## Ownership and decision

Immutable code owns exact decoded plain-string bytes, separately from runtime
StringId values. Add checked StringLiteral operands into per-function owned tables.
Verify all operands, including unreachable code, and admit retained rows/payload,
proof scans/copies and complete module overlap before publication. Empty tables
preserve existing primitive profiles. No native ID, serialized bytecode or global
mutable interner is introduced. Debug and diagnostics omit literal payloads.

Use checked module-local flattened cache offsets assigned during complete proof.
A borrowed heap context prepares a fresh invocation cache, charging initialization
work and logical cache metadata against its shared scratch budget. Lazily load
literals into the caller-owned store, cache each referenced row, and root all active
cached IDs during collection. Cache metadata, temporary root IDs and mark flags
share simultaneous scratch admission. Reusing a context for another invocation
discards the previous cache before new allocation; work never resets. Results and
host handles still require external roots for subsequent calls.

Repeated literal evaluation must not repeatedly copy payload. Partial cache/table
admission cannot activate code. Arguments and external handles are checked before
frame allocation; allocation operands remain rooted until publication. Existing
pure execution can fail at a reached string literal with HeapContextRequired;
it must never silently create/drop a heap or return dangling handles.

## Source, output and validation

Bound-module compilation may adopt decoded plain literal payloads only after exact
size/metadata/work admission, retaining per-token literal budgets. Track literal
tables through pending/resolved/staged/proof overlap without hidden descriptor
arrays. Existing isolated scalar profiles remain explicit. Localized and mixed
coercion need separate evidence. CLI strings require a persistent explicit heap,
escaped byte output and fixed-category/count-only operational logs.

Require owned tables after input drop, invalid/unreachable operands, wide tables,
exact/one-less code and invocation resources, cache reuse/cross-module separation,
pressure collection preserving cached-only roots, repeated literals in loops,
error/fuel paths, source maps/privacy and native plain helper evidence. Full native
interning/cleanup, objects/cycles/scheduling, imports/general services and recorded
stock/mod/platform acceptance remain open. Acceptance must distinguish implemented
typed literals, source construction and CLI output; one does not imply the others.

## Implemented typed evidence

Thirteen literal tests cover exact binary payload ownership after input drop,
all invalid/unreachable/missing-table operands, 70,000 retained rows and wide cache
indices, unused-table proof admission, exact/one-less standalone/module proof and
shared runtime work/scratch, cached-only collection roots, distinct function table
offsets, context reuse across different modules, independent heaps and fuel-failure
reclamation. A 1 MiB literal evaluated 1,001 times produces one live string, zero
collections and exact work of payload bytes plus 2,004 logical steps. This is a
bounded-copy observation, not a comparative performance/RSS benchmark.

Complete module preflight prepays each function's initial and later prefix table
visits; per-function proof separately charges descriptor scans and row/payload
copies. Metadata includes retained literal records/payload alongside code and the
current flow verifier. No hidden unaccounted borrowed descriptor array is built.
Invocation cache initialization and active-row index capacity share the same
scratch ceiling as temporary roots/mark flags. Prior numeric and heap suites remain
required. Source compilation still supplies empty literal tables and rejects quoted
primaries; CLI construction/output and literal-source maps remain open work.
