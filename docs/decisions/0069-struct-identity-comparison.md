# ADR 0069: Checked ordinary struct identity comparisons

Date: 2026-11-01

Status: implemented for ordinary struct equality and inequality.

## Decision

Handle two ordinary structs through the explicit object-aware heap boundary.
Validate both owner/slot/generation references, then compare identities for Equal
or NotEqual. Preserve alias identity across locals, fields, direct/indirect calls,
cycles and admitted collection. Distinct objects with equal contents remain
distinct. Do not use KernelValue's derived data equality to implement language
operators, recurse through fields or expose IDs. Ordered and mixed comparisons
remain typed errors at the original source operator. Pure/string-only entry points
still reject object arguments, and immutable code rejects object constants.

The comparison prepays two reference checks under shared heap work and requires no
scratch allocation. Standalone two-argument execution additionally pays one store
binding and two entry-reference visits: five logical heap-work steps, independent
of field count. These counters are not CPU timings or a claim of native speed.

## Native evidence and important limit

The oracle executes unchanged extracted Scr_CastWeakerPair, Scr_EvalEquality and
Scr_EvalInequality at O0/O2. All 108 ordinary-pointer outcomes agree across three
IDs, equality/inequality, evaluate on/off and three unrelated child-marker types.
Parent records are modeled ordinary objects; references use checked callbacks.
Other services abort if reached. A narrow GCC warning pragma preserves the native
equality helper's intentional error fallthrough without modifying its body.
This does not execute native allocation, reference cleanup, pointer ABI, GC,
compiler, unwinding or VM behavior.

The selected C and archival equality guards both inspect the CHILD namespace for
their array-marker tests: the assembly offset is 0x80028. Do not reinterpret that
as a parent subtype check or infer array equality/rejection from ordinary struct
outcomes. The corpus uses only Undefined/integer/function child markers and no
array parents. Array semantics remain a separate valid-pool/ownership assessment.
script_object_comparison_reference.py rejects 36 scoped role/order mutations;
selected archival blocks are inspected, not executed by this oracle.

## Verification and consequences

Rust replays all 108 ordinary outcomes through verified code and checked handles.
Tests distinguish two objects with 70,000 identical fields each while charging
exactly five work steps and zero scratch. Every insufficient global work prefix
fails with unchanged storage, and stale/reused/foreign identities reject.
Source tests cover aliases/cycles/fields/direct/indirect calls, pressure collection,
mixed/ordered errors, original spans and private diagnostics. A real process prints
numeric results while suppressing private field names, paths and identities.

No new crate, dependency, target directory, native runtime linkage or object
ordering/implicit truth conversion is introduced. Arrays/COW/indexed lvalues,
entity lifetimes, methods/self, scheduling, imports and full stock/mod acceptance
remain open. This does not close S5 or make the server joinable.

Windows workspace debug/release tests, warnings-denied Clippy/rustdoc, formatting,
repository policy and diff checks pass. Script suites contain 267 frontend/183
legacy tests, six private units and two doctests; all four field/identity process
tests pass. The 108 native helper outcomes, 36 role mutations and 99 structural
replays regenerate. Policy checks 759 source files/eight crates with a largest
source of 287 lines. Linux/hosted CI and native game interoperability remain open.
