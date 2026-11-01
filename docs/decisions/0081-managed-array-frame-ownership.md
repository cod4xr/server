# ADR 0081: Managed array ownership across VM frames

Date: 2026-11-01

Status: accepted for the current non-nested transport profile. Source array
operations remain rejected under proposal 0071. This is not full GSC.

## Problem

Typed heap arrays already have managed locations, unique mutation and staged COW
for primitive/shared-struct entries. Raw ArrayValueId copies create no owner.
VM locals, operands, arguments, returns, discarded values and failed frames need
explicit location copies/moves/releases. Retaining all aliases until invocation
exit would preserve dead callee owners and change subsequent COW behavior.

Native lifetime evidence in ADR 0075 also requires ordinary object/field lifetime
work before nested/source arrays. This component keeps the existing non-nested
storage profile: struct fields and array entries reject KernelValue::Array, and
wrong-tag Object references cannot bypass those checks. Ordinary objects remain
explicit-root shared snapshots; this ADR does not supply their native refcounts.

## Decision

Add KernelValue::Array as a redacted, nonowning location snapshot. Object-aware
entry points borrow host array arguments and clone distinct entry-cell locations.
LoadLocal and Duplicate create new owned locations; stores, swaps, call parameters
and returns move them. Replacing/discarding cells releases their old locations.
Calls retain existing legacy missing/excess-argument rules, including rejecting
excess script arguments before callee execution. Successful builtins consume and
release evaluated operands after evaluation; failure leaves ownership tracked.

Direct and indirect calls share a private invocation ownership inventory indexed
by frame depth. Each array-bearing frame has an explicit fixed-key probe table.
It tracks only VM-owned locations, never scans unrelated host values to find dead
cells, and retains popped-but-pending owners when an instruction or setup fails.
Array-location slots supply the internal hash; complete equality still checks
arena/generation. Hashes, every probe, initialization, rehash scans and relocation
are admitted. Host validation precedes entry cloning. All runtime array constants,
including unreachable ones, reject during proof.

A table uses equal logical-length value/tombstone vectors, power-of-two bucket
counts and at most 3/4 fill before staged rebuild/growth. The complete replacement
bucket bytes and initialization are admitted before allocation. Rehashing copies
nonowning snapshots while the old table remains intact; only fully admitted
staging publishes. Logical accounting excludes Vec spare capacity, allocator
rounding, fixed fields and RSS. It includes every retained bucket because cleanup
can scan vacant buckets, plus simultaneous old/new tables, frame rows, literal
cache, discovery roots and heap-operation scratch.

A single pending-owner slot records a newly copied location before fallible
accounting/probe work can fail. It also holds a detached return while the callee's
other owners release. Return target storage and probes are prepared before this
cleanup; no normal resource admission follows cleanup before transfer. A callee
releases other owners before its caller resumes. A successful top-level array
result transfers one owner to the host. Copying KernelValue or an outcome in Rust
creates no owner; the host must explicitly call ObjectHeap::release_array_value.
All occupied managed heap locations already root their data for paired GC.

## Cleanup admission

Prepay five logical units per new owned value (three fixed heap-release steps plus
registry/accounting work), two per frame row, and 2*(buckets+1) per active table
scan. Growth reserves additional scan credit before replacing its table; reused
closed frames reserve a fresh scan credit. Normal membership/probe work draws
from the remaining global heap budget before mutation. Error cleanup performs no
hash lookup: it releases the pending owner and scans frame buckets in reverse
frame order using prepaid credits, without instruction/heap fuel or allocation.
Unused credit is not refunded, so reported heap work includes conservative cleanup
admission rather than measured CPU instructions. Heap payload collection remains
separate. Core release errors or impossible private ledger state poison the context;
no recovery from arbitrary private memory corruption is claimed.

Pure/string-only entry points still reject heap arrays. Existing scalar contexts
allocate no ownership table. No new crate, dependency, opcode, source constructor,
array indexing/size/comparison/store implementation, callback or plugin is added.
The offline runner reserves an opaque array result category, but cannot currently
construct a managed array from source or CLI inputs.

## Alternatives

A standard HashSet prototype passed transport/wide/churn tests, but discovered
rounded bucket capacity only after allocation and hid collision probe work. The
explicit narrow table makes these admission boundaries reviewable. A global heap
scan on each operation, permanently retaining dead aliases, recursive destruction,
and cleanup that borrows remaining fuel are rejected. Per-frame tables retain
high-water bucket storage until invocation end; this is admitted scratch, with no
unrelated-heap dependency. A generic ownership framework or additional crate is
unnecessary. Nested values need separate lifetime/COW decisions.

## Validation

Fifteen new integration tests and two private collision tests cover host/VM copy,
move, overwrite, discard, permutations, direct/indirect returns, predicates and
evaluated extras, missing/excess arguments, runtime errors and context reuse,
foreign/stale handles, immutable constants, unsupported fields and paired pressure
collection. An independent cell model checks 512 generated programs at exact live
admission and one less. Exact/one-less global work/storage and every insufficient
instruction/setup/heap-work prefix restore host owners, including multiple table
growth stages and pending registration. Deep completion/failure uses 12,001 flat
frames; source locals exceed 65,535 owners; overwrite churn runs 100,000 times
beside 70,000 host aliases under 4096 bytes of scratch.

The array_transport_workload example reports logical resource counts and local
elapsed time, not native performance or RSS. One Windows debug run measured:

| Case | VM steps | Heap work incl. prepaid cleanup | Peak context scratch | Retained value slots | Live values after cleanup |
| --- | ---: | ---: | ---: | ---: | ---: |
| 70,000 owned locals | 140,003 | 2,271,797 | 4,915,272 | 70,003 | 1 |
| 100,000 overwrites / 70,000 host aliases | 200,067 | 1,700,999 | 2,472 | 70,036 | 70,001 |

Full Windows formatting, all-target denied-warning Clippy, debug/release workspace
tests, denied-warning rustdoc, repository policy (924 sources/eight crates/largest
287 lines) and diff checks pass. Script suites contain 310 frontend/235 legacy,
15 private units and two doctests. The 210 native lifetime traces still match at
O0/O2; lifetime/include/package guards reject 61/77/38 mutations and syntax guards
pass. Linux/hosted CI remains unrun.

No new native whole-VM/call/unwind oracle is claimed. Existing primitive/object/
array native corpora keep their scoped meanings. General ordinary/nested logical
lifetimes, cycle-retained aliases, nested COW, source array operations, game/level/
self roots, scheduling, expanded libraries, automatic loading and stock/mod runs
remain open. S5 and full GSC compilation/execution remain open.
