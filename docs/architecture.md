# Architecture and expansion

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
