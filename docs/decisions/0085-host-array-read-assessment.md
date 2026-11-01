# Assessment 0085: Read-only access to host-managed arrays

Date: 2026-11-01

Status: selected native evidence and a bounded read-profile proposal. Rust array
index/size execution is not activated by this assessment. General source arrays
remain gated by proposal 0071 and native lifetime evidence in ADR 0075.

## New evidence

generate_array_read_vectors.py extracts actual selected lookup/read/size functions
from the fully hash-verified reference. Its 100 traces agree at O0/O2. Normal cases
cover integer/string keys, missing/raw-Undefined entries, int/float/string/localized
payloads, shared ordinary objects, different insertion orders and retained parent
aliases. Separate profiles record opaque function words and nested-array results.
Error processes record callback-entry state and exit before external unwinding.
script_array_read_reference.py rejects 53 scoped role mutations; both tools are
wired into CI, and structural checks invoke the read-role guard.

Pool allocation, lookup services, nonfreeing decrements, string refs, formatting
and error exit are modeled. This is not native allocator/interner/VM/ABI/freeing or
unwind parity. Vector payload reads are excluded on the host ABI: the selected
value helper copies a 32-bit word, while vector callbacks consume a pointer.
No uninitialized upper pointer bytes are read to manufacture a vector oracle.
Vector key-type rejection remains covered without reading a vector pointer.
Function/nested result profiles remain helper-only until their own boundaries work.
The existing 306 array, 210 lifetime and 180 pointer-size outcome corpora stay
unchanged; array subtype rows in the size corpus can support later size replay.

Scr_FindArrayIndex validates the strict integer domain, selects integer/string
lookup without creating a child, consumes a string key and returns the child ID.
Scr_EvalArray evaluates that child, adds the result reference through
Scr_EvalVariable, then releases its array operand. Missing child zero becomes
Undefined. Reads do not detach COW or autovivify fields. Size reads the array's
stored count and consumes the pointer value; ordinary non-array objects still
produce size one. Native write/lvalue error ordering is different and must not be
inferred from this read-only path.

## Bounded origin/lifetime argument

ADR 0081 creates VM array locations only by copying host arguments or existing
owned cells, and moving them through calls/returns. ADR 0083 adds copies of an
explicit host game location. Immutable array constants reject. No current source
constructor or sealed builtin mints an independent array, and struct/array field
APIs reject KernelValue::Array. These closed origins mean every currently
executable array derives from a host-owned location that stays live throughout
its invocation. The context exclusively borrows the store; script cell cleanup
releases only registered VM locations, never the original host location.

This permits a proposed bounded read profile without silently choosing complete
nested ownership. The array data and its ordinary/string graph remain rooted by
the live host owner while a read copies a result and releases its VM operand.
Current stored primitives/shared ordinary objects follow existing value/explicit-
root semantics; no stored nested array owner can escape through a raw copied ID.
Ordinary returned objects remain shared and can use supported field operations.
The array read itself must not mutate or detach its backing data.

This argument does not authorize constructors, independent array-returning
builtins, stored array fields, nested reads/COW, writable indexed lvalues, game
replacement, scheduling or callbacks. Adding another array origin requires a new
lifetime review. In particular, retained ordinary-object/cycle aliases still need
0071/0075's combined logical lifetime work before general source arrays activate.

## Implementation constraints

Reuse Index and Size only for checked managed Array values in object-aware
contexts. Convert indices through the existing named legacy-key profile: strict
integers in 0..8,388,608 and complete checked plain-string bytes, with no float
coercion or localized keys. Wide authoritative ArrayKey::Index storage remains
independent of that compatibility converter; no native string-ID or u16 array-
size capacity may become a Rust storage bound. Size uses the wide stored count
and checked conversion into the current legacy int32 result, never truncation.

Evaluate operands under existing source order, keep them live through successful
lookup, then consume them using ADR 0081 cleanup. Preserve checked result tags,
shared identities, missing Undefined, key order, raw-Undefined counts, complete
global work and overlapping cache/registry/root/mark scratch. Reject unsupported
stored tags, foreign/stale identities and invalid keys with private errors.
Do not expose native addresses or treat derived KernelValue equality as language
comparison. No new opcode, crate, dependency or generic host framework is needed.

## Required acceptance

Replay supported native lookup outcomes with explicit key/string/object identity
normalization and account every helper-only row. Keep size outcome rows unchanged
while distinguishing native u16 corpus limits from wide Rust extension tests.
Test typed/standalone/source/package reads, evaluation order, strings/vectors/
functions/shared objects as Rust stored values, missing entries, binary/case keys,
invalid key categories/ranges, stale/foreign storage and result-root pressure.
Verify exact/one-less resources and every failed work/instruction/setup prefix,
retained host owners, bounded churn and wide fields/keys/indices. General array
construction/writes/nested fields must retain their rejection tests. Run all
Windows gates and the new native/structural/registration guards before accepting
Rust behavior. No full GSC/S5 or join gate closes from this component.

Current source-only validation passes 100 O0/O2 traces, 53 read mutations, 99
structural replays, 12 suite-policy tests and policy (946 sources/eight crates/
largest 287 lines). The last full Windows Rust gates remain commit 40c5455;
Linux/hosted CI is unrun.
