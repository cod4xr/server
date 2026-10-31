# ADR 0029: Owned verified scalar-program execution kernel

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

Frontend views and numeric helpers do not execute scripts. A compiler needs an
owned validated instruction target with explicit control-flow, stack, allocation,
and work contracts before functions, objects, threads, or host services can be
integrated. Native packed bytecode, short jumps, and fixed stacks must not become
new engine limits. An initial numeric/undefined kernel must not masquerade as a
complete GSC VM or silently invent unsupported value semantics.

## Decision

Implement an internal typed-instruction profile for constants, legacy numeric
operators, bool conversion, stack operations, branches, jumps, and a single-value
return. KernelValue contains only Undefined and LegacyNumber. Undefined can be
returned; numeric/branch operations require numbers. This limited value model
chooses no object/string/closure/heap representation. No source compiler, locals,
function linking, call frames, scheduler, host I/O, or serialized/native bytecode
format is implied by it. Legacy FP-profile assumptions remain explicit.

Verification consumes a borrowed input description and returns an independently
owned immutable VerifiedKernelProgram. Admit instruction count and logical code/
state/worklist storage before fallible allocation. Scan every branch operand,
including unreachable code. Iteratively propagate entry stack heights, rejecting
underflow, mismatched joins, invalid targets, fallthrough past code, excessive
depth, and returns without exactly one value. Unreachable instructions are allowed
after operand validation; no return path is required for a fuel-bounded loop.
Check verification work as scans, reachable-node visits, and edges are processed.

All retained code and temporary state/worklist slots are quota-accounted using
checked wide arithmetic. Logical metadata excludes allocator rounding, source
input storage, and process RSS. State/worklist vectors are temporary; code and
successful usage remain owned. Only this private execution boundary can construct
the proof object; callers cannot mutate verified instructions through its API.

Execution borrows the proof, admits the known maximum stack under count/byte
budgets, reserves fallibly, and checks fuel before each instruction. Zero fuel
fails before stack allocation. Stack access remains checked even behind the
proof. Pure execution returns a value/work/peak record or a typed instruction-
located error; no partial result or external state mutation is exposed. Loops,
numeric errors, unsupported undefined operands, and allocation failures remain
controlled failures. Errors/Debug omit constant and operand contents.
Failures retain charged work and admitted/actual stack peaks so a caller can
account for failed scripts instead of resetting their work budget after errors.

## Alternatives and consequences

Executing raw instructions permits invalid stack/control flow. Native bytecode
reuse imports widths and unsafe layouts. A recursive evaluator cannot provide a
flat verified control-flow target. Choosing GC/object ownership now would conflate
unimplemented language semantics with this concrete scalar boundary. A typed
instruction Vec avoids an invented binary format while leaving future validated
decoders, source maps, wider values, and actual GSC compilation separate.

Fuel counts instructions, not milliseconds; numeric operations are bounded-cost
here. Later strings/collections/host calls need additional work/cancellation
accounting and legacy scheduling/error tests. The kernel neither claims native
error-stack cleanup nor implies a production server, stock-mod support, or client
interoperability. New runtime side effects will require their own ownership policy.

## Evidence

Tests must exercise actual execution, branch paths, balanced loops/fuel, verifier
attacks and joins, unreachable bad operands, every resource boundary, immutable
input ownership, fallible admission, privacy, and deterministic hostile programs.
Route defined source-grounded scalar fixtures through the execution path where
applicable. Verify debug/release behavior and keep full GSC acceptance open.
