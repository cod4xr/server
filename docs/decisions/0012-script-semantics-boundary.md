# ADR 0012: Source-verified script semantics without host or VM coupling

Date: 2026-10-31

Status: accepted and implemented; full-workspace validation recorded separately.

## Context

Expanded GSC must retain old script behavior while removing engine allocation/ID
ceilings and adding versioned language/services/tooling. Observable scalar and wait
rules need independent evidence before a new compiler/scheduler can rely on them.
The pinned implementation mixes script semantics with game globals, native unions,
SSE rounding state, masked time, error unwinding, and reference-count side effects.

## Decision

Create cod4x-script as a real dependency-free script-semantics boundary, starting
with implemented/tested legacy numeric truth, logical negation, integer complement,
and explicit-rate wait conversion. Neither executable, content, networking, nor
host services belong below it. This is an implemented kernel, not an empty VM or a
claim that source scripts execute. Add frontend/runtime/host boundaries only when
their concrete implementations justify them.

LegacyNumber represents only the old signed-int/binary32 numeric subset. It is not
the new runtime's complete value/opaque-handle representation or modern type system.
Numeric truth preserves zero/NaN behavior; complement requires an integer. Helpers
return values/typed errors without changing caller stacks or performing native
reference-count/error-unwind effects. Those remain VM responsibilities.

LegacyWaitRate requires a positive source-representable rate. It multiplies float
seconds in binary32, rounds ties to even, and preserves the source's minimum-one
rule for positive nonzero values rounded/underflowed to zero. Exact signed zeros
stay zero. Integer seconds multiply exactly within the defined source result domain.
Valid ticks widen to u64 without imposing the old masked 24-bit clock. This does
not specify resume order, absolute time, thread scheduling, or modern wait syntax.

The original SSE helper follows ambient MXCSR; Windows bootstrap requests nearest
rounding. Test that explicit profile with FTZ/DAZ disabled, rather than claim every
client/platform/plugin-modified FP environment matches. Reject non-finite inputs,
integer overflow, and invalid SSE conversion domains explicitly. Do not reproduce
undefined C multiplication or carry an indefinite negative conversion into a timer.
Modern larger durations/numbers need separate semantics, not changes to this profile.

## Alternatives and migration

Copying native unions/pointers would reintroduce ABI constraints and old IDs. A
generic truthiness rewrite would silently accept types the source rejects. Reusing
an engine-global clock or changing floating precision would couple later scheduling
to unverified semantics. Callers will integrate these tested primitives into a
validated runtime while separately owning stack cleanup, source diagnostics, and
host operations. No dependency or state migration is currently necessary.

## Evidence and limits

Five tests include 827 original-function cases: 572 numeric and 255 wait results.
The oracle extracts Scr_CastBool, Scr_EvalBoolNot, Scr_EvalBoolComplement,
VM_CalcWaitTime, and the actual SSE f2rint helper. Its explicit nearest profile
includes signed zeros, subnormals, tie neighbors, NaNs/infinities for truth, negative
wait errors, large rates with binary32 rounding, and valid conversion boundaries.
Native heap-reference paths are not modeled; error hooks record a failure instead
of longjmp/VM stack unwinding. Undefined source wait products/conversions are excluded
from equality fixtures and tested as Rust rejections separately.

Parser/lexer, full values, bytecode, scheduler, objects, builtins, source maps,
extended types/modules/async services, debugging, and GSC execution remain open.
Neither stock-mod compatibility nor a game-client join follows from these tests.
