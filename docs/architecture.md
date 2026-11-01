# Architecture and expansion

ADR 0066 connects bound-source spawnstruct to explicit object-context allocation.
Constructor operands remain rooted until the empty object is published; admitted
memory pressure runs one complete collection pair before retry. Common heap
dispatch, string operations and object allocation remain separate leaf modules.
The offline application owns both stores, prints opaque object kinds and records
numeric storage/collection metrics. Struct fields and array ownership remain open.

ADR 0063 adds a separately assessed shared struct graph with flat object/field
arenas, opaque checked identities, owned exact keys and a purpose-specific metered
field index. Explicit-root iterative collection reclaims unreachable cycles only
after complete admission/preflight; composite string collection retains every
stored plain/localized field plus supplied external roots. Retained metadata,
key bytes, work and staging scratch have separate budgets. Wide/churn/deep/cycle/
hostile workloads assess this storage component. The current VM rejects object
snapshots in pure/string-only invocations. ADR 0064 adds optional object-aware
reference transport and roots every frame/caller/external object graph plus direct
and cached strings during pressure. Object then string collection share cumulative
work and simultaneous cache/root/mark/queue scratch, with separate success counts.
Returned snapshots need explicit host roots before later invocations. Struct field
operations, general object operators and legacy array COW remain open; empty struct
construction now follows ADR 0066.

The script heap now owns immutable byte strings in growable slots with opaque
store/slot/generation references. Copies do not copy payload or grant ownership.
Explicit complete-root reclamation validates references and preflights resources
before deleting anything; exhausted generations retire without reuse. Live bytes,
retained slots/metadata, single values, logical work and scratch have separate
budgets. Counters exclude allocator rounding/spare capacity/RSS. This is a tested
storage component, not automatic VM frame/cache rooting or an object-cycle
collector. Immutable source literal tables and runtime execution integration stay
separate ownership boundaries; see ADR 0054.

ADR 0055 now connects typed execution and supplied source arguments to an explicit
borrowed KernelHeapContext. Immutable code rejects runtime string constants. One
context shares work across the entire iterative call tree; pressure collection
scans current/caller locals/stacks and supplied external handles before popping
operands. Root IDs and mark flags share scratch admission. Failed work remains
charged and failed invocations can retain completed collection/allocations. Plain
addition/equality use checked bytes; source literals/cache/CLI output and general
object/scheduler roots remain open. Results stay in caller-owned storage.

ADR 0056 adds independently owned immutable typed literal tables and checked
operands. Complete module proof assigns flattened cache offsets without borrowed
descriptor arrays; payload/records and proof work are admitted globally. Context
cache initialization and active-index capacity share scratch with root IDs/flags.
Lazy loads publish one checked heap ID per used row and retain cached-only roots.
Reusing a context discards its previous invocation cache without resetting work.
ADR 0057 now lets bound source adopt exact decoded plain-string bytes after work/
metadata admission and carry tables through staged code into proof. Compiler scopes
include prior staged tables while lowering later bodies; kernel proof copies have
separate explicit admission. Isolated scalar profiles remain unchanged. The offline
application owns its heap through result resolution, borrowing escaped result bytes
for stdout and logging only categories/counts. ADR 0059 adds a distinct localized
value/literal tag with the same checked storage/cache/rooting. Only same-tag string
equality and plain concatenation are supported; mixed/localized arithmetic rejects
explicitly. Raw localized output performs no lookup or translation. The C oracle
checks 3,024 synthetic-identity outcomes without relaxing default string services.
String switches, translation/
mixed coercion, general objects and host/scheduling integration remain open.

ADR 0061 adds plain-string byte indexing and unary Size through the explicit heap
context. Both use the legacy first-NUL prefix; result allocation/collection retains
all operand roots until success, and global work admits scans. Bound source keeps
index-first evaluation and operator maps. Arrays and object execution remain unimplemented.

ADR 0062 adds primitive definedness with a required first argument and evaluated,
discarded extras. Exhaustive value/arity policies require explicit decisions for
future object/entity values; no native lifetime checks are fabricated. Local and
developer resolution still precede fallback. Global stack/fuel bounds include all
operand pushes; resumable preloaded stacks need their own future work contract.

ADR 0065 adds checked ordinary-struct definedness and Size=1 under that context.
Size validates the reference with constant global work instead of scanning/counting
fields. Native pointer-size rows distinguish supported ordinary structs from
helper-only subtypes; entity/thread lifetimes and other object operations stay open.

## Dependency direction

The server and future client are peers above shared engine libraries. Shared
code must not depend on either executable, on ApX, or on rendering/windowing.
The Rust core must eventually own runtime behavior; wrapping the original binary
or linking its assembly is not completion of the rewrite.

```text
server application             future client application
        |                              |
server runtime                  client runtime / prediction
        |                              |
        +-------- shared simulation ---+
        |                              |
        +-------- shared protocol -----+
        |                              |
        +-------- transport -----------+

asset readers ---> validated asset data ---> shared simulation
script runtime / host bindings -----------> server game services
ApX modules -----------------------------> engine extension interfaces
client presentation ---------------------> read-only simulation output
```

Arrows mean allowed conceptual dependency, not a list of crates already built.
Current implemented boundaries are `cod4x-protocol`, `cod4x-transport`,
`cod4x-server-core`, `cod4x-assets`, `cod4x-content`, `cod4x-observability`,
`cod4x-script`, and the
`apps/server` process. Protocol and deterministic
server services remain dependency-light; process signals and logging sinks live
above them. No client or simulation crate exists yet. Do not manufacture empty
crates to make this diagram look implemented.

## Shared libraries

The protocol crate owns byte representations, bounded parsing/serialization,
and explicit protocol profiles. It handles both communication directions and
does not bind sockets or decide gameplay policy. Current reader and writer have
no platform, rendering, runtime, asset, or third-party dependencies.

The transport crate implements ordinary sequence/fragmentation state without
sockets, logging, or a clock. Applications own scheduling, routing, authentication,
and partial-message expiry. Datagram I/O stays separate from deterministic state
so both server and client can test loss, duplication, reordering, and timeout
behavior without live networks. Reliable commands, the separate reliable packet
transport, and pacing remain implementation work.

Shared simulation should own movement, collision-facing contracts, weapon rules,
and serializable state needed by authoritative simulation and client prediction.
Use explicit time/tick inputs and deliberate floating-point behavior. A Rust port
does not automatically preserve the original x86/SSE rounding, evaluation order,
or prediction results. Recorded input/state comparisons decide compatibility.

Asset readers produce bounded, validated owned data or borrowed views. Filesystem
access, content acquisition, decompression, archive paths, and validation are
separate concerns. Shared parsers must not embed server-specific filesystem roots
or depend on graphics APIs. Rendering, audio playback, input, and windowing remain
client-specific. Proprietary content is supplied separately by the operator/user.

The asset catalog owns encoded bytes with exact kind/name lookup, configurable
logical byte/count budgets, and opaque catalog/slot/generation handles. Removal
invalidates handles and reuses slots. It has no codec, filesystem, or rendering
dependencies. Counts and names are independent of legacy wire namespaces; memory
statistics describe owned bytes, not allocator overhead or process RSS.

The content boundary borrows immutable IWD bytes, admits metadata and compressed/
decompressed work under explicit budgets, and returns CRC-validated encoded data.
It implements legacy virtual-name normalization and archive-local override order.
The application owns bounded file snapshots and the offline verifier; the shared
reader never accesses host paths or extracts files. IWD entry CRCs also supply
legacy seeded checksums. Mount order, pure admission, and decoded game
formats are separate implementation work. See ADR 0005 for dependency costs.

Borrowed BSP containers own only a growable metadata index, preserving raw kinds
and first-match lookup across the source's old/sequential directory layouts.
Every nonempty span is validated before publication. Multiple maps can coexist
without a global current-file pointer; payload views borrow the original source.
Decoded collision/entity/asset schemas and fastfile graphs remain separate layers.

`FastFileStream` owns the decoded unsigned IW3 v5 serialized payload. It admits
wide header accounting before fallible chunked growth, verifies exact decoded
length and a single complete zlib stream, and rejects signed files and trailers.
The nine serialized block fields do not create engine pools. The process shares
a bounded immutable file-snapshot reader between archive verification and fastfile
inspection. Neither the stream nor the inspector resolves asset graphs, allocates
declared blocks, follows native pointers, or publishes map readiness; see ADR 0009.

`ZonePrefix` borrows that stream, owns only a bounded script-slot index, and keeps
asset-directory records/remainder as borrowed views. Null, inline-empty, and encoded
string references remain distinct. Virtual allocation alignment does not skip input
bytes. Unknown kinds and unresolved headers remain explicit metadata until typed
graph readers validate them; prefix inspection never registers assets. See ADR 0010.

`RawFileCollection` provides the first typed asset subset above the prefix. It
rejects non-raw-file roots upfront, borrows exact name/buffer bytes, preserves null
slots and aliases, and returns only a complete validated supported collection.
Private staging maps logical alias slots and known inline string regions rather
than native pointers. Temporary headers reuse scoped block-zero capacity; virtual
block-four accounting includes alignment and inserted slots. No asset enters an
engine registry and no script executes; mixed stock-zone graphs remain open.
See ADR 0011. Expanded GSC requirements live in docs/gsc-runtime.md.

## Server ownership

The endpoint-cookie core owns only keyed MAC state and consumes explicit observed
socket addresses. It introduces reviewed HMAC/SHA-256 dependencies into server-core
per ADR 0006; protocol stays dependency-free. The process owns fresh OS entropy,
key lifetime, and provider replacement. Legacy cookies remain 32-bit opaque echoes,
with no implicit expiry, identity authentication, or network admission behavior.

The explicit handshake-probe runtime owns OS entropy and source-key zeroization.
Its core issues/verifies endpoint cookies under reply budgets and always refuses
gameplay admission. Runtime service selection is explicit; query-only and offline
content modes do not become implicit connection servers. See ADR 0007.
Development UDP now uses one nonblocking readiness registration in the process,
draining receives to WouldBlock with lifecycle checks between datagrams. Core
services still own no I/O. Mio is an explicitly reviewed application dependency;
the change removes repeated blocking receive timeouts, not gameplay admission or
transport reliability work. See ADR 0018.

Keep connection/authentication state, server commands, map lifecycle, snapshots,
game services, script execution, and operations in cohesive modules. Prefer one
explicit owner for mutable simulation state. Threading/async decisions require
measured need, bounded handoffs, cancellation, and documented shutdown order.
Do not spread `Arc<Mutex<...>>` throughout the engine as an ownership strategy.

GSC compatibility is an engine feature. Custom ApX game rules are consumers of
engine/game APIs; they must not change legacy codecs. Native plugin ABI support
needs a separate policy: an existing 32-bit plugin cannot simply be loaded into
a 64-bit Rust process. Assess a constrained compatibility host before promising
unchanged native-plugin support.

`cod4x-script` now implements dependency-free legacy numeric and wait primitives
without game globals, native references, a host clock, or I/O. The selected rounding
profile and defined input domains are explicit. It is not a compiler or running VM;
new value/handle storage, scheduling, host services, and the modern language remain
implementation work. See ADR 0012 and the expanded GSC contract.

Script source indexing and borrowed legacy lexemes now form concrete frontend
stages. Generated DFA/action data stays inside an explicit compatibility profile;
scan work/output limits and source ownership remain separate. Exact trivia/spans
support later tooling without normalizing source (ADRs 0013/0014). Symbol/module
registries, semantic AST lowering, compilation, and a running VM remain open.
Legacy literals now decode in a separate bounded phase with owned byte strings,
ordinary/localized distinction, checked scalar domains, and no interning/native
handles. This is not whole-source validation or module publication; see ADR 0015.
Bounded structural parsing now builds flat concrete grammar trees with scoped
borrowed node views and original terminal spans. Iterative stacks/tree destruction
support deep input under explicit budgets. No semantic actions or name binding run;
structural success does not establish compilation or GSC execution. See ADR 0016.
The application now consumes cod4x-script through an explicit offline --inspect-gsc
mode, using bounded file snapshots and location/count-only events. This approved
internal dependency adds no third-party packages, build scripts, or FFI; the shared
script library still owns no filesystem, logging, sockets, or server state.
Typed module syntax now owns bounded declaration/parameter views over a borrowed
tree. Includes and names retain raw spelling/order/duplicates. Iterative lowering
does not intern names, apply developer policy, bind scopes, or lower expressions;
see ADR 0017. The inspector reports separate declaration work/storage/counts.
Local function indexes now borrow those declarations and preflight allocation/work
budgets. Explicit developer mode controls active name lookup while all definition
metadata remains retained. Legacy ASCII identity is isolated from runtime string
storage; no engine function IDs are assigned. Import/call/parameter/local binding
and executable bodies remain open. The inspector reports name/region failures
with both original locations, never source text. See ADR 0019.
The scalar semantic kernel now provides explicit pair coercion/comparisons/division
for later frontend/VM consumers. Rust data equality is not a GSC language operator;
float profile/error scope remain explicit. No heap references, stack cleanup, or
host FP-control state is owned by these helpers; see ADR 0020.
Owned plain-string concatenation now admits checked result lengths and fallible
growth without the native scratch ceiling. It owns only the result bytes, leaving
aggregate heap/reference/GC policy to an implemented runtime boundary. Localized
and mixed-kind coercion remain separate; see ADR 0021.
Scalar arithmetic and integer-only bitwise/remainder/shift helpers now isolate
wrapping/count semantics from Rust build profiles and untrusted bytecode. Defined
original C cases and isolated archival instruction tests remain distinct; a native
remainder quotient fault becomes a typed error. See ADR 0022. This still owns no
executable bodies, runtime stacks, heap references, or host services.
Borrowed expression-operator views now classify one structural node in constant
work without allocation or recursion, retaining its original span and scoped
operands. Short-circuit logic stays separate from eager infix syntax. Primary
forms pass to their own local role classification; neither view decodes, binds,
or executes descendants. See ADR 0023.
Separate constant-cost primary views now classify all 22 primitive productions,
preserving literal polarity, grouped lists, call/reference subtrees, special roots,
size/array/animation/debugger forms. These local roles do not grant runtime
capabilities, resolve descendants, decode values, or create objects; see ADR 0024.
Expression-list lowering now owns a flat source-ordered borrowed-node index. Two
iterative immutable passes preflight complete count/work/logical metadata before
fallible reserve. It retains nesting, supports reverse iteration, and publishes no
partial result. Source order remains separate from runtime argument order and
group/vector/call arity; see ADR 0025.
Call/function-name views now retain raw names/virtual module paths, named/indirect
targets, optional receivers, invocation mode, and original argument syntax. Local
construction is constant-cost and allocation-free; ordered argument indices require
their explicit budget. No module is opened, function linked, receiver evaluated,
or thread scheduled. Those semantics stay in later compiler/runtime layers; see ADR 0027.
Reference views now preserve identifiers, dollar-prefixed names, fields, indexes,
and reserved .self members through local borrowed roles. Nested objects/index
expressions remain unresolved syntax. No slot, scope, mutability rule, object ID,
or runtime authority is selected. Reserved .self stays distinct from ordinary
.Self field syntax; see ADR 0028.
The scalar execution boundary now owns immutable verified instruction programs.
Instruction count/logical storage admission, full target scans, and iterative
reachable stack-flow checks precede proof publication. Execution borrows that
proof, admits stack storage, and charges fuel/peaks on successful and failed work.
Constants/operands remain outside Debug/errors. This typed profile has no native/
serialized bytecode ABI, source compiler, object heap, frames/calls, scheduler,
host services, or server state; see ADR 0029.
An explicit restricted source compiler now sits above typed function/expression
views and that kernel. It lowers zero-parameter empty/single-scalar-return bodies
iteratively, binds private branch relocations, resolves owned code/maps, verifies,
and performs separately budgeted pure validation before publishing a function.
Unsupported syntax fails explicitly, including in unreachable branches. Source
association and prior module/name/developer binding stay with callers. No module
activation, general statements/values, calls/frames, scheduler, or host authority
is introduced; see ADR 0030.
Borrowed statement views now expose local assignment/update/return/wait/call/event,
control-flow/body/label, and developer/debugger/profile roles. They preserve both
supplied wrapper and direct operation nodes, so the restricted compiler can reuse
return classification without changing source maps. Descendants, event-specific
lists, and for-condition/update options remain unresolved. Separate flat body
indices admit both spine passes, count/address arithmetic, and logical node bytes
before allocation; failures retain actual work/admitted counts. Nested bodies
stay separate. No local binding, lvalue/scope legality, developer activation,
general-body execution, or scheduling follows from this layer; see ADR 0031.
The shared scalar executor now supports immutable parameter-prefix/local layouts
and checked local load/store operands. Verification admits total locals and checks
all local indices, including unreachable code; its stack proof does not establish
source definite assignment. Each invocation separately admits exact internal
argument count, local count/bytes, and operand-stack storage before local allocation.
Arguments copy into private initialized slots; others remain uninitialized until
stored. Initialized Undefined remains a value, while never-initialized reads fail
at their instruction. Invocations share only immutable code/layouts. Local counts/
logical bytes and failed admission/work stay visible without source/value leakage.
Exact arity is an internal contract, not native GSC adaptation. Source local binding,
call frames/stack, objects and scheduling remain separate; see ADR 0032.
A separate straight-line scalar-local source profile now binds parameters and
plain-name assignments through a fallibly grown randomized map. Slot assignment
and diagnostics use source order, never map iteration. New targets remain
uninitialized during RHS lowering; stores initialize, and reassignment preserves
existing initialization. Source reads, even in skipped logical branches, must
have initialized bindings. Binding count/visited identifier bytes and logical pair
payload are admitted alongside compiler work/tasks/code/maps; table capacity/
control overhead and body/kernel scopes remain separately identified. Results own
code/maps with a profile tag and optional validation accounting. The original pure
profile pre-executes under its budget; parameter/local compilation publishes a
proof without fabricated argument execution. Unsupported statements/targets/values,
nonfinal returns and duplicate parameters fail explicitly. Full native binding/
arity, control-flow facts, calls/objects/scheduling and activation remain open;
see ADR 0033.
Scalar-local compound assignments now read an initialized plain target, lower its
RHS, emit the shared numeric infix instruction, and use the existing checked store/
initialization task. Original compound-token spans locate numeric/type failures.
The profile has no side-effecting RHS calls or object/index targets, so full native
reference evaluation order remains separate. No new numeric implementation or
resource namespace is added; postfix updates remain distinct; see ADR 0034.
Plain-name postfix statements now use the same checked initialized-name/store
path with a dedicated unary integer-only instruction. Its value helper reuses
wrapping integer arithmetic and rejects every float. Verification/stack effects,
fuel/error accounting, source update-token maps and name/storage quotas remain
shared. Guarded source profiles tie grammar tags/compiler emission to native VM
integer checks/word operations without executing that interpreter. No object/index
reference, postfix-expression, native unwind, modern numeric or control-flow
semantics is inferred; see ADR 0035.
Scalar invocation now has an explicit legacy script-argument entry point alongside
the internal exact binding contract. Already parameter-ordered supplied values
fill the prefix; only missing parameters receive initialized Undefined, while
other locals remain uninitialized. Excess arguments fail. The existing admission
plan bounds the single local vector before allocation; no temporary padded
argument vector is created. Source guards cover selected compiler/VM setter/
marker/check roles and undefined child-variable creation evidence. These policies
do not define builtin signatures, expression evaluation/push order, native stack
markers, receivers/frames or error unwinding; see ADR 0036.

Scalar-local blocks/conditionals share one function-local name table with wide
stable slots, plus flat owned initialization snapshots and structural reachability.
The iterative task stack lowers both source branches; it restores incoming facts
before else and merges only paths reaching the join. Return terminates its path;
the restricted profile still requires return to be final in its immediate list.
Conditional and unconditional relocations are private single-use tokens, resolved
to real instructions and independently verified before code/maps publish.
Snapshot flag payload has a separate live-byte quota and shares compiler metadata;
initialization/copy/restore/merge visits share global work. Capturing charges two
flag visits per slot (initial fill and indexed copy); restoring/merging charges
one visit per current slot. Logical work/payload accounting does not measure hash
capacity iteration, allocator/RSS or aggregate CPU. Constant-condition folding,
full native scope/error-order parity, loops/switch and richer values remain open;
see ADR 0037. Original pure-return validation remains unchanged.

Scalar while lowering uses flat condition/body/end tasks and a private loop label
for an already emitted real condition instruction. Consuming numeric false
branches exit; absolute verified jumps reevaluate the condition. Execution fuel
bounds endless loops, independently of compiler/snapshot admission. The body is
checked against first-iteration incoming facts; restoring those facts at the exit
keeps body-only assignments from initializing a zero-iteration path. This
conservative structural policy does not implement native constant folding or loop
watchdog timing. All snapshot/name/metadata scopes remain shared. For, break/
continue, switch and full native scope/runtime parity remain open; see ADR 0038.

Scalar while break/continue now use the nearest active compiler context. Saved
outer contexts live inside flat end tasks; transfer paths terminate structurally
and do not constrain conditional joins. Continue jumps to the real condition
head. Break sites link through private pending instruction records and bind once
to the loop exit with checked monotonic indices and globally charged visits.
Complete pending-record/task sizes share existing metadata admission; no separate
fixup vector or native fixed case array is introduced. Emission/admission and
binding/resolution now occupy separate leaf modules. Code publication still
requires independent kernel verification. Function-wide scalar slots imply no
native block-variable cleanup; full scope/for/switch/runtime parity remains open.
See ADR 0039.

Scalar for now shares generalized loop contexts and transfer records. Flat tasks
separate once-only initializer, condition, body, update and backward jump. Deferred
continue sites bind to update start (or the real backward jump for empty updates).
Admitted snapshots intersect continuing body paths with fallthrough before update
reads; break/return paths are excluded. First/merged continue facts are consumed
and released through existing global snapshot/work quotas. An unreachable update
checks incoming facts, and exit facts conservatively permit zero iterations.
Initializer/update syntax is restricted to scalar assignment/update/empty forms.
Stable slot allocation follows deterministic lowering order, without promising
native local ordering or a bytecode ABI. Selected grammar/compiler guards and
source tests do not establish full native scope/constant folding/unwinding parity;
see ADR 0040. No additional fixup allocation or recursive owner is introduced.

Legacy vectors now have standalone fixed-size owned value/operand/pair/operation/
error helpers in the script legacy module. Numeric pair coercion broadcasts while
preserving original operand kinds so addition can enforce its separate mixed-type
rejection. Coordinate arithmetic and exact vector equality remain distinct from
scalar tolerance and from any future modern numeric/world-coordinate profile.
Inputs/results are inline copies; Debug and errors redact data, without pool or
reference IDs. A bounded original-C helper oracle stubs pool/ref/string/error
services and intercepts errors; this is not native VM/cleanup execution. Kernel
values and source compilation remain scalar pending separate construction/order/
storage-budget integration. See ADR 0041; no serialized vector ABI is introduced.

The primitive kernel now stores inline legacy vectors and a typed MakeVector
instruction with explicit bottom-to-top z/y/x stack order and x/y/z coordinates.
Verification admits its three inputs
and one output; shared opcode fuel and size_of-based code/stack/Option-local
budgets cover complete enlarged payloads. Focused numeric-value dispatch keeps
vector/scalar arithmetic and comparison outside stack/opcode control. Constants,
locals, duplicate, arguments and return retain ownership through fixed copies;
there is no vector heap/ref pool. Error kinds redact data. Restricted source
programs can receive vector arguments, but three-element source constructors
remain unsupported. This defines no native/serialized bytecode ABI, source
evaluation order or whole value/heap/VM contract; see ADR 0042.

The growing compiler now exposes CompiledFunction, FunctionCompilationBudget/
Usage/Resource, FunctionCompileError/Kind and FunctionSourceProfile. Its implemented
entry points are compile_scalar_return_function (PureScalarReturn) and
compile_primitive_function (PrimitiveStatements). Leaf modules/private data and
test helper names follow the same function terminology. No compatibility alias,
duplicate implementation, profile, language/runtime behavior or wire/bytecode
format is added by this unreleased API cleanup. See ADR 0043.

Primitive source vector construction now lives in a focused function_group leaf.
The builder stores the explicit source profile once; the same value controls
accepted groups, publication validation and returned profile metadata. Exact
three-element shapes use fixed charged child probes and existing admitted flat
tasks, evaluating z/y/x into reversed MakeVector slots. Singleton groups remain
unchanged; PureScalarReturn rejects triples. No list allocation/native temporary
array or recursive owned tree is introduced. Source maps retain whole groups and
component/operator spans. Selected pinned prepend/grammar/compiler/VM roles are
guarded, without native execution/constant-folding/cleanup claims. Primitive source
still verifies without fabricated invocation arguments; see ADR 0044.

Vector reads now share an immutable helper and primitive Index dispatch leaf.
Index consumes the prior index and top object, copies a float coordinate and
retains fixed redacted errors. Actual code/stack/local sizing and fuel stay shared;
there is no indexed heap or new side allocation. A focused source reference leaf
handles names/index reads with index-first tasks and original bracket/operation
maps. Named lookup still uses existing legacy identity/admission. Indexed lvalues
remain unsupported, preserving the selected legacy vector write rejection.
Actual Scr_EvalArray vector-branch fixtures use bounded stubbed object/array/string/
pool/ref/error services; helper and kernel coverage are counted separately. The
common vector oracle builder preserves older fixture bytes. See ADR 0045; this
does not implement general arrays/strings/heap or native cleanup/unwinding.

Primitive comparison dispatch now has a separate kernel_comparison_values leaf.
It preserves numeric/vector helpers and admits only equality/not-equality for two
Undefined values; mixed/ordered Undefined pairs fail with redacted UnmatchingTypes.
Arithmetic, truth, instruction proof, source maps and memory/fuel admission retain
their existing contracts. Actual C-helper outcomes cover both operand orders;
the stubbed native service boundaries remain explicit. See ADR 0046.

Primitive direct calls now live under execution/calls. Immutable module proof
resolves every typed call target against a complete borrowed inventory,
including unreachable instructions, before copying/publishing all functions.
Inventory/functions/code/work and overlapping table/retained-code/verifier
payload share global admission; individual stack/local proof limits remain
explicit. Standalone proof rejects unresolved calls. Runtime frame stepping is
shared with standalone execution; an iterative frame vector supplies copied
locals, aggregate reserved operand/local/frame payload limits, global fuel and
separate conservative setup work. Return releases frame admission before reuse;
failed invocation drops private state. Compact fixed errors retain function/PC,
global work and peak admitted counts without allocating error boxes. This API
does not resolve source imports/names, activate modules, reproduce native call
ABI/cleanup or grant host/scheduler authority. Excess arguments now fail at the
executed caller location after argument instructions and before callee allocation/
body, preserving skipped calls. Stack operand proof remains mandatory and parameter
count is distinct from total locals. See ADRs 0047/0048.

Bound primitive source modules now share the existing function lowering pipeline
through one private mode. Isolated modes own their separate proof/validation
settings; bound mode borrows one local target scope and caller visibility. A shared
FunctionLoweringBudget supplies only code-generation limits. The internal expression
list cursor now serves both structural indexing and admitted flat call lowering.
No complete target-table clone or temporary argument array is allocated per call.
The bound_module subfolder separates inventory, staging, metadata formulas, global
admission, source maps/errors and complete proof publication. Declaration metadata
retains disabled definitions and explicitly maps original indices to active entries.
Kernel proof remains a separately admitted owned-code/verifier scope overlapping
staged compiler input. No aggregate RSS or host authorization is implied. Empty
inventories preserve metadata without fabricated code. See ADR 0049; imports,
runtime module ownership/reload, heap values, builtins and scheduling remain open.

Opaque function references combine a checked nonreused module identity and wide
index. A small atomic counter mints identities only after complete module proof;
it owns no game/script state. Private construction prevents index/owner forgery,
and references do not keep modules alive. Reference instructions validate targets;
indirect instructions validate checked argument-plus-callee stack effects and
invoke only within the owning module. Both paths share iterative frame admission,
arity, global fuel/setup and live-payload budgets. Source named calls and reference
values share one canonical resolver; indirect lowering uses admitted tasks to
evaluate arguments before the target, without a temporary argument array.
Comparison dispatch isolates function identity from numeric/vector/Undefined rules.
See ADR 0050; imports/reload, receivers/self, closures, heap and scheduling remain open.

Pure builtin instructions form a sealed typed subset (isint/isfloat/isvector/isstring),
separate from script-call frames and any future host-service capability boundary.
Bound source first resolves active canonical script names and visibility, then
permits only these known pure calls. Function-reference construction stays script-
only. Runtime checks arity after all arguments evaluate and shares existing stack/
fuel/storage admission without allocating an argument array or callee frame.
Source maps/compiler quotas remain unchanged. ADR 0058 adds plain-only isstring
with actual native helper/type/result evidence. ADR 0059 adds localized tag 3 to
the replay subset while retaining its false isstring result. See ADR 0051; this is not a general
plugin/callback system, heap, scheduler or game-service implementation.

Integer switches keep case records in admitted control metadata and save outer
contexts in flat tasks. An explicit nearest-break owner coexists with the active
loop; switches do not replace continue destinations. Label entries merge incoming
and fallthrough facts, while exits merge breaks/fallthrough and the unmatched path
only without default. All payload/scopes must close before publication. Out-of-line
balanced dispatch uses fixed checked instruction footprints, sorted full-word
keys, shrinking child ranges and existing typed jumps/comparisons. Key expression
evaluation occurs once; strict integer admission prevents float coercion. The full
int32 domain and wide case counts deliberately exceed native encoded variable-ID/
serialized-short bounds. They define no external ABI. See ADR 0052; string/heap/
full native scope/diagnostic/cleanup compatibility remains open.

Offline --execute-gsc reuses snapshot/index/parse/declaration/binding phases with
the inspector, then delegates complete primitive compilation/execution and typed
stdout output to separate application leaves. It rejects unresolved directives,
selects canonical active main through explicit declaration-to-entry mapping, and
uses the existing empty script argument prefix/global runtime budgets. Operational
diagnostics retain original source locations and aggregate counters without source,
paths, spellings or result values. Stdout is explicit operator data, not a wire/
plugin ABI; function results omit identity. Write/flush errors remain typed. The
library keeps filesystem/logging/socket ownership outside its boundary. See ADR
0053; host/game services, full mods and a joinable server remain open.

## Future custom client

The future client must have an ordinary CoD4/CoD4X connection path and must not
require an ApX server. The compatibility matrix must enumerate actual server
releases/protocol profiles; no claim about "all servers" is established by one
baseline or by changing a displayed version number.

Legacy handshake, command IDs, snapshots, downloads, authentication, checksums,
and prediction behavior remain versioned compatibility paths. Client and server
share their definitions; server-only implementation details must not leak into
wire types or become required by the client library.

ApX capabilities are optional and negotiated per connection. Define explicit
identifiers, versions, size limits, authorization, downgrade behavior, and
unsupported-capability handling before allocating wire IDs. Preserve the ordinary
handshake with peers that do not advertise extension support. Do not append
unknown opcodes to legacy packets or overload existing fields heuristically.

Until negotiation has its own compatibility tests, no extension wire format or
plugin ABI is considered defined. Future client security must independently
validate downloads, map/script data, server commands, and presentation content;
a remote server is not a trusted process.
