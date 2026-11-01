# ADR 0037: Scalar blocks, conditionals and reachable initialization joins

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

Scalar-local source now supports assignments/updates/returns, but rejects ordinary
blocks and conditionals. A single mutable initialized flag per name is insufficient:
then-only assignments must not authorize a later read, an else branch must start
with incoming facts, and a returning branch must not constrain a surviving path.
The native compiler copies fixed-size blocks/initialization bits; its 64-local
representation must not become a new capacity rule.

## Decision

Extend the existing scalar-local source entry point with ordinary blocks and
if/if-else statements. Keep prior valid straight-line syntax/validation behavior.
Maintain one function-local name namespace and source-ordered wide slots, with
separate compile-time reachability/initialization facts. Snapshot only initialized
flags plus reachability, never runtime values or names. New branch names keep
stable slots; a name absent from an incoming snapshot is uninitialized there.

Lower the condition and checked forward branches iteratively. Compile both
syntactic branches, including constant-condition source. Restore incoming facts
before else, then intersect initialization of branches that reach the join. If
only one branch falls through, use only that branch's facts. Returns terminate
their structural path; reject later source statements on an unreachable path.
Return remains final within its immediate statement list. Do not infer native
constant-folding/diagnostic order or every legacy lexical-scope rule from this
restricted scalar policy. Loops/switch/break/continue/developer blocks remain open.

Use owned private forward-jump relocations alongside existing conditional branch
relocations. Bind each once and resolve/verify every target before publication.
Retain original condition-keyword/operation spans; append a real implicit Undefined
return when needed so final conditional targets always identify real instructions.
No placeholder instruction or unverified code is published. Existing numeric/lazy
lowering, assignment/update, invocation and exact/legacy argument policies are shared.

Admit snapshot flag storage/work before allocation or copy/restore/merge. Charge
visits against global compiler work and logical snapshot payload against both a
control-state quota and compiler metadata with checked live-byte accounting.
Task headers are already admitted by live task records; hash/vector capacity,
allocator/local-field overhead, source/CST, body indices and kernel scopes remain
explicitly separate. Track peak admitted control bytes; failures retain charged
work/metadata without publishing partial functions. Flat tasks/snapshots avoid
native recursion and fixed initialization bit masks.

## Consequences and evidence

The pure-return entry point stays restricted and pre-executed under its existing
budget. Scalar-local compilation still verifies without fabricated arguments.
Source blocks/conditionals do not add objects/heap, calls/frames, host services,
developer authority, scheduler, full legacy compiler/VM or stock-mod acceptance.

Test actual source branch selection, nested/empty blocks, direct/else-if returns,
both-path assignment, then-only and else-input failures, returning-path joins,
unreachable source, unchanged prior scalar profiles, source maps and caller
isolation. Cover exact/one-less control/work/task/code/metadata admission, large
branch destinations, wide local snapshots and deep iterative lowering/drop. Source
shape evidence and tests do not execute the original compiler or prove its full
constant-condition/error-order/scope behavior; keep those acceptance gates open.
