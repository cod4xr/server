# ADR 0034: Scalar plain-name compound assignments

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Decision

Extend the explicit scalar-local statement profile with the ten compound operators
already preserved by guarded grammar rules 73..82. Admit only initialized plain
local-name targets. Read the prior target value, lower the right expression using
the existing numeric/lazy rules, apply the shared infix instruction, and store
through the existing checked assignment/initialization path. Reuse name visits,
task/code/binding budgets and immutable source maps; error operations map to the
original compound token. No new local is created by a compound read.

This profile has no side-effecting RHS calls/assignments or object/index targets;
full native reference/RHS evaluation-order behavior is not inferred. Scalar numeric
coercion, float-result division, wrapping integers, masked shifts and controlled
errors keep their existing evidence/assumptions. Richer operator overloads remain
unsupported. Postfix ++/-- remains separate: the pinned VM research requires an
integer and cannot be replaced by float-capable addition/subtraction.

No new instruction ABI, capability, dependency, source activation, control-flow
binding, native arity policy, full GSC/VM or stock-mod acceptance is introduced.
The pure-return profile keeps its accepted syntax/validation behavior.

## Evidence

Keep existing source/kernel tests. Test all compound operators, mixed and boundary
numbers, target/RHS aliasing, parameter reassignment, uninitialized/non-name target
rejections, runtime type/numeric failures, original compound source spans, and
exact binding/name/compiler budgets. Regenerate guarded compound/opcode profiles
and existing arithmetic reference vectors; neither establishes whole VM execution.
