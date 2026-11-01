# ADR 0075: Native holder release and array copy decisions

Date: 2026-11-01

Status: accepted compatibility evidence; nested-value/source ownership implementation open.

## Finding

Dropping an ordinary holder's last external reference can immediately clear its
fields and release outgoing array aliases. A self-cycle and its downstream chain
retain those aliases instead. This difference changes later array COW and key order:
an array whose acyclic aliases have disappeared stays unique, while cycle-retained
aliases can still trigger detachment/reversal. Merely retaining every field-owned
location until a later tracing collection would therefore produce wrong decisions.

## Evidence

generate_array_lifetime_vectors.py executes actual selected AddRefToObject,
RemoveRefToValue/Object, MakeVariableExternal, FreeChildValue, ClearObjectInternal,
ClearObject, RemoveRefToEmptyObject, FreeVariable, Scr_EvalArrayRef, CopyArray and key
helpers from the pinned source. Both O0/O2 yield the same 210 traces. Cases cover
one-to-four-node chains, no cycle or a self-cycle at each position, one-to-three
array aliases per holder and zero-to-four integer keys. Parent and child free-list
publication, statuses/backlinks, live-parent and total-reference counters are checked.

An independent Python model builds node edges/owner counts, drops the external root
and iteratively releases zero-owner nodes. It checks freed parents/fields, retained
array aliases, copy/no-copy, excess counts, original/selected key order and exact
210-case coverage. script_array_lifetime_reference.py rejects 61 deletion/reordering
mutations of selected native roles. CI and generate_syntax_tables.py include it.

Allocation, collision-free pool lookup, graph/ownership setup, output, and parent
allocation records/counters remain explicitly modeled. The new parent-accounting
oracle profile initializes records/counters needed by native FreeVariable; its
allocator remains monotonic and never consumes native free lists. Existing 306
array and 20 nonfreeing cycle-alias fixtures remain byte-identical. Entities and
string/vector cleanup abort as unavailable. Debug leak services are absent.
This does not execute the native compiler/VM, allocator reuse, unwinding or GC.

## Consequences for nested/source ownership

Distinguish semantic ownership events from physical payload reclamation. Native
ordinary-object lifetimes determine when their array fields lose owners, so array
counts alone are insufficient if ordinary frame/field owners remain untracked.
Copies, moves, replacement, frame exit, return, cancellation and error cleanup need
an explicit budgeted ownership contract before nested values/source arrays activate.

The implementation may retain payload storage for safe host snapshots, but doing
so must not retain logically dead acyclic aliases and change later COW/key order.
Existing host snapshots are nonowning and can be retained before collection; that
host extension is not native resurrection of freed GSC data. Any lifetime change
needs explicit snapshot/adoption/release behavior and migration tests.

Cycle-retained aliases remain a separate issue from acyclic logical release.
If physical collection removes native-retained cyclic payload, preserve the
observable live-array copy boundary, without marking acyclic aliases or inheriting
markers onto fresh copies. ADR 0070's graph probe and proposal 0071's logical-marker
candidate support assessment; neither implements full ownership or this requirement.

No particular refcount/collector design is selected here. Candidate designs must
demonstrate these native states, ordinary/nested/array cycles, host/frame/scheduler/
unwind lifetimes, bounded wide/churn/deep work and exact failure accounting. Avoid
whole-heap scans on every write or silent native-order changes. Current ADR 0074
copies only primitives/shared structs; it does not yet expose nested array fields,
so this new evidence does not activate or claim those semantics.

## Validation

Windows GCC/Python fixture regeneration/checks passed: 210 lifetime, original 306
array and 20 alias traces, 61 lifetime-role mutations, 99 syntax replays and repository
policy (847 source files/eight crates; largest 287 lines). No Rust runtime/source
behavior changed; its preceding full Windows gates passed at commit 14ff56c.
Linux/hosted CI and actual stock/mod execution remain unrun. Full GSC/S5 stays open.
