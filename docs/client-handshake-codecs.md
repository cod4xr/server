# Shared client challenge codecs

Date: 2026-11-01. Scope: additive codecs in `cod4x-protocol`, not client admission,
authentication, a gameplay join or negotiated extensions.

`ChallengeRequest` encodes a nonce-only `getchallenge` OOB command. The pinned
SV_GetChallenge reads argument one with atoi and echoes it; its ordinary response
path does not consume a GUID credential. The request contains no placeholder
identity, authorization token or credentials. This is the inspected server-handler
profile, not a claim to reproduce every argument sent by the public client.

`ChallengeResponse::parse` and `parse_packet` accept only the named
`challengeResponse SERVER CLIENT 0 xproto 18` profile. Both signed values must be
complete checked decimal i32 tokens. This preserves the original response encoder
and accepts every one of its forty executed original C fixture datagrams. The
18 marker is a compatibility signal; it does not select a gameplay join version.
Unmarked legacy replies and other marker versions remain unsupported until a
separate profile is source-grounded and tested. There is no silent fallback.

Complete-packet decoding uses the shared envelope and command-line primitives.
One terminal newline or NUL is accepted, but bytes after that delimiter reject.
Command spelling is ASCII-case insensitive; ordinary tokenizer quote/comment
rules remain shared. Numeric overflow, numeric prefixes followed by junk and
noncanonical plus signs/embedded whitespace reject instead of adopting atoi's
permissive malformed-input behavior. Cookie and nonce values stay out of Debug
and fixed errors. Endpoint, nonce correlation, pending state and time/resource
limits are responsibilities of the client runtime.

Six new tests cover all forty C responses, exact request bytes and every output
capacity prefix, every required response truncation, terminal/trailing data,
numeric domains and unsupported profile rejection. Existing challenge encoder
tests remain unchanged. The native wire generator still checks all 168 fixtures.
These tests do not execute the native challenge handler or full connect lifecycle.

The isolated worktree uses the operator's existing Cargo target. Local Cargo
configuration is ignored by Git; no alternate target directory is created.

Local acceptance: 71 protocol tests (including the doctest) pass in debug and
release, all-target Clippy and rustdoc pass with warnings denied, formatting and
repository policies pass, and all 168 native wire fixtures still match. Hosted
CI/Linux and real ordinary-server/client joins remain unrun.
