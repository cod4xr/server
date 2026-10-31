# Active implementation record

This is continuing work, not a completed server rewrite. The user explicitly
requested continuous progress and added structured logging, future custom-client
support, and removal of inherited engine/content limits as requirements.
Development logs must be readable/colorful by default. The user also explicitly
requires only the normal target directory and stopping the verified old local
development process before rebuilding; never create alternate build caches.

An attempted cleanup of target/dev-preview (~346 MiB) was twice rejected by
automatic approval review with only "blocked by policy" as the reason, including
an exact literal-path retry after checking its resolved location. Do not bypass
that block. The separate WSL validation directory was already absent when checked.

## Current tree

- `crates/protocol`: byte/bit messages, envelope classification/framing, command
  extraction/tokens, info strings, static Huffman codec, and legacy index streams.
  Keyed legacy player-command decoding is now implemented as a separate module.
- `crates/transport`: ordinary channel send sequences, borrowed transmission,
  bounded reassembly, cancellation, stale/duplicate/reorder handling.
- `crates/server-core`: deterministic query-only discovery and bounded global/IP
  budgets with explicit monotonic time and mapped-address normalization.
- `crates/observability`: structured configurable console/file sinks, daily
  retention, bounded nonblocking queues, drop accounting, and flush lifetime.
- `apps/server`: explicit query-only loopback UDP process, controlled shutdown,
  aggregate traffic/lifecycle logging, and real child-process/socket tests.
- `crates/assets`: configurable encoded-byte ownership, exact names, wide opaque
  handles, stale/cross-catalog rejection, slot reuse, and logical usage metrics.
- `crates/content`: bounded immutable IWD/ZIP64 index, virtual byte paths,
  stored/DEFLATE output, original override precedence, full payload verification,
  and legacy seeded CRC checksums. Application has an offline `--verify-iwd` mode.

The reference checkout is `../cod4x-reference`. Upstream source and all generators
are pinned/hashes checked. There are no game assets, ApX code, private configs,
remote deployments, client implementation, decoded map loader, game simulation,
or GSC VM.

## Verified checkpoints

Full protocol/transport/query/logging checks passed Windows and Debian/WSL in
debug/release, including warnings-denied Clippy/rustdoc and all four original C
artifact generators. Linux SIGTERM shutdown passed. The later pretty-log change
passed Windows and Linux debug/release process/file tests. The Windows recheck
used the normal target directory after the previous process had exited.

Independent evidence now includes 64 mixed-message packets, 116 wire/index/string
fixtures, 257 Huffman symbol entries, 32 compressor/decompressor streams, and 49
XOR cases (48 canonical plus an explicit stale-empty-command bug reproducer).
Source size remains below 400 lines, with a 287-line generated codebook and
handwritten files near/below the 250-line target. Hosted GitHub CI has not run.

## Immediate follow-on work

Asset catalog debug/release tests and warnings-denied Clippy passed on Windows.
Ten tests cover 6,500 retained assets (5,000 images with 50 prestige entries and
1,500 models with a 100-character roster), resource failures without partial
registration, exact names, redacted Debug output, and 10,000 unload/reload cycles.
Initialized slots and byte budgets do not measure total allocator/RSS overhead.
Linux validation for this new crate remains open; do not create another cache.

The IWD implementation passes Windows debug/release tests, including 66,000 ZIP64 entries,
compressed expansion/corruption, hidden corrupt duplicates, exact admission
budgets, path/header rejection, and real offline verifier processes with sanitized
logs. Eighteen independent Python ZIP fixtures and 32 original C path/checksum
vectors are checked in. All four older C generators still match. The complete
workspace passed Windows Clippy/debug/release/rustdoc before full-verification/CLI
integration; all affected checks also passed afterward, including compressed span
and repeated-work bounds. No new target
directories were created. New content Linux checks remain open under the one-
target policy. See ADR 0005 for reviewed ZIP dependencies and remaining costs.

Continue remaining user-command/delta codecs, mount order, and decoded map/
fastfile formats. The user asked for an ETA and completion percentage: answered
several weeks to a few months for a first joinable stock-map build, roughly
5–10% toward first join and under 5% of the complete rewrite. These are rough
planning estimates, not measured progress or delivery promises. User explicitly
said to continue working after answering; do not end at this checkpoint.

User-command decoder passes debug differential/boundary tests and workspace
Clippy. Evidence includes 512 original C cases at all 65 storage splits, every
command truncation with atomic cursor checks, and all 65,536 signed-short melee
angles compared through a C digest. Full Windows workspace debug/release,
Clippy/rustdoc, formatting, and source policy passed. Command encoder, full packet profile/state, and simulation
remain open. New source hashes include player.h and q_math.h; six C generators
now exist. No target directory or private/proprietary fixtures were added.

Read-only full verification also passed for three stock installation archives:
localized_english_iw06.iwd (7 entries/3 visible files), iw_12.iwd (33/33), and
iw_13.iwd (265/265). No bytes were copied into the repo or extracted. Independent
Python metadata CRC calculations matched the Rust legacy checksums. Exact archive
hashes are recorded in docs/validation.md; this is not decoded map/gameplay proof.

Next: research connection/challenge and protocol profile state. The source uses
an endpoint-derived 32-bit cookie in connect and Source-engine query challenges;
do not replace it with an expiring/domain-specific design without auditing those
other callers. It currently derives from a single SHA1 compression with a random
per-process secret. The response shape is `challengeResponse <server> <client>
0 xproto 18`. No authentication/connection handler has been implemented yet.
Continue remaining delta/message primitives and coverage-guided fuzzing; existing
stress tests cover 4096 mixed inputs and all 65536 two-byte compressed inputs.
Continue
connection/authentication and full packet profile research, then asset and GSC
work according to dependencies. Preserve the full content catalog and keep all
legacy wire limits separate from internal engine capacities, per docs/limits.md.

Do not stop merely because a checkpoint passes or ask again for authorization
already given. Do not claim the whole server or its limit-removal goals are done.
