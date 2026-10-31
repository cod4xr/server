# ADR 0006: Opaque legacy endpoint cookies backed by HMAC-SHA256

Date: 2026-10-31

Status: accepted and core implemented; live handshake integration remains open.

## Context

The pinned server computes a 32-bit cookie from observed address/port and a random
per-process secret using one SHA1 compression operation. Connect and Source-engine
query handlers echo/compare that opaque integer. Clients cannot depend on its
internal computation or share the secret. The signed integer width is a legacy
wire boundary, not permission to treat the cookie as account authentication.

## Decision

Implement `EndpointCookie` in server-core using reviewed RustCrypto `hmac` 0.13.0
and `sha2` 0.11.1 (both MIT/Apache-2.0). Pin exact versions, disable default features,
and enable their zeroize features for supported resident hash/buffer cleanup.
No first-party cryptographic primitive, unsafe code, entropy source, socket, log
installation, or secret persistence is introduced in this boundary.

The application must supply 32 fresh OS-random bytes per process, fail on entropy
failure, and own/wipe the original key material appropriately. Library construction
checks key length and retains keyed hash state. Debug/errors expose no key or
challenge. It does not promise to erase every compiler/dependency temporary copy.

MAC inputs have a fixed internal domain, explicit IP-family byte, canonical address,
big-endian UDP source port, and native IPv6 scope ID. IPv4-mapped IPv6 shares IPv4
identity. Per-packet IPv6 flow information is ignored. Issue the first four MAC
bytes as a signed little-endian integer; verify exactly four bytes using the MAC
library's truncated-tag comparison, never a caller-selected tag length.

Preserve a single endpoint-cookie namespace across legacy connect/player/rules
query consumers. No implicit time window, per-command scope, or rotating key is
introduced; replacing the provider explicitly rotates all cookies. Endpoint
validation/rate budgets must run before state allocation. Admission, account
identity, protocol negotiation, reconnect policy, and deadlines are separate.

These are only 32-bit return-path checks under rate limits. They do not authenticate
players, encrypt traffic, prevent capture/replay, or provide modern MAC-strength
authentication. Future wider capabilities need explicit peer negotiation; do not
silently widen the legacy integer or append unknown wire fields.

## Dependency review and validation

The direct crates are maintained RustCrypto projects, require stable Rust no newer
than the pinned toolchain, and declare no build scripts. SHA-256 uses runtime
CPU-feature dispatch with Rust/intrinsics implementations; dependencies may contain
unsafe internals. Review resolved transitives/licenses/scripts before accepting
the checkpoint. The native process still owns entropy; no random fallback is used.

References: https://docs.rs/hmac/0.13.0/hmac/ and
https://docs.rs/sha2/0.11.1/sha2/ plus exact registry sources. Independent Python
HMAC-SHA256 fixtures validate 24 key/address/scope combinations and signed echoes;
tests alter every tag bit, endpoint, port, scope, and key and verify Debug redaction.
This establishes the cookie core only. Challenge emission, packet admission,
network limits, entropy integration, and live-client verification remain open.

Resolved additions reviewed: block-buffer 0.12.1, cmov 0.5.4, cpufeatures 0.3.1,
crypto-common 0.2.2, ctutils 0.4.7, digest 0.11.3, hmac 0.13.0, hybrid-array
0.4.15, sha2 0.11.1, typenum 1.20.1, and zeroize 1.9.1. All offer MIT/Apache-2.0
licensing and declare no build scripts. The prior resolution was retained.
Windows core debug/release, warnings-denied Clippy/rustdoc, and Python fixture
regeneration passed. No live peer or production authentication claim follows.
