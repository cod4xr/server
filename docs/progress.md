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

ADR 0081 implements checked managed host-array transport across VM cells and
direct/indirect calls, host result transfer, immediate dead-callee release and
prepaid cleanup including pending registration/return values. A private explicit
probe table replaces the initial HashSet prototype so complete bucket admission
precedes allocation and every probe is metered. Fifteen integration/two private
tests cover independent 512-program peak ownership, exact/shorter scopes, 70k
locals, 12k frames, 100k bounded overwrites beside 70k host aliases, hostile
handles/fields and paired pressure roots. Full Windows debug/release, formatting,
all-target denied-warning Clippy/rustdoc, policy (924 sources/eight crates/largest
287) and diff checks pass: 310 frontend/235 legacy/15 private script tests plus two
doctests. Native lifetime 210 O0/O2 replays and 61/77/38 lifetime/include/package
mutations plus syntax guards pass. Linux/hosted CI remains unrun. Source operations
and ordinary/nested logical lifetimes remain gated; no full-GSC or join gate closes.

ADR 0066 now compiles/executes spawnstruct through explicit object contexts and the
offline runner. Arguments evaluate before allocation and remain roots through one
pressure collection/retry. New native helper traces match at O0/O2 (32 cases,
modeled allocator/output with actual selected type/add/remove-ref helpers); the
constructor-role guard rejects 22 mutations. Six typed tests cover fresh identities,
65,536 extras/exact stack/fuel, 70,000 live constructors, caller-stack graph/string
roots, live/slot/metadata pressure and all insufficient work/scratch prefixes.
Three source tests cover query/case/shadow/error maps, temporary operand roots and
100,000 iterations under two slots/99,999 paired collections. Two process tests
check opaque stdout, numeric usage, private errors and readable logs. The demo
prints integer two from Size plus definedness. General heap dispatch and string/
object operations remain separate leaves. Full Windows debug/release, denied-warning
Clippy/rustdoc, formatting, policy (727 sources/eight crates/largest 287) and diff
checks pass; 99 structural replays regenerate. Current suites are 248 frontend/
171 legacy, six private units and two doctests. Native allocation/cleanup/ABI/VM,
fields/arrays, scheduling/services/full mods, Linux/hosted CI and joins remain open.
Next: source-grounded field reads/writes/deletion and key/evaluation semantics.
No complete GSC workstream or full server milestone is closed by this checkpoint.

ADR 0065 adds checked ordinary-struct definedness and Size=1, independent of stored
field count. A new actual C pointer-size oracle records 180 identical O0/O2 outcomes:
15 ordinary-struct rows replay through Rust, 165 other-subtype rows remain helper-
only. The parent-pool size guard now rejects 46 mutations. Existing definedness
coverage expands to 64 Rust replays (56 results/eight missing errors) out of 351
isolated archival/modeled-admission rows, leaving 287 helper-only. Four actual C
type predicates now replay 224/832 rows (32 results/192 arity errors), leaving 608
helper-only; outcome rows are unchanged. Typed Size executes with 70,000 fields
and exact three-step global work; one-less rejects without mutation. Compiled GSC
calls combine Size/IsDefined and evaluated extras. Full Windows debug/release,
denied-warning Clippy/rustdoc, formatting, policy (714 sources/eight crates/largest
287) and diff checks pass. Current suites are 245 frontend/165 legacy, six private
units and two doctests. Constructors/fields, general object operators, arrays,
scheduling/library/game services, native lifetimes/full mods, Linux/CI and joins
remain open. No complete GSC workstream is closed by these component checkpoints.

ADR 0063 now assesses explicit-root shared struct storage: opaque wide checked
identities, flat object/field arenas, owned exact binary keys, metered probes and
staged tombstone rebuilds, atomic iterative graph reclamation and composite stored/
external string roots. Fourteen new integration tests plus two private boundary
tests cover 70,000 objects/fields, 100,000 churn cycles, 12,000-object shared/cyclic
graphs, 1 MiB keys, exact/one-less retained/work/scratch limits, every insufficient
growth/collection work prefix, late invalid roots, namespace/generation/lifetime
checks and an independent 4,096-operation graph model. Immutable object constants
and pure/string-only invocation paths explicitly reject snapshots. This store
does not provide source constructors/fields, arrays or general object operators.

ADR 0064 now adds optional object-aware reference transport through standalone,
direct/indirect and compiled GSC calls. Complete live frame/caller/cache/external
object/string roots share global work and simultaneous scratch admission. Object
then composite string collection have separate success counts; a completed object
collection can persist after a later failure. Seven typed/one source tests cover
12,000 frames, 1,000 compiled loop/call allocations, caller-stack-only objects,
cached-only strings, external/nested graph roots, every insufficient work prefix,
exact/one-less scratch, stale/wrong-bound stores, late stale fields, returned roots
and context reuse. Current suites are 244 frontend/163 legacy tests, six private
units and two doctests. Full Windows workspace debug/release, denied-warning Clippy/
rustdoc, formatting, policy (709 sources/eight crates/largest 287) and diff checks
pass. Execution heap coordination now lives in execution/heap with separate
accounting, validation, roots and collection leaves. No new dependency or target
directory was introduced. Native refcount/GC/operator, Linux/hosted CI, full mods
and joins remain open.

The runnable object_heap_workload performs 100,000 updates under 64-object/string
budgets and reports 1,613 paired collection rounds, 8,825,031 logical work steps,
4,096 peak scratch bytes, 576 peak key bytes, 1,012 peak string bytes and 21,504
peak logical retained metadata bytes. Final storage is one object/field, five key
bytes and four string bytes, with 64 object slots/128 field slots/256 index buckets/
64 string slots. Local debug/release elapsed observations were 683,795/48,573 us,
not native speed/RSS claims. Full Windows workspace debug/release, formatting,
denied-warning Clippy/rustdoc, policy (699 sources/eight crates/largest 287) and
diff checks pass. Native array 306/69/141, string-value 3,024 and isolated defined
351 checks still match. Linux/hosted CI, full GSC/stock mods and joins remain open.
Next: source-grounded struct constructors/field semantics and object operators,
followed by the distinct legacy array copy/ownership strategy and broader services.
No complete GSC workstream is closed; the approximate 10–12-workstream estimate
has not decreased based on this component's tests or commit.

The owned string-store component now has 11 integration tests plus private
namespace/generation boundary tests. It admits 70,000 live values and sustains
100,000 temporary values under 64 slots. One local release workload observed
1,587 collections, 2,008,131 logical work units, 1,012 peak payload bytes and
four final payload bytes with 2,048 retained metadata bytes. These are component
observations, not full VM/RSS/server benchmarks. ADR 0054 records exact evidence
and its component scope. ADR 0055 now integrates typed heap-backed execution and
supplied source arguments: 15 kernel tests plus two source tests cover reclamation
across 12,000 frames, 100,000 loop allocations in four slots, global work/scratch,
errors/lifetimes, native plain-concat results/expansion and hostile inventories.
Source literal lowering and CLI output follow in ADR 0057; no GSC workstream is closed.
ADR 0056 now adds owned typed literal tables/cache with 13 tests, including
70,000 rows, 1,001 evaluations of one 1 MiB value without repeated payload copying,
cached-only roots, independent heaps and exact complete proof/runtime resources.
ADR 0057 now implements bound source plain literals and offline escaped output.
Nine source tests cover 70,000 rows, 100,000-byte values, calls/loops/ownership,
exact compiler/proof resources and maps. Three real process tests cover byte output,
JSON/human privacy and failures. The normal target strings_demo.gsc command prints
string b"[[[Rust]]]". Translation/mixed coercion, string indexing/switches, objects/
scheduling/library/game services and full mod acceptance remain open.
ADR 0058 adds plain-only isstring with 832 actual native predicate outcomes,
168 Rust replays (24 results/144 arity failures), 664 helper-only cases and 46
source-role mutations. Five source tests cover categories/shadowing/maps/order/
skipped calls and 256 string arguments; typed fixtures reach 70,000 handles.
ADR 0059 now adds localized values and raw escaped output. The native value oracle
replays all 3,024 comparisons/arithmetic outcomes; predicate coverage expands to
196 Rust replays (28 results/168 arity errors), leaving 636 helper-only rows.
Six kernel/six source/two process tests cover lifetimes, mixed tags, pressure,
70,000 mixed rows and privacy. A separate 1 MiB alias test proves constant logical
comparison work after handle validation; distinct handles still pay content work.

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
or complete GSC VM.

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

Scalar add/subtract/multiply and integer OR/AND/XOR, signed remainder, and masked
shifts now pass full Windows workspace gates. Five tests include 10,701 defined
original-helper cases and 818 explicitly tagged isolated instruction cases. The
archival assembly hash/instruction blocks are checked; this is not execution of
full native routines. GCC O0/O2 agree. Integer results wrap consistently, signed
shift counts use five low bits, noninteger inputs fail before remainder checks,
and MIN%-1 becomes RemainderOverflow instead of a host fault. See ADR 0022. All
thirteen original-function/profile generators pass after adding the source hash.
No compiler, full value system, VM, or Linux acceptance follows from this kernel.

Typed borrowed expression-operator views now pass full Windows workspace gates.
Eight tests cover all sixteen eager operators, two logical branches, two prefixes,
source order/spans, precedence/associativity, primary opacity, private Debug/errors,
independent trees, category failures, and 12,000 nested operators viewed iteratively.
Classification performs bounded local reads without allocation, recursion, or a
descendant walk. It does not decode literals, bind names, select overloaded values,
or evaluate expressions. Source profiles guard action roles/opcode identity without
native action execution; lexical/literal/structural regeneration passes. See ADR
0023. Compiler/VM/stock-mod and Linux acceptance remain open.

Typed borrowed primary views now pass full Windows workspace gates. Seven tests
cover all 22 primitive productions, raw literal polarity/spelling (including still-
unconverted overflow spellings), parenthesized lists, unresolved call/reference/
function subtrees, special roots, size/empty-array/animation/#animtree/breakon forms,
category failures, unchanged source, and private diagnostics. Existing operator
tests also pass after sharing checked child access. Primary action profiles now
guard every local role; no native semantic actions execute. This does not convert
values, resolve descendants, choose arity/evaluation order, or enable runtime
animation/debugging. See ADR 0024. Linux/compiler/VM/stock-mod gates remain open.

Budgeted source-ordered expression-list indices now pass full Windows workspace
gates. Six tests cover nested expression retention, forward/reverse order, original
spans/borrows, empty/single/prefix lists, every exact/one-less quota, zero/preflight
work, unrelated-node failures, privacy, independent trees, and 70,000 entries.
Both passes and complete logical storage are admitted before fallible allocation.
Native actions prepend nodes; source ordering is not runtime evaluation/push order.
No arity, executable parameter capacity, binding, or evaluation is established.
See ADR 0025. Source list-role/structural regeneration passes; Linux gates stay open.

Script integration tests now use two small module-only Cargo roots instead of
thirty leaf binaries. Pre/post Cargo inventories retain all 81 names with identical
multiplicities (63 frontend, 18 legacy), and full debug/release suites pass. Policy
guards every plain #[test] file's unconditional registration, including subfolders;
disposable checks reject missing/duplicate/extra/gated/nested-unregistered leaves.
Leaf/fixture files stay in place, helper modules are shared, and all builds reuse
the normal target. No old artifacts are deleted or speed/RSS improvement claimed.
See ADR 0026. Focused Cargo commands now select frontend/legacy plus module filters.

Borrowed invocation/function-name views now pass full Windows workspace gates.
Seven tests cover named/external/indirect targets, function/method receivers,
immediate/thread spelling, original case/module/name/spans, direct/primary wrappers,
empty/nested separately budgeted arguments, failures/privacy, independent trees,
and a borrowed 100,000-byte callee name. Full suites retain 70 frontend and 18 legacy
tests. Source profiles guard rules 28..39 without executing native actions. These
views do not resolve modules, link functions, evaluate operands, grant host/builtin
authority, or schedule threads. See ADR 0027. Linux/compiler/VM/stock-mod gates stay
open. The existing branch was renamed externally to main; history remained intact.

Borrowed reference views now pass full Windows workspace gates. Five tests cover
all five local roles, names/dollar prefixes, direct/primary wrappers, exact spans,
field/index/call composition, reserved .self versus ordinary .Self, private errors,
independent sources, and an iterative 12,000-index chain. All 75 frontend and 18
legacy tests pass. Source profiles guard rules 62..66 without native action or
reference execution. These views do not resolve variables/objects, decide scopes/
mutability, or grant dollar/self runtime authority. See ADR 0028. Full grammar
recovery was not completed; only the targeted .self role was confirmed. Linux/
compiler/VM/stock-mod acceptance remain open.

The owned verified scalar execution kernel now passes full Windows workspace gates.
Thirteen tests exercise actual arithmetic/comparison/division/unary/branch/stack/
return execution, ownership/privacy, malformed targets/returns/fallthrough/joins,
exact/one-less budgets, 70,000-index jumps, 4,096-value stacks, balanced loop fuel,
and 4,096 deterministic hostile streams. Execution routes 22,383 existing fixtures:
11,519 arithmetic/profile cases and 10,864 comparison/division cases. Coercion-only
P records are not counted as executed opcode cases. Failed work/peaks remain
charged; no result or external mutation escapes failures. The suites contain 75
frontend and 31 legacy tests. See ADR 0029. This typed numeric/undefined profile
has no source compiler, full value/object model, calls/frames, scheduling, host I/O,
native/serialized ABI, stock-mod acceptance, or Linux acceptance. It owns no game
state and grants no runtime services. Normal target reused; no new dependencies.

Restricted source-to-kernel compilation now passes full Windows workspace gates.
It accepts zero-parameter empty/single-scalar-return functions and owns verified
code/original source maps. Twelve integration tests cover all numeric operators,
lazy branches, checked signed domains/negative float zero, source-independent
ownership/private diagnostics, compiler/literal/kernel admission, failed work,
12,000 groups/operators, and branch destinations beyond 16 bits. Unsupported
parameters/bodies/values, including unreachable calls, return typed errors. No
partial function is published before proof and bounded pure validation complete.
The suites contain 87 frontend and 31 legacy tests; source policy checks 377 files
in eight crates (largest 287 lines). Body/action profiles regenerate alongside 99
structural replays and reject 18 in-memory production/action mutations. These
checks do not execute the native compiler. Signed MIN admission is an explicit
checked domain policy, not undefined native atoi-overflow parity; see ADR 0030.
Normal target reused, with no new dependency or alternate cache. Linux, complete
source/compiler/VM/stock-mod acceptance and gameplay remain open.

Typed statements and admitted ordered body lists now pass full Windows workspace
gates. Fourteen tests cover all 37 direct statement productions, bounded wrappers,
original/direct operation spans, ten compound operators, distinct event payloads,
opaque for options, nested bodies/labels, privacy/independent trees, 12,000 nested
conditionals, 70,000 immediate statements, and complete count/two-pass work/storage
admission. Failed work/counts remain visible without partial indices. The restricted
scalar compiler reuses return classification with unchanged source maps/profile.
Statement/body/compound source profiles and 99 structural replays regenerate; the
new guard rejects 131 in-memory shape/action mutations. See ADR 0031. Suites now
contain 101 frontend and 31 legacy tests; policy checks 395 source files/eight
crates, largest 287 lines. Normal target reused with no new dependency/cache.
These are typed syntax/body indices, not local binding, lvalue/control-flow scope,
developer activation, debugger/event runtime, general-body execution or scheduling.
Linux and full compiler/VM/stock-mod acceptance remain open.

Scalar local storage/arguments now pass full Windows workspace gates. Immutable
layouts admit parameter/total-local counts; every local operand is checked even
in unreachable code. Fresh private invocation slots distinguish never-initialized
from initialized Undefined, and argument/other invocation state stays unchanged.
Local count/checked bytes, operand-stack storage and instruction fuel remain
separate scopes. Successful results expose logical local use; failures retain
actual work/stack peaks/admitted local counts. Ten added tests cover ownership,
arity/header/operand/stack/admission errors, selected-path initialization, exact
float data, numeric failures, huge checked overflow/zero fuel, 70,000 internal
arguments/slots, and 4,096 hostile local streams. Existing scalar/source tests pass.
Suites now contain 101 frontend and 41 legacy tests; policy checks 401 source
files/eight crates, largest 287 lines. See ADR 0032. Normal target reused without
dependencies or alternate caches. Exact internal arity is not native GSC policy;
source local binding/definite assignment/general-body execution, call frames,
scheduler, host services, stock-mod acceptance and Linux remain open.

The separate straight-line scalar-local source profile now passes full Windows
workspace gates. Parameters, plain-name assignments/reassignments, empty statements
and final/fallthrough returns compile into owned verified code/maps. Initialized
reads use legacy name identity; first self-reads/late/missing names fail, including
in skipped logical source. New targets initialize after RHS/store, while assigned
locals and mutable parameters retain prior initialization. Hash iteration never
selects slot or diagnostic order. Binding counts/identifier visits/logical map-pair
payload share compiler admission; body indices/kernel/table overhead remain distinct.
The original pure-return entry point retains accepted syntax/validation. Results
now tag their source profile; validation steps are Some for pure-return validation
and None for local compilation, which never executes fabricated arguments.
Eleven added tests cover source invocation, runtime source maps, privacy/ownership,
initialization/rejection policies, exact/one-less compiler/body/kernel limits,
70,000 source locals, 100,000-byte names and 12,000 groups. All 112 frontend/41
legacy tests and doctests pass; policy checks 411 source files/eight crates,
largest 287 lines. Grammar/statement/source guards and 99 replays regenerate.
See ADR 0033. Normal target reused with no dependency/cache additions. This is
not full native binding/arity/duplicate-parameter/diagnostic-order equivalence,
control-flow compilation, whole-module/stock-mod execution, or Linux acceptance.

Scalar-local compound assignments now pass full Windows workspace gates. Three
added tests execute all ten operators through existing numeric instructions,
including wrap/shift/float-result division, mixed floats, target/RHS alias values,
parameter mutation isolation, uninitialized/non-name rejections, located runtime
type/numeric failures, and complete exact compiler/name admission. Checked
initialized-name lookup is shared with expression reads; stores reuse existing
assignment tasks and budgets. Source maps retain the compound token. Existing
source/kernel tests pass: 115 frontend and 41 legacy tests plus doctests; policy
checks 413 source files/eight crates, largest 287 lines. Grammar/compound profiles,
99 structural replays and 11,519 arithmetic reference cases regenerate. See ADR
0034. No new dependency/cache or target directory. Full object/reference/RHS
side effects, postfix integer updates, control-flow binding, native arity, full
GSC/stock-mod acceptance and Linux remain open. The twelve-workstream acceptance
breakdown is now recorded in docs/gsc-runtime.md.

Integer-only postfix updates now pass full Windows workspace gates. The legacy
value operation reuses established wrapping arithmetic and rejects every float;
a unary typed instruction and initialized plain-name source statements share
existing stack/fuel/name/store/source-map machinery. Five added tests cover word
boundaries/float classes, kernel proof/fuel/type errors, source locals/parameters,
target admission, update-token maps and exact quotas. Both 4,096-stream hostile
corpora now include updates. All 118 frontend/43 legacy tests plus doctests pass;
policy checks 417 source files/eight crates, largest 287 lines. Source guards now
tie grammar tags/compiler opcodes/header types to native VM integer checks/word
operations and reject 15 in-memory compiler/VM/header mutations. All thirteen
original-helper/instruction generators regenerate, along with 724 lexical, 545
literal and 99 structural replays. See ADR 0035. Normal target reused; no new
dependency/cache. No native interpreter execution, whole reference/unwind/VM
equivalence, postfix expressions, object/index updates, modern numeric rules,
control flow, full GSC/stock-mod acceptance or Linux success is claimed.

The explicit scalar legacy script-argument path now passes full Windows workspace
gates. It accepts a supplied prefix in source parameter order, initializes only
missing parameters to Undefined, rejects extras and leaves other locals
uninitialized. Direct single-vector filling uses existing local/stack/fuel
admission, without padded argument temporaries. Exact/empty internal entry points
retain their own contracts. Seven added tests cover every supplied prefix,
source functions/error maps, default versus uninitialized slots, caller/float-bit
isolation, 70,000 missing parameters, quota boundaries and huge checked failures.
The 4,096-stream local hostile corpus now invokes both exact and legacy binding.
All 120 frontend/48 legacy tests plus doctests pass; policy checks 421 source
files/eight crates, largest 287 lines. Selected compiler/VM/header/factory argument
guards and 99 structural replays regenerate; the new guard rejects 17 in-memory
mutations. See ADR 0036. Normal target reused, no dependency/cache additions.
Builtin signatures, native evaluation/push order, receivers/frames, stack markers,
full cleanup/unwinding/GSC/mod conformance and Linux remain open.

Scalar-local ordinary blocks and if/if-else now pass full Windows workspace gates.
Flat lowering emits checked forward conditional/unconditional relocations and
retains original condition/operation spans. Stable wide slots carry separate
initialization snapshots: restore incoming facts before else and intersect only
paths reaching the join. Both source branches remain checked, including constant
conditions. Immediate nonfinal returns and later structurally unreachable source
are rejected under this explicit restricted profile. No fabricated arguments run.
Snapshot flag payload shares compiler metadata plus a separate live-byte quota;
initial fill/copy/restore/merge visits share global work. Eleven new tests cover
path results, initialization/return failures, maps, exact/one-less admission,
70,000 local snapshots, forward distances beyond 65,535 instructions and 12,000
nested branches with successful/failed flat cleanup. All 131 frontend/48 legacy
tests and doctests pass; warnings-denied Clippy/rustdoc, formatting and repository
policy pass. Largest source remains 287 lines. Existing pinned syntax profiles
and 99 structural replays regenerate. See ADR 0037. The normal target is reused;
no dependencies or alternate caches added. Native compiler execution/full scope,
constant folding/diagnostic order, loops/switch, full GSC/mod acceptance and Linux
validation remain open.

Scalar while compilation now passes full Windows workspace gates. Conditions
reevaluate through consuming numeric branches and verified absolute backward
jumps; execution fuel bounds endless loops. Incoming initialization snapshots
restore at the exit for zero-iteration safety; body-only writes never authorize
later reads. Existing flat lowering, name/task/code/snapshot quotas and source
maps remain shared. Nine added tests cover repeated/nested/lazy loops, returns,
condition/operator locations, illegal initialization/unsupported source, exact
compiler/fuel admission, 70,000 local snapshots with long backward jumps and
12,000 nested loops with bounded failed cleanup. The combined wide source
explicitly raises only its test frontend node admission to two million; default
production budgets remain unchanged. All 140 frontend/48 legacy tests and doctests
pass, alongside formatting, warnings-denied Clippy/rustdoc and repository policy;
largest source remains 287 lines. Selected pinned while compiler/header/VM guards
and 99 structural replays regenerate, and nineteen in-memory source mutations
are rejected. See ADR 0038. No dependencies or alternate target directories added.
Native routines/watchdog timing, scopes/constant folding/diagnostic order,
for/break/continue/switch, richer values, full GSC/mod acceptance and Linux remain open.

Scalar while break/continue now pass full Windows workspace gates. The nearest
active loop context saves/restores outer ownership in already admitted end tasks.
Transfers terminate structural paths, preserve conditional initialization joins,
and retain original keyword maps/fuel. Illegal outside-loop contexts and later
unreachable source fail explicitly. Break fixups link within complete admitted
pending instruction records, bind once with monotonic/forward checks and charge
every visit globally; no second fixup allocation/native fixed array is introduced.
Emission/admission and relocation binding/resolution now occupy separate leaf
modules. Six added tests cover nested targeting, returning/continuing joins,
privacy/maps/fuel/caller isolation, exact/one-less quotas, 20,000 linked fixups
with wide exits, and 12,000 nested contexts carrying pending outer breaks through
successful execution and bounded failure cleanup. All 146 frontend/48 legacy
tests and doctests pass; formatting, warnings-denied Clippy/rustdoc and repository
policy pass (441 sources/eight crates, largest 287 lines). Selected pinned
compiler context/legality/fixup/restore and VM jump profiles plus 99 structural
replays regenerate; 38 in-memory source mutations are rejected. See ADR 0039.
No dependencies or alternate target directories added. Native routines/block
cleanup/scope/watchdog/error order, for/switch, full GSC/mod acceptance and Linux
validation remain open.

Scalar for now passes full Windows workspace gates. Flat phases run initializer
once, reevaluate condition, execute body/update, and jump back. Optional clauses
are explicit; initializer/update accept scalar assignment/compound/postfix/empty
syntax. Deferred continue fixups reach update, with admitted initialization facts
intersecting continuing body/fallthrough paths and excluding break/return. An
unreachable update checks incoming facts, and exit facts conservatively allow
zero iterations. Shared generalized contexts/transfer records introduce no extra
fixup array; all snapshots/task/code work and metadata remain admitted. Nine new
tests cover all eight optional combinations, mixed loops/transfers, returns and
update joins, explicit rejection policy, clause maps/fuel/caller isolation, exact
compiler limits, 20,000 wide deferred continues, and 12,000 nested for contexts
with bounded failed cleanup. All 155 frontend/48 legacy tests and doctests pass;
formatting, warnings-denied Clippy/rustdoc and repository policy pass (450 sources,
eight crates, largest 287 lines). Selected optional grammar/compiler phase guards
and 99 structural replays regenerate; 35 in-memory mutations are rejected. See
ADR 0040. No new dependencies/alternate target directories. Native compiler/VM,
scope/constant folding/watchdog/error order, switch, richer values, full GSC/mod
acceptance and Linux remain open.

Owned legacy vector helper values now pass full Windows workspace gates. Fixed
inline triples, numeric/vector operands, explicit weaker pairs, operators and
typed errors preserve float bits, int conversion/broadcast, component arithmetic,
exact equality and addition's separate mixed-type rejection. Any signed-zero
denominator fails atomically. Debug/errors redact coordinates; no vector pool,
reference ID or runtime allocation is introduced. Three tests cover all 19,852
recorded original-helper outcomes plus boundary/coercion/equality/privacy cases.
The corpus has 756 constructions, 1,736 weaker pairs, 6,944 arithmetic outcomes
and 10,416 comparisons: 11,580 successful outcomes and 8,272 intercepted type/
division error outcomes. Fourteen original C functions and original math macros
run at O0/O2 under nearest/SSE, FTZ/DAZ-off, with bounded pool/ref/string/error
stubs. Construction/coercion and finite math bits match; arithmetic NaNs match by
class. This is not native error/ref/cleanup execution. q_math.c was compared to
the existing pinned Git blob before adding SHA-256
5601c38a82b0f7200beaffebf79a26a13ade36759cffd3c4412c619479027c74.
All seventeen helper/source-profile generators pass after that lock addition;
existing fixtures remain unchanged. All 155 frontend/51 legacy tests and doctests
pass, alongside full formatting/warnings-denied Clippy/rustdoc and repository
policy (462 source files/eight crates, largest 287 lines). See ADR 0041. No new
dependencies/alternate target directories. Kernel/source vector support, native
component evaluation/push order and constructor cleanup, full heap/VM/mod acceptance
and Linux remain open.

Primitive-kernel vector execution now passes full Windows workspace gates. Inline
values and MakeVector retain immutable proof, copied locals/arguments/duplicate/
return, global fuel and actual size_of-based code/stack/Option-local admission.
MakeVector consumes bottom-to-top z/y/x and produces x/y/z coordinates, matching
the selected native constructor's reversed slots without defining a serialized
opcode ABI. Focused numeric-value dispatch preserves scalar behavior and vector
operator/coercion/equality rules; unsupported component/truth/integer types and
fixed nested vector errors disclose no data. Five new kernel tests replay 18,116
original construction/arithmetic/comparison outcomes, retain 1,736 helper-only
coercion rows as separate evidence, and cover underflow, exact/one-less byte/fuel
limits, default arguments, data isolation and 70,000 vector argument slots. Both
4,096-stream hostile corpora now include vector constants/MakeVector; local streams
also execute vector arguments. Two source tests pass vector parameters through
existing restricted syntax with caller isolation and original operation maps.
Three-element source groups still fail explicitly. All 157 frontend/56 legacy
tests and doctests pass; formatting, warnings-denied Clippy/rustdoc and repository
policy pass (469 sources/eight crates, largest 287 lines). Original vector O0/O2
fixtures regenerate unchanged. See ADR 0042. No dependencies or alternate targets.
Source constructors/evaluation order, native VM error indices/ref/cleanup, heap/
calls/scheduling, full GSC/mod acceptance and Linux validation remain open.

Function compiler naming now passes full Windows workspace gates. Owned results,
budgets/usage/resources/errors and profiles use CompiledFunction and Function*
names. compile_scalar_return_function selects PureScalarReturn;
compile_primitive_function selects PrimitiveStatements. Leaf/private types and
function_compile_support imports match that terminology. The unreleased API has
no compatibility wrappers/duplicate implementation or additional profile; accepted
syntax/runtime behavior, quotas/accounting, proofs, diagnostic categories/spans/
privacy and source maps remain covered by existing suites. Diagnostic labels
follow the new names. No new rename-only tests were added. All 157 frontend/56
legacy tests and doctests pass, along with formatting, warnings-denied Clippy/
rustdoc and repository policy (469 sources/eight crates, largest 287 lines).
Stale Rust compiler/helper symbols were checked. See ADR 0043. No dependency,
alternate target, wire/content/bytecode format or full-GSC claim is introduced.
Source constructors, native/full-GSC/mod conformance and Linux remain open.

Primitive source vector construction now passes full Windows workspace gates.
The builder owns one explicit source profile for permission, validation and
published metadata. PureScalarReturn retains singleton/scalar syntax;
PrimitiveStatements accepts triples through fixed charged child probes and flat
tasks, evaluating z/y/x into MakeVector's reversed slots. Whole group and operator
maps remain available, with runtime type checks and no fabricated-argument/native
constant-folding execution. Eight added tests cover actual vector source values,
locals/operators/loops, float boundaries, rightmost component failure and name
order, maps/privacy/profile/shape rejection, exact and separate nested quotas,
70,000 vector locals and 12,000-deep vector expressions with flat failed cleanup.
The wide fixture explicitly admits 3M nodes/4M edges/512MiB frontend metadata;
production defaults remain unchanged. All 165 frontend/56 legacy tests and
doctests pass, with formatting, warnings-denied Clippy/rustdoc and repository
policy (475 sources/eight crates, largest 287 lines). Parse-tree helper bytes
matched the pinned Git blob before adding SHA-256
efca4fa2b3fc5eabf5f8f5bea644425da1610f1a40293c16f126b252312feacf.
All seventeen pinned helper/source-profile checks pass; existing fixture bytes
remain unchanged. Selected prepend/grammar/compiler/VM/helper guards now reject
33 in-memory grammar/tree/compiler mutations, including combined prepend call
argument roles. See ADR 0044. No dependencies or alternate targets added. Native
parser/compiler/VM execution, full side-effect/constant-folding/error-index/ref
cleanup, richer values/indexing/calls/scheduling, mod conformance and Linux remain open.

Vector index reads now compile and execute under the primitive source profile.
The actual pinned Scr_EvalArray vector branch supplies 299 O0/O2 outcomes: 39
exact-bit successes and 260 type/range errors. All replay through verified kernel
instructions; the numeric-only helper replays 286 rows and excludes 13 vector
indices its API cannot express. Selected parser/compiler/VM/helper source guards
reject 27 mutations; bounded object/string/ref/error harness services do not prove
native cleanup or VM execution. Source tests retain index-before-object error
order, redacted bracket maps, immutable reconstruction, 70,000 locals, 12,000-deep
reads and exact/one-less resource admission. Hostile streams include Index.
All 170 frontend/59 legacy tests and doctests pass in workspace debug and release
on Windows, alongside formatting, warnings-denied Clippy/rustdoc and repository
policy (486 source files/eight crates, largest 287 lines). Original vector fixtures
remain unchanged; vector, index and syntax generators pass. See ADR 0045. No new
dependencies or build targets. General indexable heap values, calls, native/full
GSC, mods and Linux acceptance remain open.

Undefined comparison dispatch now has its own focused leaf, preserving existing
numeric/vector rules while admitting two-Undefined equality and rejecting every
mixed/ordered Undefined pair. The actual pinned coercion/comparison helpers yield
402 O0/O2 outcomes (two successes/400 errors), all replayed through verified code.
Source tests cover both profiles, lazy evaluation, missing/stored Undefined,
uninitialized rejection, operator maps and privacy. Existing vector/index fixture
bytes and syntax profiles remain unchanged. All 173 frontend/60 legacy tests and
doctests pass in workspace Windows debug/release, with formatting, warnings-denied
Clippy/rustdoc and repository policy (491 sources/eight crates, largest 287 lines).
See ADR 0046; no dependency/target/storage changes. Full value/native VM/mod/Linux
acceptance remains open.

Primitive direct calls now use a complete immutable VerifiedKernelModule proof
and iterative private runtime frames. All target/arity operands, including
unreachable calls, and each function's stack/flow must verify before publication.
Inventory/code/work/overlapping retained table/code/verifier payload share global
admission. Runtime shares frame stepping with standalone execution, global fuel,
separate conservative initialization/copy/reservation work, and aggregate frame/
reserved-operand/local count/byte budgets. Missing parameter suffixes become
initialized Undefined; nonparameter locals stay uninitialized. Returns release
frame admission; failures drop private state with function/PC/global work/counts
and no private values. Static arity rejection and the argument stack convention
are this typed profile, not native source diagnostic/evaluation/push-order parity.
Fourteen added tests cover forward/mutual/recursive calls, defaults/order/caller
operands/isolation, located error recovery, malformed later/unreachable operands,
exact/one-less aggregate proof/runtime limits, shallow loops, 12,001 flat frames,
66,000 functions, 70,000-argument calls/missing suffixes and 4,096 hostile inventories.
All 173 frontend/74 legacy tests and doctests pass in workspace Windows debug/
release with formatting, warnings-denied Clippy/rustdoc and repository policy
(510 sources/eight crates, largest 287 lines). See ADR 0047. No dependencies or
alternate targets. Source calls/linking, methods/indirect calls, heap/scheduling,
native/full GSC/mod compatibility and Linux acceptance remain open. No full legacy
workstream gate is closed by this kernel implementation.

Excess direct-call arguments now fail at invocation after caller argument
instructions and before callee allocation/copies/body. Proof still checks every
target and reachable stack effect, while skipped/unreachable excess counts do
not fail. Checks use parameter count rather than total local slots. Three added
tests cover dynamic branches, caller/body error precedence, privacy/recovery and
huge reachable-count underflow; hostile inventories now admit dynamic arity errors.
Stronger selected argument source guards tie fallback caller traversal/emission
and parameter check-before-body roles. The executable fault driver rejects 38
in-memory role changes, including reorderings retaining all original needles;
99 syntax fixture replays remain unchanged. All 173 frontend/77 legacy tests and
doctests pass in workspace Windows debug/release, with formatting, warnings-denied
Clippy/rustdoc and repository policy (511 sources/eight crates, largest 287 lines).
See ADR 0048, superseding ADR 0047's static excess rejection. No dependencies or
alternate targets. Native parser/compiler/VM execution, parameter safe-set/ref/
error-index/cleanup parity, full GSC/mod and Linux gates remain open.

Bound primitive source modules now compile/execute local named forward, recursive
and mutual calls through one shared target scope. Arguments lower rightmost-first
via the existing expression cursor and admitted flat tasks, without temporary
argument arrays. One private mode owns standalone proof/validation or bound caller
visibility/context; shared FunctionLoweringBudget avoids irrelevant module settings.
Complete staged code/layout/maps verify before publication; declaration metadata
retains developer-disabled definitions and maps original rows to active entries.
Source/tree/symbol/name borrows do not escape. Ordinary-to-developer references
fail separately from inclusion, while inactive unsupported bodies remain metadata.
Empty/all-disabled modules contain no fabricated code; empty individual instruction
streams remain invalid. Global declaration/function/work/instruction/logical
metadata budgets include scope/table/staging/current lowering overlap, with distinct
per-function and body/literal/kernel scopes. Fifteen source tests pass real calls,
loops/branches/vector results, defaults/arity/error/name order, privacy/maps/source
drop, developer modes, unsupported forms, exact/one-less scopes, 70,000 arguments/
parameters, 66,000 linked functions and 12,000 nested call expressions/failed cleanup.
Selected resolved/deferred visibility guards reject 11 mutations; argument timing
guards reject 38; 99 structural fixture replays remain unchanged. Full workspace
Windows debug/release tests pass (188 frontend/77 legacy plus doctests), with
formatting, warnings-denied Clippy/rustdoc and repository policy (537 sources/eight
crates, largest 287 lines). See ADR 0049, including empty-inventory supersession.
No dependencies or alternate targets. Native compiler/VM/linker/error/ref cleanup,
imports/animation resolution, heap/receivers/builtins/scheduling/host/mod
and Linux gates remain open. No full legacy acceptance workstream is closed.

Local function references and immediate indirect source calls now compile/execute
under ADR 0050. Canonical local/developer resolution is shared with named calls;
rightmost arguments execute before the indirect target expression. References copy
through locals/parameters/returns, use redacted opaque immutable-module identities
and wide indices, and cannot invoke in foreign/recreated owners. Identity minting
occurs after complete proof, never wraps/reuses, and preserves failed proof work.
All static targets and indirect count-plus-target arithmetic are verified even in
unreachable code. Invocation shares private iterative frames and global arity/fuel/
setup/live-payload admission; actual sizeof-based budgets reflect representation.
Eight added frontend tests cover factories/higher-order recursion, evaluation/type/
arity/error order, developer restrictions/quotas and isolated-profile rejection.
Seven added legacy tests plus the private identity allocator test cover copies,
foreign/drop/redaction/parallel uniqueness/full-domain exhaustion, malformed proof,
66,000 targets/70,000 arguments and 2,664 actual C comparisons (72 successes/2,592
errors). Existing exact/one-less tests now cover direct and indirect calls; deep
source tests cover both paths at 12,000 nesting, and hostile inventories include
indirect/ref/type failures. Synthetic C identity labels map bijectively to valid
Rust references; these are not native code pointers/allocation/invocation. Selected
compiler/header/VM guards reject 31 mutations, alongside argument 38/visibility 11
and unchanged 99 structural replays. Related vector/index/Undefined oracles match.
Windows workspace debug/release tests pass (196 frontend/84 legacy plus doctests).
Formatting, warnings-denied workspace Clippy/rustdoc, repository policy (550 source
files/eight crates, largest 287 lines) and diff checks also pass.
No dependency or alternate target directory was added. Full legacy acceptance
workstreams, native compiler/VM/self/ref/unwind parity, imports/reload, heap/closures,
receivers/builtins/scheduling/game services/mod and Linux gates remain open.

Three pure builtins now compile/execute directly from bound source under ADR 0051:
isint/isfloat/isvector. Active script names shadow this sealed subset, canonical
case/developer rules stay enforced and disabled developer declarations do not
shadow. Unknown/external/receiver/thread/builtin-reference authority stays rejected.
All supplied arguments execute before exact-one runtime arity checking at the caller;
skipped bad counts remain valid. Proof still requires reachable operands. Pure
instructions reuse fuel/stack/count/actual-byte admission without argument arrays,
extra frames, per-call resets or the native compiler's 255-argument ceiling.
Two new reference files match pinned Git blobs and canonical LF hashes. Original
C helpers produce 624 O0/O2 count/tag outcomes: 105 bounded representable cases
execute in Rust (15 successes/90 arity errors), and 519 stay explicitly helper-only
for unavailable enum tags and UINT32_MAX declared counts. These do not allocate
all native values or execute the native compiler/VM/error unwinder. Selected C
signature/registration/type and compiler lookup/developer/argument roles reject
36 mutations. Existing function-reference/argument/visibility guards and 99 syntax
replays remain valid. Six new frontend tests cover real source predicates, shadowing,
case/developer/authority, arity/error order, privacy/maps/quotas and isolated profiles.
Three new legacy tests replay helpers and cover exact/one-less proof/execution
resources, huge unreachable operands and recovery. Existing source depth now
includes 12,000 nested pure calls with one frame; hostile streams include builtins.
Tests reach 256 source/70,000 typed arguments. Full Windows workspace debug/release
tests pass (202 frontend/87 legacy plus doctests), along with formatting, denied-
warning Clippy/rustdoc and repository policy (558 source files/eight crates,
largest 287 lines). No dependencies or alternate targets. General builtins, heap/
entity lifetime/isdefined, scheduling/game services, full mod/diagnostic/native
cleanup and Linux acceptance remain open. No full legacy workstream is closed.

Primitive integer switches now compile/execute under ADR 0052 in isolated primitive
functions and bound modules. Key evaluation occurs once; RequireInteger preserves
the word and rejects other runtime categories without float coercion. Integer-label
syntax remains distinct from booleans that decode to integer values. Full int32
keys deliberately expand the native encoded array-index domain; wide case inventories
avoid the native serialized short. These are explicit capacity expansions, not native
invalid-input parity. Default placement, adjacent labels, source-order fallthrough,
scoped breaks/continues and actual initialization exits work through flat owned
contexts. Continue keeps nearest-loop/for-update facts; returning/continuing paths
do not establish after-switch initialization. Case records/facts share admitted
control/metadata quotas and must release before scope publication. Sorting/search/
emission/fixup/fact work is charged. Balanced out-of-line typed dispatch uses checked
fixed footprints/shrinking ranges and existing proof/fuel/stack budgets.
Twelve added frontend tests cover defaults/misses/fallthrough/adjacency, full-word
expansion, every break/continue nesting direction, actual joins/unmatched/excluded
paths, for-update initialization/global fuel, source-key/call evaluation, strict
label/runtime types, duplicates/misplaced/prefix/unreachable forms, maps/privacy,
isolated profiles and all exact-one-less compiler quotas. Stress tests reach 66,000
reverse-ordered cases with at most 195 executed instructions per tested lookup and
12,000 nested switches with flat failed cleanup. Two typed legacy tests cover word
boundaries and noninteger/stale-reference rejection. Hostile typed/module streams
include integer admission. Selected pinned switch scope/type/sort/duplicate/domain
guards reject 39 mutations, and existing predicate/reference/call/argument guards
and 99 syntax replays remain valid. These are source-role guards plus Rust execution,
not original compiler/VM/switch cleanup execution. Full Windows workspace debug/
release tests pass after the final label check (214 frontend/89 legacy plus doctests),
along with formatting, denied-warning Clippy/rustdoc, repository policy (576 source
files/eight crates, largest 287 lines) and diff checks. No dependencies, new source
hashes or alternate targets. String switch, heap/general builtins/scheduler/game
services/full native scope/diagnostics/cleanup/mod and Linux acceptance stay open.
No complete legacy workstream is closed.

The explicit offline --execute-gsc PATH mode now compiles/executes supported
primitive source under ADR 0053. It shares bounded source/index/parse/declaration/
binding phases with inspection, rejects unresolved include/animation directives,
compiles all active bodies and selects canonical active main through declaration
mapping. It never falls back to entry zero; missing parameters receive initialized
Undefined. Default compiler/proof/global execution budgets apply. Typed result
data goes to stdout only; function results omit identity. Fixed operational events
retain original source/compiler/runtime locations and logical work/peak counts,
never source/paths/spellings/arguments/return contents. Write/flush failures propagate.
Pretty development and JSON/file/log-flush behavior remain shared. The mode opens
no listener and provides no unknown host/plugin/game services. Existing inspection
remains structural and now says explicitly that its mode does not compile/execute.
Eight real process tests cover calls/references/builtins/switch execution, all
primitive results, nonzero/case/developer/disabled declaration mapping, missing main/
directives/unsupported active bodies/services, located compile/runtime errors,
global infinite-loop fuel, privacy/human/JSON logs and conflicting/offline options.
A private output test injects write/flush failures. Existing inspection/binding/
query behavior passes. The public examples/gsc/primitive_demo.gsc ran through the
normal debug executable, printing integer 18: 64 compiled instructions/479 compiler
steps, 181 execution steps, two peak frames and four peak locals. Full Windows
workspace debug/release tests, formatting, denied-warning Clippy/rustdoc, repository
policy (581 sources/eight crates, largest 287 lines) and diff checks pass. Core
script suites remain 214 frontend/89 legacy plus doctests. No dependencies,
alternate targets or deployments. Imports, heap/general builtins/scheduler/game
services/full mod/join/Linux and CLI budget overrides remain open.

Next: source-grounded string/value ownership and broader builtin/semantic/call/frame
work. Full objects, calls, scheduling,
host services, modern language, and join/gameplay remain required. The user's
latest full-GSC task-count question was answered as roughly 10–12 major remaining
workstreams, each containing many implementation/testing tasks, not an exact task
inventory or delivery promise; expanded language/services require further work.
The audit also identified 32-value native compiler temporaries and short expression
jump patches; preserve configurable budgets/wide locations rather than those caps.
Further pinned source research finds a 64-local registration check in
Scr_CalcLocalVarsVariableExpressionRef and 32 call frames/2,048 value/local slots
in scr_vm.h. These native capacities must not become new internal allocation rules.
Scr_FindLocalVarIndex checks initialization bits and emits an uninitialised-variable
compiler error on invalid reads; allocating every local as a readable Undefined
would not preserve that behavior. Runtime initialized Undefined must stay distinct
from a slot that has never been initialized. Native arity policy needs separate research.
Research-only Git-blob verification for src/asmsource/_scr_vm.asm records canonical
SHA256 364d29c723327105116498dd6369ea710d27e00078738f4bb12de97b89c9567e
at the pinned revision, matching the checkout. That dispatch uses 135 entries/
opcode <=0x86 versus the header's broader 0x89 inventory; do not assume universal
VM/profile equivalence. The file is now added to upstream.lock.json for selected
update/argument-profile guards, not whole native runtime/adapter conformance.
Selected missing-Undefined/excess-error behavior is implemented; native evaluation/
push order, builtin signatures, receivers/frames and error cleanup remain incomplete.
Research in that same verified VM blob finds ++/-- dispatches require native type
6 (int), then add/subtract one 32-bit word at labels 690/740. Treating postfix
updates as arbitrary float-capable addition would be incorrect for this profile.
These selected source roles now guard the implemented scalar update path; original
VM/reference/error-unwinding execution remains unverified.

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
