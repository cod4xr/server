# ADR 0047: Verified primitive direct calls with bounded owned frames

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

The static excess-argument rejection below is superseded by
[ADR 0048](0048-call-argument-error-timing.md). Targets/stack/flow still verify;
excess arguments now fail at invocation after caller argument evaluation.

## Context

Single primitive functions execute under owned proof/local/stack/fuel limits.
Calls require a complete target inventory and runtime frames; invoking each callee
through the standalone API would reset fuel, hide aggregate storage and recurse
on the host stack. The pinned runtime has 32 function frames and 2,048 value/local
slots. Those capacities cannot become new engine allocation rules. Source call
views and selected missing-argument rules exist, but native receiver/evaluation/
push order, frame cleanup and complete linking remain separate acceptance work.

## Decision

Add an internal typed direct Call instruction carrying a wide function index and
argument count. Its explicit stack convention is bottom-to-top last argument to
first argument; parameter zero is popped first. This convention is not a native
or serialized ABI or a claim about source expression evaluation. A call consumes
the specified argument prefix and returns one primitive value. Missing parameter
suffixes receive initialized Undefined; excess arguments fail verification.

Verify a complete borrowed inventory of function layouts/instructions before
publishing one owned immutable module. Check every target/arity operand, including
unreachable calls, and reuse the existing intraprocedural stack/flow proof. Support
forward calls and recursion without traversing the call graph recursively.
Standalone verification rejects every unresolved call. No partially verified
module escapes. Admit total functions/instructions, global verification work and
overlapping retained-code/module-table/current-verifier payload before allocation;
per-function stack/local limits remain explicit. No source import/name resolution,
module activation, builtin/host authorization or reload is implied.

Execute with an iterative owned frame stack and one global instruction counter.
Share primitive frame stepping between standalone and module execution. Admit
aggregate reserved operand slots/bytes, complete local slots/bytes and frame
payload/count before creating each frame. Bound repeated setup work separately,
including local initialization, argument copies and a conservative reservation/
relocation charge for the live frame table. Account peak payload across all live
frames and release the popped frame's admission on return. Parameters/locals are
private copied primitive values; no temporary padded argument buffer is needed.
No host recursion, native pointer/marker or fixed archival frame capacity enters
the runtime. Failure drops private frames and publishes no partial result or
external mutation. Diagnostics identify function/instruction and resource counts,
never argument values or private names. Logical payload accounting excludes
allocator rounding, vector spare capacity, fixed stack fields and process RSS.

## Alternatives and validation

Recursive standalone execution would evade aggregate fuel/storage and risk host
stack overflow. Native frame structs/capacities would prevent expansion. A generic
host/call/scheduler framework would add unsupported authority and lifetimes.
Implement the concrete primitive direct-call boundary first, with no dependency
or empty crate. Keep proof, inventory admission, frame accounting and execution in
focused modules under an execution calls subfolder.

Test forward/recursive/mutual calls, argument order/defaults/excess, return values,
caller isolation, fresh invocations, malformed and unreachable operands, stack
joins, entry selection, error locations/redaction and complete failed cleanup.
Exercise exact/one-less global work/count/byte limits, repeated shallow calls and
recursion beyond the old 32-frame capacity; wide inventories must exceed 16-bit
indices. Preserve standalone/source/kernel/oracle regression suites. These are
typed runtime tests, not native parser/compiler/VM execution or full GSC/mod
acceptance. Source calls, strings/objects/receivers, indirect calls, scheduling,
native unwind parity and Linux acceptance remain open.
