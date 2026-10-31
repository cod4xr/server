# ADR 0035: Guarded integer-only postfix updates

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context and decision

Native grammar rules 71/72 produce update tags 0x29/0x2a. The compiler dispatch
emits OP_inc/OP_dec and a store; the pinned VM checks VAR_INTEGER before adding/
subtracting one 32-bit word. Float-capable arithmetic would therefore widen the
legacy operation incorrectly. Pin the VM assembly hash and guard this source path
without claiming to execute its whole interpreter or error unwinding.

Add an explicit LegacyIntegerUpdate value operation. Require an integer, reuse
the established wrapping arithmetic kernel with integer one, and retain typed
nonnumeric errors without operand values. Add a typed unary kernel instruction
with ordinary stack-proof/fuel/error behavior; it has no native/serialized ABI.

Extend the scalar-local source profile with standalone initialized plain-name
postfix statements. Load the prior slot, apply the integer-only update, and use
the existing checked store/initialization task. Preserve source update-token spans,
name/work/task/code budgets, argument isolation, and uninitialized/non-name target
failures. This does not add postfix expressions, object/index reference behavior,
general control flow, native arity, calls, heap, scheduling, or module activation.
Modern numeric-update rules remain a separately versioned language decision.

## Evidence

Source checks tie grammar tags to compiler opcodes, header types and VM dispatch/
type checks/word instructions. The VM dispatch inventory ends at 0x86 while the
header also enumerates debugger opcodes 0x87..0x89; do not infer complete opcode/
profile equivalence. Existing arithmetic fixtures ground the reused wrapping
primitive; new boundary/type, kernel stack/fuel, and actual source execution tests
cover this update path. Source profiles do not establish original VM execution,
full native reference/error semantics, or stock-mod interoperability. Keep the
normal target directory and all existing source/kernel regression checks.
