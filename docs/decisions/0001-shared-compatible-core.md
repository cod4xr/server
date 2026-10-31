# ADR 0001: Shared compatible core with separate product runtimes

Date: 2026-10-31

Status: accepted as the initial direction; implementation is incremental.

## Context

The requested product is a standalone Rust CoD4X server rewrite, independent of
the ApX Deathrun workspace. Existing clients, maps, and mods are compatibility
targets. A later custom client must support ordinary CoD4X servers while allowing
optional ApX-specific features. The upstream engine includes 32-bit C/C++ and
assembly; preserving its implementation structure would retain the problems the
rewrite is intended to address.

## Decision

Use one Rust workspace with explicit library boundaries. Both products consume
shared protocol and simulation code. Protocol libraries perform bounded,
deterministic parsing and encoding without product/runtime dependencies. Runtime
ownership, scheduling, sockets, rendering, and operating-system concerns stay
outside the shared wire definitions.

Start with a pinned upstream revision and independent test oracles. Preserve
valid protocol behavior while documenting safer malformed-input handling.
Compatibility is measured separately for each protocol/version/profile and
requires actual peers before it can be claimed.

Represent ApX behavior as optional capabilities above the compatible baseline.
Do not reserve wire IDs, invent a handshake, define a plugin ABI, or add generic
extension frameworks before requirements and interoperability tests justify them.
An ordinary server/client must remain usable without any ApX capability.

Create crates only as implemented boundaries emerge. The initial crate owns
message primitives; future protocol codecs remain cohesive leaf modules until
dependency or ownership concerns justify a split. Keep handwritten source files
below 400 lines, aiming for 250, with automated checks.

## Alternatives considered

- Reproduce the upstream file structure: rejected because it preserves large,
  mixed-responsibility units and server-specific coupling.
- Separate server/client protocol implementations: rejected because shared
  formats would drift and require duplicate compatibility fixes.
- Wrap or link the original engine as the final runtime: rejected because that
  does not fulfill the Rust rewrite. A disposable reference process remains
  useful for differential testing.
- Create every anticipated engine crate immediately: rejected because empty
  boundaries conceal missing implementation and freeze untested abstractions.

## Consequences

The first deliverable is a tested foundation, not a playable server. Full
compatibility requires ongoing source research, asset/scripting work, original
peer tests, and documented profiles. A custom client can reuse codecs without
depending on server internals. Native plugin ABI/bitness and exact simulation
floating-point parity remain separate decisions.

## Validation and migration

Current protocol fixtures come from selected unmodified functions in pinned
`src/msg.c`. Rust encoding is compared byte-for-byte, and decoding is tested at
every storage split. The codec's dependency allowance is empty. CI checks Rust
quality gates, metadata inheritance, source file sizes, and dependency policy.

Further extraction into crates requires updating the dependency policy, preserving
conformance fixtures, and documenting the new boundary. The server and future
client become release candidates only after the milestone gates and named peer
matrix pass. This decision does not establish a public stable Rust API.
