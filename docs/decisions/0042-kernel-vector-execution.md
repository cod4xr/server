# ADR 0042: Verified primitive-kernel vector execution

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

Standalone vector helpers now have original-C value/operator evidence. The owned
kernel still accepts only Number/Undefined, and three-element source groups remain
unsupported. Vector execution must retain independent verification, fixed storage
admission, fresh locals, fuel and redaction before source-constructor integration.

## Decision

Extend owned kernel values with inline LegacyVector data and a typed MakeVector
instruction. Its explicit internal stack contract consumes z/y/x in bottom-to-top
order, matching the selected native constructor's reversed value slots. It
validates numeric components in coordinate order and produces a copied triple.
This is not a native/serialized opcode ABI or proof of source
expression evaluation/push order. Keep source three-element group lowering open.

Add vector dispatch to existing arithmetic/division/comparison identities while
retaining scalar-only behavior. Use the established vector pair/operator rules,
including mixed-add rejection and exact equality. Undefined remains outside these
limited numeric operations. Vector conditions, bitwise and integer updates remain
unsupported; native general type-error/unwind fidelity is not inferred. Surface
fixed nested vector failures without coordinates. Locals/arguments/duplicate/
return copy complete inline data; missing parameters remain initialized Undefined.

MakeVector verifies a three-value stack input and one-value output. Existing
instruction/fuel and actual size_of-based code/stack/Option-local admission now
include larger value payloads before allocation. No vector heap/refcount/pool or
extra side allocation is introduced. Derived data equality remains Rust equality,
distinct from GSC comparisons and broadcast. Numeric dispatch belongs in a focused
leaf module rather than growing the opcode/stack controller into a monolith.

The restricted source APIs retain their accepted literal/group syntax. Their
parameterized low-level programs can receive vector arguments through the widened
kernel value domain; this does not establish general source-vector compilation.
Complete values/objects, calls/frames, scheduler, host services and stock-mod
acceptance remain open.

## Evidence required

Replay original construction/arithmetic/comparison fixtures through actual owned
verified programs; keep coercion-only helper rows distinct because the kernel has
no published coercion instruction. Check finite bits, NaN classes, typed errors,
operand/coordinate order, maps/fuel, fresh local/argument isolation, redaction and
unsupported truth/integer/component types. Test verifier underflow/joins, exact/
one-less enlarged stack/local storage, deep/wide operands and hostile typed input.
Preserve source-profile regressions and every Windows workspace gate. Standalone
oracles and kernel tests do not execute native VM/refcount/error-unwind routines.
