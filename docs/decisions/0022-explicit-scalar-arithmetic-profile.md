# ADR 0022: Explicit scalar arithmetic and integer instruction profile

Date: 2026-10-31

Status: accepted; validation recorded separately.

## Context

Legacy scripts need consistent arithmetic before expression compilation and
execution. The active C source uses signed integer operators whose overflow and
invalid shift counts cannot safely define a Rust port. The pinned archival
_scr_variable.asm records ordinary 32-bit ADD/SUB/IMUL, SHL/SAR with CL counts,
and CDQ/IDIV for remainder. These inform an explicit compatibility profile; they
do not establish every compiler build or historic server's behavior.

## Decision

LegacyArithmetic evaluates already-coerced integer/float pairs for addition,
subtraction, and multiplication. Integer results wrap at 32 bits in every build.
Binary32 operations follow the previously selected nearest/SSE profile; arithmetic
NaN payloads, host FP-control independence, and sticky flags remain unasserted.
There is no string/vector conversion inside this scalar API. Division keeps its
existing separate float-result operation.

LegacyIntegerOperator accepts integers only, without numeric pair coercion. It
implements OR/AND/XOR, signed remainder truncated toward zero, wrapping left shift,
and arithmetic right shift. Counts use their low five bits, including negative
and oversized values, matching the recorded instruction profile. These are value
semantics, not restrictions on engine IDs or runtime storage.

Zero remainder denominators return DivisionByZero. MIN%-1 returns the new typed
RemainderOverflow error instead of executing a faulting IDIV or panicking. That
case is an intentional invalid-input improvement, not native error parity. All
operations leave caller values unchanged, allocate nothing, and expose no operand
contents in errors. They do not reproduce native stack cleanup or unwinding.

## Alternatives and consequences

Plain Rust signed operators would retain build-profile dependence or panic with
workspace overflow checks. Applying C operators to overflow/invalid shifts inside
an oracle would claim evidence from undefined behavior. Rejecting every overflow
or masking floats into integers would break the selected legacy scalar profile.
Using checked remainder avoids both denominator and quotient faults while keeping
ordinary signed results. Modern wider numbers remain separately versioned work.

## Evidence

The new generator pins and checks the archival source hash and exact relevant
instruction blocks. Its C harness executes eleven actual source functions with
minimal service stubs. Nonnumeric storage services abort; type/zero errors record
without native unwinding. Int32/binary32 shapes, nearest SSE, FTZ/DAZ off, and no
excess expression precision are asserted.

Of 11,519 cases, 10,701 use original helpers only within defined C domains. The
remaining 818 execute isolated ADD/SUB/IMUL/SHL/SAR instructions after profile
verification. They are explicitly tagged M, not claimed as full original assembly
routine execution. The trapping remainder input is excluded from both oracles and
tested only as the deliberate Rust error policy. GCC O0/O2 outputs must match.

Five Rust tests cover those fixtures, overflow boundaries, count masking/sign
extension, signed remainder, quotient/zero errors, and float type rejection before
denominator checks. Finite results compare exact bits, arithmetic NaNs class only.
No compiler, bytecode, VM, object/reference handling, stock mod, live gameplay,
performance improvement, or complete historic-server matrix follows from this
kernel. The source-only archival profile and actual C evidence stay distinguished.
