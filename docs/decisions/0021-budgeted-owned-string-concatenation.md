# ADR 0021: Budgeted owned plain-string concatenation

Status: accepted, 2026-10-31.

## Problem

The pinned Scr_EvalPlus uses an 8192-byte stack array and rejects combined
plain-string payloads of 8191 bytes or more, including cases that available memory
could support. Reproducing that scratch limit would conflict with the engine's
capacity requirements. Runtime interning, references, and heap ownership are not
yet implemented; a byte operation must not silently choose their design.

## Decision

LegacyString::concatenate combines ordinary decoded strings into a separately
owned value. It checks addition, admits complete logical result bytes under a
caller-supplied u64 budget, and reserves fallibly before copying. Failure leaves
inputs untouched and exposes no partial result. It preserves byte order without
UTF-8 conversion, terminators, normalization, or the native scratch ceiling.
Localized operands return a typed error. Mixed numeric/vector conversion remains
separate work, as do interning, string equality, references, and VM instructions.

The result budget excludes input storage, spare capacity, allocator overhead, and
other live or temporary values. A future runtime must account for those separately;
this operation does not establish a collector or unlimited script memory. The
existing literal-value module owns this concrete operation until an implemented
runtime ownership boundary justifies moving or replacing its representation.

## Alternatives and consequences

Keeping the scratch array retains an unwanted capacity ceiling. Unchecked Vec
growth loses actionable admission failures. Emulating native refcount callbacks
would tie owned bytes to an unimplemented interner. A speculative heap/value crate
would add abstractions without execution semantics. None is adopted.

Existing representable results retain their bytes. Admitted larger results are an
intentional change from native errors, not a claim of exact error parity. This
server-side value helper changes no client protocol or wire string capacity.
The archival tools disassembly permits an 8191-byte payload where the active C
helper rejects it. The fixture cutoff describes that C helper, not every historic
server binary; the expanded result admits either case under its explicit budget.

## Validation and migration

The generator extracts Scr_CastWeakerStringPair and Scr_EvalPlus from hash-verified
source. String/ref services are stubs; mixed-kind services abort. It records 108
synthetic cases: 54 matching native successes and 54 native capacity failures that
Rust intentionally admits. Native ref callbacks are checked only in that harness;
native VM error unwinding and interning are not reproduced.

Tests compare complete checksums and exact byte lengths up to 200,000 bytes, exact
and one-less budgets, unchanged inputs, empty/self operands, non-UTF-8 bytes,
localized rejection, and content-redacted errors/Debug. No proprietary scripts
are fixtures. Full legacy execution and aggregate runtime accounting remain open.
