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
