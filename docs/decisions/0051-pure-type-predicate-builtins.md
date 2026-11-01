# ADR 0051: Pure type-predicate builtin calls

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

Bound primitive modules can call local script functions but no builtin library.
The pinned C implementations of isint, isfloat and isvector require exactly one
argument and return an integer boolean from the original type tag. They need no
game state, filesystem, network, native plugin, object heap or scheduler. The
compiler checks the local script-function table before builtin lookup. Disabled
developer definitions are skipped in its initial table, while enabled developer
definitions retain ordinary-to-developer reference restrictions.

## Decision

Add a sealed typed builtin subset containing those three predicates. Keep it in
the script execution layer; it is not a plugin registry or general host callback
interface. A typed builtin instruction contains its declared supplied argument
count. Proof checks reachable stack effects without evaluating the predicate or
rejecting a skipped wrong-arity call. All argument expressions execute in existing
rightmost-first order before runtime arity checking. Exactly one copied value is
consumed for successful calls; all primitive categories, including opaque function
references, can be inspected without dereferencing them. Wrong counts fail with
fixed ArgumentCount at the caller instruction. Reuse global fuel, operand count/
byte admission and private failure cleanup; no argument array, callee frame, heap
allocation, per-call budget reset or native 255-argument ceiling is introduced.

Bound source resolves active local script names first, with existing canonical
identifier/developer rules; only an absent local name can resolve to this sealed
builtin set. Unknown names, external paths, receivers, threaded calls, and builtin
function-reference literals stay rejected. Local definitions may shadow builtins
and remain valid ::function targets. Disabled developer declarations do not shadow
builtins. Isolated source profiles stay unchanged. Source maps and compiler work/
identifier/per-call argument/global instruction/metadata quotas remain required.
Explicit admitted typed pure instructions are usable without module/host authority.

## Evidence and limits

Verify new source files against the pinned Git blobs before adding canonical LF
hashes. Execute extracted original C predicate functions under stubbed argument/
type/result/error services at O0/O2. Exercise every pinned type and representative
zero/one/extra counts, plus wide declared counts; distinguish helper tag coverage
from the smaller currently representable Rust value set. Guard selected source
registration/arity/type tags and compiler lookup/developer roles with in-memory
fault injection. Retain source/kernel arity/error order, shadowing/case/developer,
maps/privacy, exact count/byte/fuel budgets, skipped calls, malformed proof and
wide/deep/hostile/recovery tests, plus all existing workspace gates.

This adds three usable pure builtins, not the full standard library. Strings,
arrays/objects/entities, isdefined's pointer/lifetime semantics, general builtin
signatures/services, scheduler, native compiler/VM/unwind/mod and Linux remain
open. Future host services require explicit capabilities and separate ownership/
budgets; unknown calls must never gain ambient authority. No dependencies,
speculative crates or alternate target directory are added.
