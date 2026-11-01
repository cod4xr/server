# ADR 0048: Excess script arguments fail at invocation

Date: 2026-10-31

Status: accepted; supersedes ADR 0047's static excess-argument rejection only.

## Context

The new primitive module verifier rejects excess arguments even in skipped call
sites. The pinned script compiler emits parameter safe-set instructions followed
by OP_checkclearparams before the body, and the VM reports excess arguments there.
The existing ADR 0036 source guards tie those compiler/VM roles to the locked
source. Rejecting excess counts during proof would reject otherwise usable source
with dynamically skipped bad calls and move errors before argument evaluation.

## Decision

Retain complete target/stack/flow verification for every typed call, including
unreachable code. Let Call's supplied count describe its stack operands, not a
static source-signature validity gate. Check excess count at execution, after
caller argument instructions and the call instruction's global fuel admission,
before callee frame allocation/copies/body execution. Check against parameters,
never total locals. Missing parameter suffixes remain initialized Undefined.
Entry-point excess arguments retain their existing early admission error.

Calls on unselected paths never produce an excess-argument error. Reachable calls
still need all declared operands under proof; huge unreachable counts cannot
allocate or run. Reuse fixed ArgumentCount errors at caller function/instruction,
global work/setup accounting, private failed cleanup and all storage/fuel limits.
No extra instruction variant, mutable proof, native marker, error allocation or
new dependency is required. The previous static-only policy is unnecessary for
stack safety once runtime count validation precedes parameter writes.

## Validation and limits

Test skipped and unreachable excess calls, reachable huge-count stack underflow,
runtime count checks even when extra values fit nonparameter locals, argument
failure before count errors, count failure before callee body errors/allocation,
locations/privacy, fresh recovery and hostile inventories. Preserve wide arguments,
missing suffixes, all quota and existing kernel/source tests. Regenerate selected
compiler/VM argument source guards and exercise their mutation checks.
The executable guard driver also tests reordered roles whose original needles
remain present, distinguishing timing checks from mere text-presence checks.
These roles do not execute the original compiler/VM or prove native parameter
safe-set side effects, error indices, stack cleanup/unwinding or full GSC parity.
Source call compilation/linking remains separate work.
