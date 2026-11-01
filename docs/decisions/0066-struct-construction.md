# ADR 0066: Explicit-context struct construction

Date: 2026-11-01

Status: accepted and implemented for empty ordinary structs.

## Decision

Expose sealed SpawnStruct through bound-source spawnstruct after active local
script/developer lookup. The native constructor body does not inspect parameters;
evaluate all supplied arguments, allocate the empty struct while those operands
remain rooted, then discard them. Support wide counts under explicit code/stack/
fuel budgets rather than native byte counts. Pure/string-only execution still
requires an explicit object context to execute this allocation; skipped calls do
not gain ambient authority or fail merely because their branch is unselected.

Allocate through the existing checked ObjectHeap. On live-object, retained-slot or
metadata pressure, gather the complete current/caller/cache/external roots,
perform object then composite string collection under cumulative work/scratch,
and retry once. Other failures do not retry. Individual store operations are atomic;
completed allocations/collections can remain after invocation failure. Publish the
new object before collapsing operands, with no intervening collection. Current
frames and returned/host roots retain ADR 0064's nonowning snapshot contract.

Separate common heap dispatch from string operations and object allocation leaves.
Add no new crate, dependency, native linkage, allocator escape or target directory.
Array ownership/COW, struct fields, entity lifetimes, methods and scheduling remain
separate implementation work. Immutable object constants remain rejected.

## Evidence and native boundary

The new source guard protects nondeveloper registration, selected allocator/type/
output/ref-transfer roles and absence of argument inspection, rejecting 22 scoped
mutations. The constructor oracle executes actual AllocObject, AddRefToObject,
RemoveRefToObject, Scr_GetObjectType, Scr_AddObject and Scr_AddStruct at O0/O2.
Its allocator and output-slot services are bounded models; free/clear/entity
branches abort if reached. All 32 traces agree, including enabled/disabled native
debug leak counters and the constructor's decrement from one excess reference to
zero. Rust replays fresh empty object results and distinct identities, not native
refcount ownership, allocator/cleanup, input-stack, unwinder or pointer ABI parity.

Six typed tests cover those traces, 65,536 evaluated extras with exact/one-less
stack/fuel admission, 70,000 simultaneously live constructors in VM locals,
caller-stack-only graphs/nested strings under object pressure, independent live/
slot/metadata pressure and every insufficient global work prefix plus scratch.
Three source tests cover query/case/shadow behavior, original argument-error maps,
temporary-operand roots that cannot evade live limits and 100,000 loop iterations
under two slots (99,999 paired collections). These capacity/resource tests are not
native compiler/VM execution. Full workspace and source guards remain mandatory.

## Offline application

The offline runner owns both stores and keeps them borrowed through output/logging.
Object results print only the opaque `object` kind, like function results; no ID,
address or fields are exposed. Logs contain numeric object/field/slot/index/metadata
usage and separate completed object/string collection counts. Two process tests
check constructors, nested/evaluated extras, script shadowing, opaque output,
structured metrics, private failures and readable no-color logs. The normal target
struct_demo.gsc command returns integer two from definedness plus Size. This is
still an offline subset, not a joinable server or full GSC/mod acceptance.

Windows verification passed full workspace debug/release, warnings-denied Clippy/
rustdoc, formatting, repository policy (727 sources/eight crates/largest 287 lines)
and diff checks. All 32 helper traces, 22 constructor-role mutations and 99 existing
structural replays regenerate. Current script suites contain 248 frontend/171 legacy
tests, six private units and two doctests; both new process tests pass. Linux/hosted
CI, native game execution, fields/arrays and full GSC remain open.

```powershell
cargo run -p cod4x-server --locked -- --execute-gsc examples/gsc/struct_demo.gsc --color never
```

The command prints `integer 2`; human/JSON events expose counts only. Returning
spawnstruct directly prints `object`, without an identity, address or field dump.
