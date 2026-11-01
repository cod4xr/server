# Draft ADR 0088: Local nonnested array assignment

Date: 2026-11-01

Status: accepted for the typed-cell component; source indexed writes remain a
proposal and are still rejected. This review does not
authorize stored/nested arrays, compound/postfix updates, game replacement/writes,
uninitialized local targets, scheduling or complete GSC.

The typed-cell component passes complete Windows acceptance.
It adds StoreArrayLocal/ClearArrayLocal, checked receiver ownership, staged COW,
an internal nonreplacing child selector and metered pressure retries. Fifteen
focused tests currently pass, including all 32 native primitive D outcomes,
Undefined B/N normalization, hostile targets/keys, every shorter work/instruction/
setup/publication prefix, sixteen one-less resources, real transient child quotas,
stack-only RHS graphs through COW/insertion pressure, result graphs after last-array
release, 70k entries and 100k updates beside 70k host aliases under 4 KiB scratch.
KernelResource moves to its own leaf with its existing public re-export preserved.
Source lowering stays closed pending native block-local cleanup work: the pinned
compiler emits OP_RemoveLocalVariables (0x17) in if/if-else, break/continue, switch
and developer-block paths. Current source locals do not establish that scope parity.

Full Windows formatting, all-target denied-warning Clippy, debug/release workspace
tests, denied-warning rustdoc, policy and diff checks pass. Both profiles preserve
all prior names and execute 345 frontend/257 legacy tests, fifteen private script
tests and two doctests. The selected write/lifetime guards reject 141/61 mutations;
306 actual selected native array traces match O0/O2, 99 structural replays and
twelve suite-policy regressions pass. All existing outcome rows remain unchanged.
Policy checks 981 sources/eight crates/largest 287 lines. Linux/hosted CI is unrun.
No new crate/dependency/target directory or source assignment activation follows.

## Problem

Independent empty construction and host reads cannot implement ordinary local
array-building scripts. General ownership remains unfinished under 0071/0075.
Consider a bounded extension for plain assignment/clear through already initialized
local value cells, while preserving the prohibition on every stored Array value.

## Candidate lifetime argument

With Array values forbidden in both struct fields and array entries, ordinary
object graphs cannot hold an Array owner or form a path back to Array data. Array
owners therefore still consist only of explicit host locations and VM cells,
including copies/moves through calls. Their current metered registry determines
which locations survive callee exit and error cleanup. Ordinary cycles cannot
retain hidden Array aliases within this closed graph. This argument must be
revisited when persistent/nested array edges or scheduler/host callbacks activate;
it does not substitute counters for the combined policy required in 0071/0075.

Before source activation, also verify native block/loop-local lifetime lowering.
A lexical alias retained longer than the native local cell could force a later
COW/key reversal even without stored arrays. The existing function-frame registry
alone does not establish those source scope boundaries. Typed-cell tests can
proceed independently, but source lowering must preserve or explicitly reject
unsupported scope behavior rather than silently extending alias lifetimes.

Populated independent data changes the read proof. A callee-returned array can be
the last owned receiver; reading a struct/string result then consuming that array
can leave its data unowned. Core releases defer payload collection and cannot
allocate, collect, yield or invoke callbacks. The result can therefore be pushed
into the current frame before any collection is possible. Subsequent collection
must root that frame/caller result even while reclaiming the old array data.
Tests must demonstrate this with runtime-created strings/objects that have no
remaining host or literal-cache backing. No stored Array may escape a getter.

## Candidate writable local instructions

Use distinct typed local-cell store/clear operations with complete local operand
validation, initialized-local checks and documented stack effects. A writable
receiver must borrow the location already owned by that frame. LoadLocal creates
another owner and would incorrectly force COW on an otherwise unique target.
Core make_array_unique changes data inside the selected location, preserving its
identity and registry membership. Other local/host aliases retain old data; copies
retain the existing native sibling reversal and shared ordinary struct identity.

Evaluate normal assignment RHS, then key, then the writable local target. Keep
RHS/key operands rooted until successful completion; pressure retries must include
all current/caller/cache/world/managed roots. A target explicitly initialized to
Undefined may autovivify through the same admitted constructor/publication path,
storing that new owner into its local before key validation. Existing array targets
select COW before key validation. Do not reuse read-path preflight ordering.

Reject unsupported stored Array RHS values without publishing an array field.
This out-of-profile rejection need not invent native nested-write behavior. Current
primitive/shared-struct/function/string payloads retain checked references and
opaque image identities. Source targets outside an initialized plain local name,
including nested paths, fields, game roots and compound/postfix updates, stay
rejected at compilation. Restricted standalone source validation profiles retain
their boundaries; typed standalone execution still needs explicit object context.

## Undefined and publication order

The existing native write guard and actual removal corpus distinguish direct
literal Undefined from grouped/computed Undefined. Literal dispatch uses ClearArray;
an Undefined target errors rather than autovivifying. Computed Undefined uses the
ordinary lvalue path: create/select the array, create/select a child, then remove
that child. Missing computed keys transiently consume a field/key slot and peak
size, while missing literal clears do not create a child. Raw host Undefined field
storage is separate from this source assignment behavior.

Preserve existing payloads while selecting an existing child. Add a focused internal
ensure-child operation if needed; blindly replacing an existing value with Undefined
before removing it adds an unnecessary state change and loses the old payload early.
Each core phase remains atomic. Completed autovivification/COW/child creation can
precede later invalid-key/work/resource failures; an invocation is not a transaction.
Registry cleanup must release every VM owner with no remaining fuel. Unowned data
may remain deferred; surviving host aliases and their key order must stay correct.

Convert keys through the existing named strict integer/plain-string profile after
target selection. Preserve complete byte keys, checked wide size results and
authoritative u64 typed storage. Do not introduce native packed capacity limits.
Only eligible storage pressure gets one paired retry per allocation phase; work,
scratch and unsupported types fail directly. Unique updates cannot scan unrelated
heap rows or recreate aliases merely to obtain a receiver.

## Evidence and acceptance required

Use the existing 306 native array traces and selected write-role guard as evidence,
with explicit supported/helper-only classifications. In particular, replay the
primitive D clear/computed-Undefined outcomes and A/B autovivification behavior
through real source/typed execution where representable. Do not claim native VM
execution, allocator reuse, nested COW, cycle markers or error unwinding from helper
replays. Keep all old outcome rows unchanged.

Check unique versus aliased writes, host/caller retention, shared structs, complete
key/order/type results, RHS/key/target evaluation order, literal/grouped/computed
Undefined, missing keys, Undefined target behavior and COW-before-bad-key state.
Bind transient field/key quotas so missing computed assignment cannot bypass them.
Verify result graphs after their last array owner is consumed and old data reclaimed,
plus temporary RHS/key graphs during COW/field-pressure retries. Cover exact and
every shorter instruction/setup/heap/publication prefix, resource/scratch limits,
same-context recovery, wide entries/locals and bounded churn beside unrelated data.
Retain constructor/transport/read coverage and explicit nested/root/update gates.

Run runnable source/CLI examples, private located diagnostics, numeric array
accounting, native write/lifetime/structural/suite guards and complete Windows gates
before accepting activation. No new crate, dependency or generic mutable-reference
framework is needed. Full GSC/S5 and join milestones remain open.
