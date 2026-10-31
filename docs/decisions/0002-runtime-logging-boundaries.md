# ADR 0002: Deterministic services and application-owned structured logging

Date: 2026-10-31

Status: accepted and implemented for the query-only runtime.

## Context

The rewrite needs real local network execution, reliable diagnostics, and bounded
behavior under malformed or abusive traffic. Its shared protocol must remain
usable by a future client without owning sockets, process handlers, or logging
sinks. Logging must not create an attack-amplified output stream or expose secrets.

## Decision

Place deterministic discovery/policy services in `cod4x-server-core`, driven by
explicit byte inputs, caller storage, peer IP, and monotonic time. Add `apps/server`
to own UDP sockets, shutdown signals, runtime counters, and logging installation.
The first runnable mode requires `--query-only`, binds loopback by default, and
advertises zero playable slots. It does not simulate unimplemented gameplay.

Use the maintained tracing ecosystem behind `cod4x-observability`: `tracing`
0.1.44, `tracing-subscriber` 0.3.23, and `tracing-appender` 0.2.5. These direct
dependencies are MIT licensed. Use JSON records, explicit filters, optional daily
files with retention, bounded lossy sink queues, and retained flush guards.
Expose queue drops and aggregate rejected traffic instead of logging every packet.

Use `ctrlc` 3.5.2 (MIT/Apache-2.0) for OS signal integration in the process only.
Use `serde_json` 1.0.154 (MIT/Apache-2.0) for structured-output assertions in tests;
the subscriber also requires JSON serialization transitively. Exact direct
versions and the full resolution are recorded in Cargo.lock. Review any change
to transitives/build scripts and future dependencies through the policy file.

Protocol remains free of third-party dependencies. Server-core initially depended
only on protocol; ADR 0006 subsequently adds HMAC/SHA-256 for endpoint cookies.
Neither depends on process signal or sink crates. The signal callback
shares only an atomic stop flag with the runtime; mutable service state has one
owner. No async executor or thread-per-client architecture is introduced.

## Alternatives and consequences

Handwritten JSON/file logging would duplicate mature formatting, filtering,
rotation, and flushing behavior. Synchronous sinks would allow slow output to
block networking. Unbounded queues would turn floods into memory exhaustion.
Bounded dropping requires explicit monitoring and cannot guarantee durable logs.
Daily retention bounds file count, not disk bytes; the documented operational
limit remains open for later work.

## Validation

Independent C fixtures establish channel envelope, tokenizer, and info-string
semantics. Deterministic tests cover per-IP/global budgets, mapped addresses,
monotonic-time regressions, and atomic failures. Real process/socket tests check
query responses, JSON lifecycle events, aggregate counters, nonzero error exits,
and exclusion of wire secrets. File tests verify flush, filtering, and span fields.
Neither a passing query nor these logs establish a playable game-server milestone.
