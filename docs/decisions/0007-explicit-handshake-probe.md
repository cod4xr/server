# ADR 0007: Explicit bounded handshake probe before gameplay admission

Date: 2026-10-31

Status: accepted and implemented; Windows workspace gates passed.

## Context

Challenge/cookie exchange needs executable network evidence while maps, simulation,
scripts, and player admission are absent. Sending a successful connectResponse now
would misrepresent readiness. The query-only runtime has an explicit established
contract and should keep answering only discovery requests.

## Decision

Add an explicit `--handshake-probe` development mode, mutually exclusive with
query-only/offline verification. It binds loopback by default, shares the existing
bounded UDP/shutdown/logging runtime, issues challenges, validates echoed cookies,
and always refuses connects with a gameplay-unavailable error. It never allocates
players, authenticates accounts, sends connectResponse, or contacts remote auth/
master services. Ordinary query-only mode retains its behavior.

The process obtains one OS-random key with getrandom 0.4.3 and protects the source
array with zeroize 1.9.1. Both already existed transitively; direct use and licenses
are reviewed in prior dependency ADRs. Enable getrandom's std error integration;
add no persistent keys, fallback randomness, or deterministic production key flag.
Entropy/key failure aborts before binding. MAC state stays inside server-core.

`HandshakeProbe` is a socket/entropy/log-free core service. Syntax is bounded before
address-state allocation, and the existing per-IP/global reply budgets precede MAC
work. Invalid cookies are silent drops, consume budget, and never create client
state. Failed output writes preserve caller bytes. Receive time is sampled after
the socket read for rate accounting. No implicit cookie expiry is introduced.

The response codec exactly follows pinned SV_GetChallenge -> NET_OutOfBandPrint,
including signed decimal values, the fixed suffix, OOB marker, and absence of a
newline/NUL terminator. Original functions produce 40 boundary/echo datagrams.
Authorizer/cookie platform hooks are test stubs; authorizer branches abort if used.
These fixtures validate formatting, not remote authorization behavior.

The pinned legacy client disassembly sends `getchallenge 0 "<guid>"` and consumes
the first response challenge. Preserve its extra argument without logging it.
The probe accepts absent/valid signed-decimal nonce fields; malformed/overflowing
nonce text is rejected rather than atoi-prefix coercion. Full version/profile
interoperability remains unverified and must not be inferred from xproto 18.

## Evidence and consequences

Deterministic core tests cover nonce boundaries, endpoint binding, honest refusal,
capacity/clock/rate drops, and malformed input without output/budget mutation.
A real UDP child-process test exchanges a challenge, submits a bad-cookie connect,
then a valid-cookie connect, checks refusal and counters, and excludes nonce/GUID/
password inputs from logs. Aggregate `probe_challenges` and
`probe_verified_rejections` count prepared replies; `replied` counts successful
socket sends. No per-request logs or remote metric labels are added.

This is a diagnostic interoperability step, not a joinable server. It introduces
no placeholder gameplay readiness state. Real admission needs the remaining
version/profile, resource, game initialization, asset, and scripting contracts.
