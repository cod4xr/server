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
`cod4x-server-core`, `cod4x-assets`, `cod4x-observability`, and the `apps/server` process. Protocol and deterministic
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

## Server ownership

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
