# ADR 0054: Owned runtime string storage and execution

Date: 2026-10-31

Status: accepted for the string-store component; automatic VM integration remains proposed.

## Context and semantic assessment

Owned literal/string helpers preserve decoded bytes and bounded plain concatenation
but establish no VM heap accounting. Runtime values currently copy primitive data
and opaque function references. Large immutable strings must not copy payload bytes
on every local load, argument or return, nor depend on native packed IDs/fixed pools.
Native string reference counts/interning are implementation mechanisms; valid
plain-string language equality concerns byte contents, not exposed native IDs.
Strings have no user finalizers or child references. Future arrays/objects can have
identity/cycles and require additional root/edge/lifecycle evidence; string support
must not assert that a complete object collector has been selected or implemented.

Per-invocation append-only storage would retain every overwritten temporary until
return and create avoidable exhaustion in loops. Owned Vec copies would repeatedly
copy large immutable values. Per-value Rc/Arc allocation complicates fallible
admission and couples immutable compiled code to runtime ownership. Assess an owned
growable string slot store with opaque wide owner/slot/generation references and
explicit complete-root reclamation instead. It requires no unsafe code, reference-
counting framework or third-party allocator. Measure churn/wide/large workloads and
complete runtime rooting/error tests before accepting the strategy.

## Proposed storage boundary

Own exact bytes without UTF-8 repair, native terminators or ID interning. References
copy cheaply, expose no addresses/raw constructors, and do not keep a store alive.
Checked nonreused store identity and wide nonwrapping generations reject stale/
foreign/recreated owners. Reuse vacant slots; retire exhausted generations. Admit
live strings/payload, retained slot metadata/count, single-string size, work and
reclamation scratch independently through configurable budgets, never native pool
widths. Preflight before fallible allocation; no partial value escapes failure.

Complete-root reclamation validates every reference and preflights all fallible
work/scratch/count changes before deleting data. Mark immutable leaf strings from
the supplied roots, then sweep without fallible allocation. Work failures preserve
the store and retain charged work. Duplicate roots are valid. Collection is an
explicit ownership operation, not automatic proof that arbitrary caller handles
remain rooted. Returning borrowed bytes prevents mutation through that same Rust
borrow; copied IDs remain checked data, not ownership leases. Logical metadata/
scratch excludes allocator rounding/spare capacity/RSS, stated explicitly.

## Proposed execution integration

Keep immutable compiled literal tables separate from mutable stores. Verify every
literal operand, including unreachable code, and include table/payload copies in
compiler/kernel admission. Runtime literal caches avoid copying the same source
literal on each evaluation. Stack/local/argument/return copies use opaque handles.
Plain string reads/comparisons/concatenation resolve checked bytes; preserve actual
legacy-helper cases and explicitly distinguish former capacity failures. Localized
and mixed coercions need separate evidence, never guessed formatting rules.

Reclaim only between instructions, before allocating/popping string operands.
Roots must include all live frames/locals/stacks, cached constants, supplied
external roots and any pending result. Share one work/memory context across the
entire call tree; no per-function budget resets. An explicit heap-backed invocation
keeps caller ownership/root requirements visible. Pure existing entry points cannot
fabricate a dangling string result from a silently dropped temporary store.
Source/CLI string support needs separate integration/rooting acceptance evidence
before being claimed. This implemented storage component adds no heap/value crate
or host callback framework; it is exercised through real literal/store operations
and the measured executable workload.

Operational diagnostics expose only fixed categories/work/counts. Explicit CLI
string output must escape arbitrary bytes and never put result contents into logs.
Forward host/scheduler/game roots and cycles remain unimplemented requirements,
not inferred from immutable leaf collection. No serialized/native/client ABI.

## Acceptance evidence

Require namespace/generation exhaustion, copy/foreign/drop/stale/reuse/retirement,
exact/one-less live/count/byte/metadata/work/scratch admission, atomic failed root
validation/collection, empty/duplicate roots, malformed/hostile sequences, 70,000
live strings and strings beyond native scratch bounds. Measure bounded churn with
complete roots, retained payload/slots and work before accepting reclamation.
Replay existing actual string-helper fixtures through integrated operations with
explicit parity/expansion counts. Test source literals/copies/calls/returns/loops,
GC pressure/error/cancellation rooting, maps/privacy, CLI result handling and all
existing workspace/native-profile gates. Full arrays/objects/closures/scheduling,
general builtins/coercion/localization, mods and Linux stay open.

## Initial measured component evidence

The standalone storage component preserves 54 actual native plain-string helper
successes and separately admits 54 old capacity failures, following literal decode,
owned insertion, concatenation and complete-root reclamation. It does not execute
the native VM/interner/ref cleanup. Tests cover 70,000 distinct live strings,
100,000 temporary values under 64 live/slot budgets, 4,096 hostile operation
sequences, exact/one-less resource admission, atomic invalid/failed collection,
identity/generation exhaustion, retirement and privacy.

On Windows x86-64/Rust 1.95 the deterministic 100,000-operation churn example used
1,587 pressure collections and 2,008,131 logical work units including final
reclamation. Peak payload was 1,012 bytes; final payload four bytes, 64 retained
slots and 2,048 logical metadata bytes. One observed debug run took 130,453 us;
one release run took 9,311 us. These are local component observations, not
allocator/RSS/full-VM/native-server performance evidence. Automatic frame/cache/
external-root discovery, collection timing and source/CLI strings remain proposed
integration work; no object-cycle/scheduler/native cleanup acceptance is implied.
