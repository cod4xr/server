# ADR 0018: Readiness-driven UDP in the development process

Date: 2026-10-31

Status: accepted and implemented; full-workspace validation recorded separately.

## Context

Repeated release smoke tests lost the valid-refusal exchange after a negative
cookie receive wait. Increasing test timeouts did not resolve it. Controlled local
repetitions found server received=2 rather than 3, one invalid cookie, and no send
error. Failures clustered near receive-timeout timing; changing cookie sign or
payload did not remove them. This is evidence of missing datagrams, not proof of
a particular OS/kernel defect. No firewall or machine settings were changed.

The development process used repeated blocking receives with a 50-ms socket
timeout to check shutdown/timers. Readiness notification removes timeout-cancelled
receives while retaining bounded timer/shutdown checks and socket-free core logic.

## Decision

Own one Mio nonblocking UDP socket, Poll registration, and eight-entry event buffer
in ReadySocket. Register read interest only. Runtime attempts receives until
WouldBlock and then polls, accepting spurious wakeups as another reason to attempt
a nonblocking receive. Check lifecycle/metrics before every datagram; sustained
traffic cannot postpone those checks through an unbounded inner drain loop.

Poll waits are capped at 50 ms and the remaining process/metrics deadlines. Poll
interruption returns to lifecycle checks. No SO_RCVTIMEO is set and no outstanding
receive is cancelled on timer expiry. Poll is an OS wait, not a sleep/busy loop or
simulation clock. Event capacity is a small fixed control allocation for one
registered socket, not an entity/asset/client capacity limit. See the upstream
[Mio polling contract](https://docs.rs/mio/1.2.4/mio/) and packaged Poll rustdoc for
spurious readiness and draining to WouldBlock.

Retain bounded receive/output buffers, cookie/budget semantics, packet accounting,
aggregate logs, and explicit refusal of gameplay. Nonblocking send failure remains
a counted drop with no retry queue. Reliable gameplay transport/pacing, shutdown
wakers, multi-socket scheduling, and operational load/soak evidence remain separate
work; this is not a final simulation/executor architecture.

## Dependency review and alternatives

Pin Mio 1.2.4 with default features disabled and only net/os-poll enabled, solely
in the application. Reviewed Cargo metadata/source: Tokio maintainers, MIT license,
MSRV 1.71, and no build script. It adds no executor, TLS, timers, macros, or default
logging feature. libc/windows-sys versions are reused from the lockfile; Windows
API feature unification rebuilds affected packages in the normal target directory.
New lock entries are Mio and WASI 0.11.1+wasi-snapshot-preview1. WASI is target-
conditional and not built on current Windows/Linux hosts; its metadata specifies
MIT/Apache-2.0 alternatives (including an LLVM-exception option) and no build script.
WASI is not a supported/tested product target. No third-party version was upgraded.

Platform FFI/unsafe implementation remains inside the reviewed dependency. Project
unsafe code stays forbidden; protocol, transport, and deterministic core gain no
I/O dependencies. Dependency policy explicitly allows Mio only in cod4x-server.
Primary project source/maintenance information: [tokio-rs/mio](https://github.com/tokio-rs/mio).

More blocking-timeout tuning would retain the failing mechanism. Sleeping after
nonblocking reads would add polling latency/CPU tradeoffs without readiness.
Handwritten platform FFI would require unsafe code and duplicate a maintained I/O
boundary. An async executor is not required to serve this concrete single-socket
probe. No old engine state or peer protocol is migrated by this change.

## Evidence and limits

The original three-packet exchange and counter/privacy assertions now repeat 16
times per smoke test without further timeout increases. A four-request queued
burst verifies draining without waiting for another readiness edge. Windows
focused debug tests and warnings-denied workspace Clippy pass. Full debug/release,
rustdoc, source/dependency policy, and Linux gates are recorded separately. Local
passing repetitions do not establish arbitrary packet-loss recovery, zero packet
loss, OS root cause, throughput, latency, or gameplay interoperability.
