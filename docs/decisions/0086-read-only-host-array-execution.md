# ADR 0086: Read-only execution of host-origin arrays

Date: 2026-11-01

Status: accepted for read-only host-origin arrays. General array ownership,
construction/writes and complete GSC remain open.

## Problem

Managed arrays can travel through VM cells and calls, but scripts cannot inspect
their entries or size. Assessment 0085 supplies native read evidence and a bounded
lifetime argument. General arrays still require the combined ordinary-object,
array and cycle-alias lifetime policy identified in 0071/0075.

## Decision

Reuse Index and Size for checked managed arrays in object-aware execution. All
executable arrays must continue to derive from occupied host argument/game
locations. Their hosts remain live throughout the exclusive context borrow.
Fields still reject Array values, so a read cannot return a borrowed managed
location or reveal a nested ownership edge. Adding an independent array origin
requires another lifetime review before activation.

Keep dispatch, array lookup/size and ownership bookkeeping in separate leaves.
Evaluate keys before receivers using existing lowering; keep operands live until
lookup succeeds, then consume them through the prepaid VM owner registry. Failure
uses the same registry cleanup even with no remaining instruction or heap work.
Reads never create fields, detach data, mutate key order or release host owners.
Stored ordinary objects retain shared identity and supported field operations.

Use the named legacy expression-key converter: strict nonnegative int32 keys
below 8,388,608, or complete checked plain-string bytes. Other tags/ranges return
fixed private errors. Byte keys preserve case, embedded NULs and complete length;
this does not assert native interner equivalence for every byte sequence. Typed
ArrayKey::Index(u64) storage remains independent of the legacy expression domain.
Missing entries return Undefined; stored Undefined still counts toward size.
Convert the wide stored count into the current int32 source result with a checked
conversion and SizeOverflow, never native u16 truncation.

No allocation or collection occurs inside a read. Core validation/hash/probe work
charges the same remaining invocation budget; literal caches, managed owners,
root inventories and collector marks retain their shared scratch ceiling. Return
payloads remain checked strings/localized strings, inline vectors, numbers,
Undefined, ordinary object IDs or opaque function references. Existing indirect
call checks preserve image identity.

## Alternatives and consequences

Waiting for all general array semantics would keep safe host data inaccessible.
Activating constructors/writes at this point would bypass known ordinary-object
lifetime requirements. The selected closed-origin profile gives host integrations
useful reads while keeping those origins and nested storage closed. It needs no
new opcode, crate, dependency, native address adaptation or generic host framework.

This is not general GSC array support. Constructors, writable indexed lvalues,
nested/stored arrays, game replacement, native logical RC/cycle aliases, scheduler
ownership and full stock-mod execution remain open. The offline command does not
fabricate array arguments or a game world. Host users must root returned ordinary
IDs/strings in later invocations under the existing context contract.

## Validation and migration

No wire/code serialization or persistent-format migration. Existing host APIs
and immutable proofs remain usable; previously unsupported reads gain behavior.
Constructor, stored-array, arithmetic and indexed-write rejection tests remain.

The 100-row native read corpus replays 66 current payload reads, six sizes and
nine supported key errors. All 19 helper-only rows are explicitly classified.
The unchanged 180-row pointer-size corpus adds 15 array-size replays backed by
real populated arrays, including 65,535 fields. Native vector payload ABI, nested
results, native freeing/error unwinding and whole-VM behavior are not claimed.
Checked function/vector composition is tested as Rust behavior separately.

Eleven source tests cover arguments/game, direct/indirect/imported/qualified
package calls, key order, missing/Undefined, shared mutation, bad key categories,
foreign/stale storage, function-image rejection and same-context recovery.
Every shorter heap/instruction/setup prefix and exact/one-less frame/scratch
limits restore host owners. Paired collection after operand consumption preserves
returned shared graphs and strings across a callee, including every shorter heap
prefix and overlapping registry/cache/root/mark scratch failures.

Wide tests execute over 70,000 fields and a 70,000-byte NUL-bearing key, retain a
typed u64::MAX key, and run 100,000 reads with two reusable VM slots beside 70,000
other host aliases under 4 KiB scratch. These are correctness/resource checks,
not comparative performance benchmarks. The runnable array_reads example returns
20 from two compiled functions, 14 VM steps, 210 heap work, 576 scratch bytes,
two fields and one remaining host owner in four retained value slots.

Complete Windows formatting, all-target denied-warning Clippy, debug and release
workspace tests, denied-warning rustdoc, policy and diff checks pass. Both build
profiles execute the same 334 frontend/240 legacy tests, preserving all prior
323/236 test names, plus fifteen private script tests/two doctests. Policy checks
957 source files/eight crates, largest 287 lines. Native 100-row O0/O2 verification,
180 unchanged size outcomes, 53 read-role mutations, 99 structural replays and
twelve suite-policy regressions pass. Linux/hosted CI remains unrun. Complete
GSC/S5 and join milestones remain open.
