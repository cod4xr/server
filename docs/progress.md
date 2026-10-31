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

Borrowed connect parsing now passes protocol debug/release tests and Clippy.
Four tests include twelve new original C tokenizer/lookup/decimal vectors, making
128 wire fixtures total. Signed-short/unsigned qport spellings normalize to raw
16-bit routing values; unknown userinfo is retained; duplicated critical fields
and undefined/overflowing atoi-style inputs fail explicitly. Request Debug omits
credentials and challenge echoes. sv_client.c is now hashed in upstream.lock.json.
This parser has no admission/cookie/account-auth behavior.

The endpoint-cookie core is now implemented using pinned RustCrypto hmac 0.13.0 /
sha2 0.11.1 with reviewed zeroize features and eleven MIT/Apache-2.0 additions, no
new build scripts or prior version changes. Three Windows core debug/release tests,
Clippy/rustdoc, and 24 independent Python HMAC vectors pass. It binds observed
address/port and native IPv6 scope, canonicalizes mapped IPv4, ignores flowinfo,
and verifies exactly four bytes without exposing key/echo through Debug. It is a
limited return-path check, not account authentication. No implicit expiry or new
wire format is introduced. Application OS entropy is integrated in the probe
described below. See ADR 0006 for the rationale and reviewed dependencies.

The explicit `--handshake-probe` mode is implemented and passes full Windows
workspace formatting, Clippy, debug/release, rustdoc, and source/dependency policy.
Four core tests and two real child-process tests prove challenge exchange, bad-
cookie drops, explicit refusal after valid proof, bounded budgets/clock handling,
mode separation, and sanitized aggregate counters. It never admits players or
sends connectResponse, and never contacts auth/master backends. Process entropy
uses getrandom 0.4.3 with a Zeroizing source key; no new transitive versions were
introduced by their direct application use. See ADR 0007.

Forty datagrams now come from complete original SV_GetChallenge and
NET_OutOfBandPrint functions with aborting/unexercised auth stubs and synthetic
cookie hooks, making 168 wire fixtures. All six original C generators and both
independent Python fixture generators passed again. Client handshake disassembly
is now source-hashed; it sends `getchallenge 0 "<guid>"` and consumes response
argument one. Its extra GUID must never enter logs. No live game-client join has
been attempted or established. Current source policy: 142 files, seven crates,
largest source still the 287-line generated codebook.

IBSP container validation is implemented in content/bsp and passes full Windows
workspace gates (155 source files, seven crates, largest still 287 generated lines).
Ten tests include 280 original C lookup/count/presence/checksum cases across all
forty supported container versions, checked wide offsets, exact budgets, malformed
and mutating inputs, 1,000 directory entries, first-match duplicates, empty/unknown
kinds, zero-copy views, independent map instances, and redacted Debug output.
com_bsp_load_obj.c is source-hashed; see ADR 0008. This is metadata, not decoded
collision or a joinable map. Seven C oracle generators now exist.

Unsigned version-5 fastfile envelope/decompression is implemented and passes full
Windows workspace gates (167 source files, seven crates). Five content tests and
three child-process tests use fifteen independent Python zlib fixtures. Exact
budgets, output length/checksum/end/trailers, all small-sample truncations, mutations,
forged huge declarations, output ownership, and log redaction pass. Direct flate2
use adds no package versions or backends. See ADR 0009 and docs/validation.md.
`--inspect-fastfile` is offline and reports envelope integrity only. Shared bounded
file snapshots replace archive-specific file I/O; arbitrary OS strings/paths are
excluded from snapshot diagnostics. Actual Rust inspection of both stock samples
below matched independent Python results without copying or extracting content.

Root asset-directory/script-string prefix decoding is implemented and passes full
Windows workspace formatting, Clippy, debug/release, rustdoc, and repository gates.
Six content tests and two additional process tests use eleven independent Python
fixtures. Raw unknown kinds/references remain explicit; inline strings/remainders
borrow the stream, null/inline-empty differ, virtual alignment does not consume
input, and five configurable metadata resources have exact boundary coverage.
Synthetic catalogs preserve 70,000 script slots and 70,000 asset records; these are
metadata results, not legacy handle/pool or gameplay compatibility. See ADR 0010.
`--inspect-zone-prefix` reports counts only and states bodies/references remain
unvalidated. src/xassets.h is source-hashed. Linux validation remains open.

Checked serialized block addresses are implemented and pass full Windows workspace
gates. Four tests include ninety original-C relative offsets across all nine
blocks, distinct null/-1/-2 classification, 28-bit boundaries, invalid tags, checked
wide spans, empty/end spans, and overflow. Address classification never produces
a native pointer; capacity is separate from loaded residency, aliases, alignment,
ownership, and target type. Eight C oracle generators now exist. Prefix metadata
still preserves unresolved words and requests header classification lazily.

The first typed raw-file-only collection reader is implemented and passes full
Windows workspace formatting, Clippy, debug/release, rustdoc, and repository gates
(198 source files, seven crates, largest still the 287-line generated codebook).
Six content tests and three child-process tests use twenty independent Python
envelopes. Whole-collection publication, root/inserted aliases, forward-null cells,
known prefix/shared/suffix names, binary/null/empty buffers, temp reuse, alignment,
work/metadata budgets, truncations/mutations, and sanitized diagnostics pass.
`--inspect-rawfiles` is offline, rejects other asset kinds upfront, and reports
logical usage without executing scripts. Stock mp_crash_load.ff is explicitly
rejected as UnsupportedAsset, with no partial success. rawfile.h and _db_load.asm
are source-hashed. See ADR 0011. Full native loader/registry differential evidence,
mixed-zone decoding, and Linux validation for this layer remain open.

The user requires a significantly expanded GSC runtime, beyond legacy parity,
with built-in language/services/tooling instead of native plugins. This is now in
AGENTS.md, docs/gsc-runtime.md, and the roadmap. A versioned modern mode must coexist
with legacy semantics; modules/collections/optional types/errors/async host I/O,
JSON/database/HTTP services, debugging/profiling, and reload are required directions,
not implemented features. Keep wide handles/budgets and reusable host boundaries.

Next: source-grounded GSC semantics/compiler/runtime foundations alongside broader
asset graphs. Read docs/gsc-runtime.md before designing the scripting subsystem.
Audit found parser/lexer actions and tables in asmsource/_scr_yacc.asm, bytecode
opcodes in scr_vm.h (0x00..0x89, count 0x8a), and wait/scheduler helpers in
cscr_vm.cpp. VM_CalcWaitTime uses SSE f2rint, which follows the current rounding
environment; Win32 Sys_SetFloatEnv selects nearest rounding. Under that profile,
positive nonzero fractional waits rounded to zero become one tick, while exact
zero remains zero. Numeric bool conversion in
Scr_CastBool accepts integers/floats only. Need independent evidence before porting;
do not claim a running compiler/VM based on these observations.
Stock main/iw_00..iw_13 contain no raw d3dbsp entries; maps live in zone fastfiles.
Read-only analysis of stock mp_crash_load.ff/mp_crash.ff confirmed IWffu100/v5,
zlib streams, a 44-byte XFile header (size, externalSize, nine block sizes), exact
total output = 44 + size, and no compressed trailing bytes. The full map stream
was 63,106,933 bytes, declared payload 63,106,889, external bytes 243,254,095;
the load stream was 921 bytes with payload 877 and external bytes 1,223,352.
No bytes were extracted/copied into the repo. db_load.cpp rejects signed IWff0100
and uses ordinary inflate for unsigned files. Preserve that explicit unsupported
case; decompression integrity must not be described as validated asset graphs.
Source defaults XBLOCK_COUNT_IW3=9, FASTFILE_VERSION=5 are format-profile bounds,
not global engine pools. Full mixed XAssetList graph loading remains unimplemented.
Read-only prefix research finds zero script strings/six asset records in
mp_crash_load.ff and 372 inline script strings/823 asset records in mp_crash.ff.
Actual Rust prefix inspection matches these counts and independent Python offsets;
stock contents remain outside Git. Broader graph validation, block-relative
relocation, and other typed schemas remain open. RawFile source/disassembly uses a
12-byte name/length/buffer record; its pointer loader distinguishes -1 inline,
-2 inline-with-alias-slot, and encoded aliases. It scopes temporary block zero
separately from virtual block four. Do not conflate logical allocation alignment
with serialized input offsets or direct block addresses with alias slots.
Continue remaining delta/message primitives and coverage-guided fuzzing; existing
stress tests cover 4096 mixed inputs and all 65536 two-byte compressed inputs.
Continue
connection/authentication and full packet profile research, then asset and GSC
work according to dependencies. Preserve the full content catalog and keep all
legacy wire limits separate from internal engine capacities, per docs/limits.md.

Do not stop merely because a checkpoint passes or ask again for authorization
already given. Do not claim the whole server or its limit-removal goals are done.
