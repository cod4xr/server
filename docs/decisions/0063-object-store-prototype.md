# ADR 0063: Checked shared-object storage prototype

Date: 2026-11-01

Status: accepted for explicit-root shared struct storage; VM/array activation remains open.

## Scope and assessment

The array assessment established value-copy semantics that ordinary mutable shared
handles cannot provide. Structs instead retain reference identity and can form
cycles. Prototype that concrete shared-object subset separately before activating
objects or choosing array ownership in the VM. This does not accept a complete
object/array collector, native refcount policy or source execution profile.

Use opaque wide owner/slot/generation object identities, owned exact field-key bytes
and copied primitive/object values. Bind each object store to one existing string
store identity; neither copied ID owns its store. Validate every supplied object
and string reference before storing it. Opaque function snapshots grant no module
ownership; their module must be checked separately before any later invocation.
Preserve raw stored Undefined separately from absent fields;
script array deletion is a later lowering/runtime responsibility, not a generic
field-table rule. Object references share identity, without copying fields.

Use flat object/field arenas and a purpose-specific open-addressed field index.
Charge every probe and content comparison, rather than depending on unobservable
hash-table collision work. Admit retained bucket lengths explicitly and rebuild
tombstones with separately budgeted staging. Hashes are internal, not an ABI;
the pinned toolchain and workload define reproducible logical visit counts.

Prototype explicit complete-root iterative mark/sweep for this struct graph only.
Validate/preflight all references, work, scratch and usage changes before deletion.
Trace cycles without native recursion or recursive Rust destruction; retain and
reuse vacant arena slots and retire exhausted generations. String payloads remain
owned by StringHeap. A composite string-reclamation operation must include all
stored string fields plus supplied frame/cache/host string roots, so callers can
preserve nested strings while the object graph is borrowed immutably. Collect
objects first when unreachable cycles should stop retaining their string fields.

Admit live objects/fields, key bytes, retained object/field/index metadata, individual
keys, work and scratch independently. Logical counters exclude allocator rounding,
spare capacity, borrowed input, fixed owner fields and RSS; staged index/mark/root
storage has explicit scratch accounting. Failed operations retain charged work
without publishing partial mutations. No new crate/dependency or unsafe code.

## Activation boundary

Object values may be stored by this component, but immutable executable constants
and every current primitive invocation path must reject them. String-only VM
collection cannot silently ignore an object graph. No object source constructor,
field lowering, game/entity service, pointer subtype, scheduling or native plugin
authority is added by the storage prototype. Later VM integration must assess
complete frame/temporary/result roots and failure/cancellation cleanup.

Array ownership remains open under ADR 0060. In particular, shared-array writes
need managed alias counts, recursive nested-array copies and copy-order evidence;
collecting otherwise unreachable references must not silently change observable
legacy copy behavior. Do not infer array support from this shared struct graph.

## Validation and consequences

Fourteen integration tests cover 70,000 simultaneous objects and fields,
content-equal/non-UTF-8/1 MiB keys, 100,000 churn cycles, a 12,000-object cyclic
graph, shared identity, exact/one-less retained/work/scratch admission, every
insufficient work prefix during index growth and collection, atomic late invalid
roots, foreign/stale/recreated stores and nested plain/localized string roots.
A separate HashMap/BTreeMap graph model checks every live field through 4,096
mixed operations with independent budget/operation/value selections. Coverage
assertions require successful object/string edges, wide live sets and reclamation.
Two private tests inject exhausted generations and a 128-entry equal-hash chain;
the common identity allocator separately checks the full nonzero namespace domain.
Current primitive entry points and immutable constants reject object snapshots.

Reproduce the component workload using the normal Cargo target directory:

```powershell
cargo run -p cod4x-script --example object_heap_workload --locked
cargo run -p cod4x-script --release --example object_heap_workload --locked
```

The 2026-11-01 Windows x64 local runs perform 100,000 shared graph updates under
64-object/64-string budgets. Both report 1,613 paired object/string collection rounds,
8,825,031 logical work
steps, 4,096 peak scratch bytes, 576 peak key bytes, 1,012 peak string bytes and
21,504 peak retained logical metadata bytes. They finish with one object/field,
five key bytes, four string bytes, 64 object slots, 128 field slots, 256 index
buckets and 64 string slots. Observed elapsed times were 683,795 us debug and
48,573 us release; these are local example observations, not stable benchmarks,
native-server comparisons or RSS measurements. Tombstones retain the bounded
index until later rebuild or complete field removal; no automatic shrink is promised.

Flat records avoid recursive owned destruction and per-object hash allocations.
The dedicated index makes collision work and retained buckets observable, at the
cost of maintaining checked links, tombstones and staged rebuilds. An ordinary
HashMap would conceal probe work; tracing arrays with copied handles would violate
legacy value-copy behavior. A native refcount port would inherit narrow fields
and require a separate cycle/lifetime policy. These alternatives do not justify
activating this component as a general object VM or array strategy.

Full workspace gates and current primitive/CLI regressions remain mandatory;
native GC/VM, full mods, Linux/hosted CI and platform acceptance stay open.
