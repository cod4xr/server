# Draft ADR 0071: Managed array values and COW ownership

Date: 2026-11-01

Status: proposal for complete ownership/COW. ADR 0072 implements only managed
host-owned empty data, snapshots and empty detachment. Populated/nested storage,
frame/scheduler ownership, alias-marker publication and source/VM activation remain open.

## Required behavior

Preserve native local/field value copies, unique writes, COW on excess owners,
recursive copies of nested arrays, shared ordinary struct identity and copy-induced
key-order reversal. Root-only reachability does not determine COW ownership:
ADR 0060's cycle-alias traces and ADR 0070's graph analysis establish a distinct
cycle-retained alias boundary. No native packed count/name/pool width becomes an
authoritative allocation rule. Source activation waits for working ownership/COW.

## Candidate: data identities and owned value locations

Keep immutable array-data identity separate from a mutable owned value location.
Each local, operand, stored field or retained host value owns one location; copying
a script value creates another location referencing the same data. Moving a value
transfers a location. Resolving a writable location detaches its data when managed
owner count exceeds one or a retained-cycle alias marker requires native COW.
An owned location is a checked arena resource with explicit lifecycle, not a raw
Copy snapshot or an implicit Rust Clone/refcount operation.

Use the existing flat object/field graph for data when practical, with checked
array-versus-struct classification and typed keys. Shared raw ObjectId mutation
must never bypass array COW. Integer keys use wide authoritative storage; native
script index validation belongs to a named legacy profile. Key ordering remains
the explicit sibling order, including reversal when copied.

An independent value-location arena can use its own full HeapIdentity namespace.
This may distinguish data snapshots from managed locations without packing a tag
into generation/slot bits or growing every KernelValue. This representation is a
candidate requiring layout, ownership and workload measurements, not an accepted
public API. Separate parallel array metadata is another candidate for avoiding
overhead on struct-only heaps; compare it with tagged parent records before choice.

Caller-visible snapshots must stay nonowning and normalized. Reading/removing a
field cannot return a handle into a value location that has just been released.
Explicit host-owned locations and nonowning data snapshots need different contracts;
copying a handle cannot accidentally create a second native value owner.

## Frame, scheduler and failure lifecycle

Value-location creation must charge allocation, managed-reference changes and
bounded unwind cleanup before publication. Successful frame exit releases its
remaining locations; moving a return value must preserve it for the caller.
Failure cleanup cannot depend on remaining instruction/heap fuel, allocate an
unbounded queue, invoke user callbacks or leak owners into later invocations.
Ownership transfer into a persistent field must survive invocation cleanup.

Owner registries should track live locations, not append allocation history: a
100,000-copy loop must reuse bounded metadata. A lazy frame-owner record and
intrusive location lists are candidates for bounded cleanup. Suspended scheduler
frames/events must retain owners across ticks; an invocation-epoch shortcut must
not silently discard their ownership. The offline executor is an adapter, not the
world runtime's lifetime policy. Reentrant calls and scoped host roots need checked
owner boundaries before any host callback authority is enabled.

## Collection and copy publication

Before reclaiming garbage that native cycles retain, mark live array data reached
by that closure with a logical retained-alias marker. Keep the marker on old data;
fresh COW copies do not inherit it. Reclaim payload and release ordinary owned
locations without suppressing native-observable future COW/key reversal. Validate
this against native traces and independent models rather than assuming a tracing
collector or reference counter alone establishes compatibility.

Preflight nested-copy nodes, fields, keys, owner locations, index growth, work and
simultaneous staging scratch before target publication. Traverse iteratively and
copy each nested-array occurrence according to native behavior; do not preserve
sharing with a memo table without evidence. Host-created cyclic array paths need
explicit checked handling. A failed staging allocation must not publish a partially
rewritten lvalue. Completed prior COW/path steps versus later invalid-key errors
need their own native-profile tests.

## Acceptance before activation

Replay existing 306 array helper traces and 20 cycle-alias traces with clear
helper/model/native VM distinctions. Implement simple/nested local and field
lvalues, RHS-before-target and index-before-parent order, literal/computed Undefined
delete differences, copied key order and strict legacy key errors. Retain temporary
RHS/receiver/key/result roots through every allocation/collection retry.

Measure at least 70,000 live arrays/owners/keys, bounded 100,000-cycle churn,
12,000-deep nested copies, repeated unique writes, duplicate nested-array values,
struct/array cycles and live alias boundaries. Check all resource failure prefixes,
source maps/privacy, source-drop ownership, frame/caller/host/unwind lifetimes and
exact metadata/work/scratch. Preserve full workspace/fixture gates and file limits.

Avoid a speculative crate/framework, unmetered hash-map probes, infallible allocation
escape, unconditional whole-heap scans on every write or a new target directory.
Implementation evidence may revise this proposal. S5 and complete GSC remain open.
