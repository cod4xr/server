# ADR 0070: Budgeted reference-cycle boundary analysis

Date: 2026-11-01

Status: implemented as an opt-in read-only graph component. Array storage,
managed-value ownership, logical alias markers and VM integration remain open.

## Problem and decision

ADR 0060's 20 native cycle-alias traces show why reachable-array alias counts are
insufficient for compatible COW/key order. Before selecting storage/collector
integration, implement and measure the graph property needed to identify native-
retained aliases without keeping unreachable payload indefinitely.

ObjectHeap::analyze_cycles accepts the complete roots and returns unreachable
object count, the unreachable closure retained by reference cycles, outgoing
reference-edge count into live objects, and unique checked live target snapshots.
The closure includes downstream garbage, not just cycle members; ordinary
acyclic upstream garbage is excluded. Root-reachable cycles are excluded.
Results do not root their targets or their store and redact identities/payloads
in Debug output. No automatic collection, COW decision or alias pinning changes.

Trace root reachability, count incoming edges within the garbage graph, then peel
zero-indegree garbage iteratively. Remaining garbage is the cycle-retained
closure. Inspect only its outgoing edges for live boundary targets. Multiple
edges count separately, while returned targets are unique in arena-slot order.
All graph passes validate object identities, owner links and visited field counts;
keys, strings and opaque function bodies are not inspected.

Keep the implementation in a cohesive objects/cycles folder with separate report,
edge walk, workspace and algorithm leaves. Reuse explicit object work/error/root
admission. No crate, dependency, native linkage, root lease or target directory is
introduced. The array strategy remains under assessment; this component is not
Rust array execution or native refcount/collector conformance.

## Resources and failures

Prepay each root/node/field/reference visit and output scan/copy. Admit initialized
reachability/target flags and degree rows, the requested maximum queue reservation,
and simultaneous output ObjectId rows before fallible allocation. Counts/byte
formulas use checked arithmetic and actual target layout. Scratch excludes
allocator rounding/spare capacity/RSS, not the deliberately reserved queue slots.
Analysis is read-only; invalid roots/invariants, exhausted work/scratch or failed
allocation return charged usage without a partial report or heap mutation.
Ordinary collection retains its previous work/storage profile.

## Evidence and measured workload

Six tests cover the native corpus's 20 graph topologies and old/fresh target
distinction, 4,096 deterministic graphs compared with an independent creation-owner/
reference-release model, cycle/downstream/upstream topology, duplicate roots,
missing/vacant/reused/stale/foreign identities, every insufficient work prefix,
exact scratch/output overlap, nonrooting reports and a 70,000-node cycle plus
downstream node. The native topology replay uses ordinary graph nodes as array
stand-ins; Rust array allocation, copying and mutation are not implemented by it.

The reproducible object_cycle_workload example reports 840,016 logical analysis
steps and 1,260,042 peak scratch bytes for a 70,000-node cycle with one live boundary
target. The wide heap uses 13,714,440 logical metadata bytes on this Windows layout.
Its separate 100,000-cycle churn phase takes 1,563 rounds and 1,619,012 analysis
steps, with 1,194 peak analysis scratch bytes and 65 retained object/128 field
slots. Construction and ordinary collection work are excluded from those analysis
step totals. Timings are local observations, not portable benchmarks, native speed
comparisons, process memory or a server-tick latency guarantee.

```powershell
cargo run -p cod4x-script --example object_cycle_workload --locked
```

Native fixtures/source guards remain separate evidence. Managed array copies,
COW staging/publication, field/value-slot ownership, frame/unwind/scheduler roots,
legacy key rules, scoped alias-marker publication and source activation still need
implementation and their own acceptance checks. The S5 gate remains open.

Windows workspace debug/release tests, warnings-denied Clippy/rustdoc, formatting,
diff checks and repository policy pass (773 sources/eight crates/largest 287 lines).
Script suites contain 267 frontend/189 legacy tests, six private units and two
doctests. The debug workload and six new graph-analysis tests pass. Linux/hosted
CI, Rust arrays and native game interoperability remain open. Draft ADR 0071
records the next managed-value ownership assessment, not a completed runtime.
