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

The dependency-free cod4x-script kernel implements legacy numeric truth, logical
negation, integer complement, and explicit nearest-even-profile wait conversion.
Five tests include 827 original-function cases (572 numeric, 255 waits) and boundary
rejections. Full Windows workspace gates passed (208 source files, eight crates);
all nine original-source generators and independent raw-file regeneration passed.
The C oracle controls SSE rounding/FTZ/DAZ, excludes undefined wait conversions,
and records errors rather than implementing longjmp/VM cleanup. Relative results
use u64 without the old 24-bit mask; modern durations/numbers remain separate.
No third-party dependency or old package version was added/changed. See ADR 0012.

Bounded immutable script-source indexing is implemented and passes full Windows
workspace gates (216 source files, eight crates). Six tests cover raw byte locations,
LF/CRLF/standalone-CR behavior, tabs/NUL/non-UTF-8/UTF-8, exact EOF/empty/trailing
lines, borrowed spans, invalid ranges, exact budgets, independent sources, redacted
Debug, and 70,001-line input. Every offset of 256 deterministic byte patterns is
checked against a separate scan. SourceText is a frontend/tooling container, not
lexical validation, a compiled source map, a module registry, or a debugger. See
ADR 0013. Linux validation for this layer remains open.

Bounded borrowed legacy lexing is implemented and passes full Windows workspace
gates (229 source files, eight crates). Eight tests cover 724 source-derived lexical
replays, exact terminal codes/spans/trivia/case, comment modes, EOF/NUL failures,
fused error closure, work/lexeme/byte budgets, redacted Debug, 100,002-byte raw
literals, and all 65,536 two-byte inputs. Generated tables/actions and reachable
fallback invariants are checked by generate_lexical_tables.py. These are source-
table replays, NOT original x86/parser execution or another original-C generator.
An independent boundary test caught comment-mode mapping; changes are now derived
from actual source actions. Full workspace checks also caught an unexpected extra
reference in the BSP vector test, restored to its valid baseline without other
edits. No literal decoding, interning, AST, parser, or VM execution is implemented.
See ADR 0014. Linux and original-parser trace conformance remain open.

Bounded legacy literal decoding is implemented and passes full Windows workspace
gates (239 source files, eight crates). Four tests use 545 source-action/CRT replays
(522 strings, 23 numeric cases). Owned ordinary/localized bytes, r/t/n/default
escapes, checked decimal scalars, integer bools/undefined, no interning, source
immutability, exact budgets, redacted Debug, and 100,000-byte decoded strings pass.
No old 8191-byte scratch buffer is retained. These replays do not execute the
original parser/interner and are not counted as original-function generators.
Finite/in-range C-locale conversions are evidenced; undefined overflow and nonfinite
literal results fail explicitly. Exponent notation is not a single legacy float
token. Test helpers enforce full single-lexeme input rather than accept a prefix.
See ADR 0015. Linux, parser/interner, and whole-program execution remain open.

The first release full-suite run had a probe socket receive timeout; an isolated
run passed. The smoke test now allows five-second positive receives and a fifteen-
second child safety bound, with identical three-packet/count/privacy checks and
immediate max-datagrams exit. Full debug/release workspace reruns passed. No runtime
or cookie semantics changed. Do not claim a proven timeout root cause.

Bounded flat structural GSC trees now pass full Windows workspace gates (258
source files, eight crates). Seven new tests include 99 independent recovered-
grammar replays, all two-byte inputs, borrowed source spans, privacy, exact budgets,
and 12,000 nested groups without recursive tree teardown. The driver respects
default reductions, sticky EOF, and first-needed lexical/syntax failure ordering.
Script-loader plus selection is source-pinned; star entry also supports statement
fragments. These are structural trees, not semantic actions, name binding, a
compiler, or an executing VM. See ADR 0016. All nine original-function generators
and lexical/literal/structural replay regeneration pass; Linux and original-parser
trace/stock-mod conformance remain open.

Offline --inspect-gsc now passes full Windows workspace gates (260 source files,
eight crates). Four process tests cover JSON/human logs, counts, original error
locations, oversize/missing sources, mode/network exclusion, and source/path
redaction. It uses bounded snapshots and adds only an internal script dependency.
Structural inspection does not validate literal values, resolve includes, bind
names, compile, or execute GSC. Linux checks remain open.

Typed module syntax is implemented with six tests, 66,000-function/1,024-parameter
synthetic coverage, raw spelling/order/duplicates, original body views, budgets,
and privacy. Windows full debug/Clippy/rustdoc, focused script release, policy
(269 source files/eight crates), and source-profile regeneration pass. See ADR
0017. The full release probe subsequently timed out; local controlled repetitions
found missing third datagrams near blocking receive-timeout timing, with no MAC/
send failure. That I/O loop is now replaced with reviewed Mio 1.2.4 readiness
polling (ADR 0018). All Windows workspace gates pass again (270 source files/eight
crates), including 16 original exchanges per probe test and a queued burst test.
No further timeout increase, firewall changes, or alternate target directory was
used. Do not claim a proven OS defect, zero packet loss, or performance gains.

Legacy name/local function binding now passes all Windows workspace gates (288
source files/eight crates), with fifteen new tests and 125 original ASCII helper
cases. All ten original-function generators and lexical/literal/structural checks
pass. See ADR 0019. Borrowed name keys retain native long-name case fallback
without a capacity ceiling; short names fold under an explicit ASCII/C profile.
ModuleSymbols preserves every definition, indexes only mode-active definitions,
rejects active duplicates/invalid developer regions, and keeps original current/
previous offsets. Disabled developer functions do not participate in duplicate
checking. Source compiler's conditional registration flag is gScrVarPub+7,
developer_script; assembly/header are now hash-pinned with mechanical guards.
Directive placement follows definition/region checks; calls/imports/parameters/
locals/bodies remain unresolved. No compiler/VM or modern services execute.

Offline --inspect-gsc now binds local definitions with --gsc-developer enabled or
disabled (default disabled). It reports total/active/visibility counts and both
duplicate locations, never script names/paths. The flag is rejected outside this
mode. Linux and original-compiler/stock-mod acceptance remain open.

Scalar coercion, six numeric comparisons, and float-result division now pass full
Windows workspace gates (294 source files/eight crates) against 12,416 cases from
eight actual functions. See ADR 0020. All eleven original-function generators pass
after shared declaration extraction/post-return calling-convention support. The
explicit nearest/SSE profile preserves mixed conversion, a binary32 difference
compared against a binary64 1e-6 threshold, and <=/>= predicate negation for NaNs.
Int division yields float; signed-zero denominators return DivisionByZero without
partial caller mutation. Arithmetic NaN payloads are unspecified; FP control state,
VM error cleanup, strings/vectors/objects, and other operators remain separate.

Owned ordinary-string concatenation now passes full Windows workspace gates.
Four tests preserve 54 native helper byte results and intentionally expand 54
native scratch-capacity failures, including a 200,000-byte result. Checked length,
exact/one-less result budgets, fallible reserve, empty/self operands, raw bytes,
localized rejection, source privacy, and unchanged inputs pass. The result budget
does not account for aggregate runtime memory, allocator overhead, or input storage.
No interner, heap/GC choice, mixed coercion, VM integration, or wire expansion is
established. The actual C helper uses a stricter cutoff than archival assembly;
ADR 0021 records that distinction. All twelve original-function generators pass;
string/ref service stubs and capacity changes are explicit, not full native parity.

Next: source-grounded arithmetic and expression/body lowering. Integer signed
overflow, shifts, and remainder faults need explicit machine-profile/error policy
rather than undefined C operators or Rust build-mode-dependent overflow behavior.

Next: source-grounded semantic/bytecode/runtime foundations alongside broader
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
