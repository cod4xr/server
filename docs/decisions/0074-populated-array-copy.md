# ADR 0074: Staged populated array copy

Date: 2026-11-01

Status: implemented component for current primitive/shared-struct entries;
nested arrays, retained-cycle markers and source/VM ownership remain open.

## Decision

Add ObjectHeap::make_array_unique for managed values from ADRs 0072/0073. Unique
selection checks the value/data/string-store identities and returns Unchanged in
three logical heap-work steps, with no entry/owner scan, allocation or scratch.
Shared selection stages a fresh data node and copied entries, then switches only
the selected location. Its handle stays valid; all other locations retain old data.
Owner counts use checked u64 arithmetic. The explicit empty-only API remains
separate and rejects populated data without discarding content.

Copy every owned key exactly; preserve primitive/string/localized/function values
and ordinary shared struct identity. Walk source siblings and prepend staged
entries, reproducing copy-induced reversal. Repeated unique selection preserves
order. No KernelValue array tag or source-array instruction is added. Entry values
cannot yet contain nested arrays or represent array-containing struct fields.
Host aliases in tests reproduce native owner-count states, not retained cycles.

## Staging and admission

First validate complete source links, canonical keys and referenced values, count
key bytes and select free/append field slots. Admit final parent/field/index/key/
metadata quotas before staging allocations. Prepay logical record cleanup and
publication visits. Own copied keys in a fallibly allocated flat record vector;
reserve destination arenas before publishing. Retained key payload is charged to
final owned-key quotas; temporary record/index payload is simultaneous scratch.
Possible arena moves and every key/hash/comparison/index/overlay visit are metered.

Use a small scratch occupancy set to simulate actual index placements against the
immutable existing table. Reuse Deleted positions without increasing filled count.
Reset and reuse that admitted allocation for the real staged placements. Rebuild
the whole index only for capacity growth or when actual new Empty placements
would exceed half fill. Drop the trial overlay before allocating a rebuilt table;
scratch admits records plus the relevant overlay/table at each phase. A rejected
trial or reserve changes no heap data, counter, location or logical usage; spare
arena capacity may grow. Hash/index internals remain private and toolchain-pinned.

Publish the fully checked parent, index patches/replacement, fields, links and
location/count changes without further allocation, work admission or callbacks.
Cleanup remains iterative/flat and invokes no script code. Strings stay borrowed
and source data stays rooted throughout staging; ordinary struct graphs are never
traversed or cloned during a copy. Explicit collection retains/reclaims their
graphs and strings through the newly published entries as before.

## Alternatives and consequences

Mutating a shared parent would violate GSC alias behavior. Publishing fields one
at a time with public fallible insertion would expose a partially copied target.
Cloning the whole heap/index for every copy would scale with unrelated contents.
Staged records plus bounded sparse bucket patches retain failure atomicity and
work proportional to the selected entries except admitted arena/index growth.
Nested-copy planning and lifecycle cleanup remain a separate, larger extension.

A regression with a two-entry source and 65,532 unrelated fields showed that a
conservative filled+count test requested 2,097,392 scratch bytes on repeated copies
despite reusable tombstones. Actual-placement preflight now passes eight repeated
copies under 1,024 work steps/4,096 scratch bytes, preserving the unrelated data.
This is a checked workload bound, not an overall runtime/latency comparison.

## Evidence

Thirteen integration tests cover values/binary keys/IEEE bits, ordinary identity,
string/function lifetimes and privacy, exact final resources, every failed work
prefix and exact/one-less scratch for empty/patch/rebuild/reuse cases, 70,000 entries,
70,000 owners, a 12,000-struct cycle, bounded 100,000-copy churn and a 16,384-operation
independent version/value/order/ownership/collection model. Two private tests force
scratch-set collisions and actual-fill promotion to a same-capacity rebuild.

All 20 native alias/COW states replay using explicit host-owned aliases, including
the 16 populated states that ADR 0072 could not execute. This does not establish
retained-cycle marker behavior, native allocation/freeing/ABI/compiler/VM/GC parity
or nested-copy compatibility. Original native fixtures remain unchanged.

The runnable array_copy_workload reports component logical work/storage/scratch.
One Windows debug observation copied 70,000 entries into 140,000 total fields,
524,288 buckets, 1,260,000 owned key bytes and 22,948,784 logical metadata bytes;
copy work was 3,069,891 steps/16,788,608 scratch bytes. The 100,000-copy churn
retained two parent/value slots, eight field slots, 16 buckets, 36 key bytes and
1,264 logical metadata bytes; copy work was 23,299,937 steps, share/release/collect
work 8,200,000, and peak operation scratch 736 bytes. Observed elapsed times were
111,334/2,049,550 microseconds. Setup is excluded; these are individual runs,
not native performance, RSS, frame-time or server-load guarantees.

Windows workspace debug/release, all-target Clippy, rustdoc, formatting and policy
passed: 268 frontend, 229 legacy tests, 13 script unit tests and two doctests;
842 source files/eight crates, largest source 287 lines. Original array 306/alias
20 native traces, 14 alias-role mutations and 99 syntax replays passed.
Linux/hosted CI remains unrun locally. Full nested arrays, retained-cycle markers, frame/field/scheduler ownership,
source lvalues and the complete GSC/S5 gate remain open under proposal 0071.
