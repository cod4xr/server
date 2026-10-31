# ADR 0033: Explicit straight-line scalar local source profile

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

The scalar backend now executes admitted parameter/local layouts, while source
compilation still accepts only zero-parameter empty/single-return functions.
Connecting named locals requires checked name identity, initialization ordering,
source maps, ownership, and compiler budgets. Native local binding is more complex
than registering all names as readable Undefined; complete branch/scope/call
semantics remain open.

## Decision

Keep the existing pure-return entry point/profile. Add a separate straight-line
scalar-local source entry point, accepting parameters, immediate name assignments,
empty statements, and an optional final return. Fallthrough returns Undefined.
Reuse existing numeric/undefined/bool expressions, groups, and lazy logical
lowering; allow initialized plain-name reads through the local table. Reject
duplicate parameters under this explicit restricted profile, nonfinal returns,
non-name targets, compound/postfix updates, nested blocks/control flow, calls,
objects, strings, and host features. These policies do not establish whole legacy
compiler acceptance or native diagnostic ordering.

Parameters bind initialized slots in declaration order. A new assignment target
registers an uninitialized slot before its right side is lowered, then initializes
only after lowering that expression and emitting the store. Existing assigned
locals remain initialized during reassignment. Reject unknown/uninitialized reads,
including those in source branches that runtime short circuit would skip. A stored
Undefined remains initialized. Use borrowed LegacyIdentifier identity and retain
the long-name case quirk; never copy/intern names or use native short local IDs.

Use a private fallibly grown standard randomized HashMap with monotonically
assigned wide slots; output/error order never depends on hash iteration. Admit
binding count, visited identifier-byte work, compiler work/task/code storage, and
logical binding-pair payload bytes before growth. Account binding payload alongside
live compiler tasks/pending code and overlapping resolved code/maps. Exclude input
source/CST, hash/vector capacity/control overhead, allocator rounding and RSS;
body indices, literals, kernel proof and invocation have separate scopes. Logical
work quotas do not claim wall-time/aggregate CPU enforcement or exact hash probes.
Failures retain charged compiler work/metadata and sanitized nested failures.

Results own only code/maps/accounting plus an explicit source-profile tag. No
source/tree/name references or module activation escape. Publish local-profile
code after verification, without executing it using fabricated parameter values.
Represent validation execution as optional accounting: the original pure profile
records its bounded validation steps; the local profile explicitly records none.
Runtime numeric/type failures map through the owned instruction spans when invoked.
Exact internal argument binding still needs a separate native arity adapter.

## Alternatives and consequences

Silently widening the original pure profile would change its validation contract.
Treating an unknown name as Undefined would erase compile-time initialization
errors. Marking an assignment initialized before its right side would accept a
first self-read. Pre-executing parameter functions with made-up inputs would reject
valid functions or hide runtime errors. Recursive statement/expression traversal,
native 64-slot tables, and packed short operands are excluded.

This is another explicit restricted profile, not full GSC source/function/module
compilation. Definite assignment across control flow, lexical/developer scopes,
legacy duplicate-parameter/arity policies, calls/frames, objects, events, scheduling,
host services, modern language, and stock-mod acceptance remain required work.

## Evidence

Retain existing pure-profile behavior/source maps/tests. Test parameter/name case,
assignment/reassignment and RHS-before-initialization, initialized Undefined,
short-circuit name checks, final/fallthrough returns, runtime error source maps,
source-independent ownership/privacy, unsupported bodies/targets, every new
compiler/body/kernel budget, wide local indices/names, and iterative lowering.
Source profiles support only the stated policies; no native compiler execution or
full legacy binding equivalence is inferred from them.
