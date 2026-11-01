# ADR 0038: Scalar while loops with conservative exit facts

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

The scalar-local compiler now lowers ordinary blocks and conditionals, but rejects
loops. The pinned compiler emits a consuming false branch and a backward loop
jump for dynamic while conditions. Its short encoded offsets and fixed local
representations must not constrain owned Rust instruction indices or storage.
The VM also invokes a native infinite-loop checker; that timing/error policy has
not been reproduced by the Rust kernel's deterministic instruction fuel.

## Decision

Extend the existing scalar-local profile with while statements using flat staged
tasks, a private backward instruction label, a consuming numeric false branch,
the existing verified absolute jump, and existing execution fuel. Reevaluate the
condition on every iteration. Retain source keyword spans for branch/back-jump
operations and existing expression spans for condition operations.

Compile/check the condition and body without constant folding, including false
conditions. Validate the body against incoming initialized names for its first
iteration. Restore incoming initialization facts after the loop: body-only
assignments cannot authorize a later read because zero iterations are possible
under this conservative structural policy. Stable slots and initialized Undefined
remain distinct from uninitialized locals. Returning bodies still leave the
zero-iteration exit reachable. Do not infer constant-true reachability or native
initialization/scope/error-order equivalence from this policy.

Snapshot payload/visits, staged task metadata/work, code/maps, and independently
verified kernel execution retain existing admission. No fabricated pre-execution
or native runtime dependency is introduced. Backward targets must identify
already emitted real instructions and all targets are independently verified.
For, break/continue, switch, calls/objects and scheduling remain separate work.

## Evidence required

Guard selected grammar/compiler/header/VM branch and jump roles against the pinned
source. These checks do not execute original routines or prove the native loop
watchdog, constant folding, scope, diagnostics or complete VM semantics.
Test zero/many iterations, mutation and condition reevaluation, nested branches/
loops, returning bodies, undefined/type/operator source maps, short-circuit
conditions, deterministic fuel exhaustion, caller isolation and fresh invocation.
Reject first-iteration/body-only reads and check source even when conditions are
constant false. Exercise exact/one-less compiler limits, nested snapshots, deep
flat lowering/failure cleanup and wide backward targets.
