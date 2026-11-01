# Compatibility contract

ADR 0078 runs explicitly supplied packages through the real offline process, with
eleven process/two private tests for source/entry/developer/error/log/output and
aggregate admission behavior. It preserves current single-file/query modes and
rejects includes/animation directives in every file. This is not filesystem mount,
automatic dependency/preprocessing, native loader or stock/mod compatibility.

ADR 0077 adds closed supplied-file GSC compilation/execution with canonical module
lookup, local namespaces, strict qualified misses and developer visibility.
Thirteen integration tests cover wide/deep/owned/error/budget/heap boundaries;
38 mutations guard selected native source roles, without executing the native
compiler. Filesystem/include/deferred loading, cleanup and stock/mod parity are
not established. Existing single-module qualified calls remain unsupported.

ADR 0076 canonicalizes NUL-free ASCII virtual script names under native helper
rules. Of 579 actual O0/O2 outcomes, 574 replay through Rust; five NUL-containing
inputs intentionally reject ambiguous prefix identities. Twenty source-role
mutations and 4096 independent model cases guard the component. Non-ASCII ctype,
interner/loader/filesystem/native unwinding or full file-link compatibility is not claimed.

ADR 0075 adds 210 actual selected native lifetime/COW traces at O0/O2, an independent
owner-release model and 61 source-role mutations. Acyclic holder release clears
outgoing aliases immediately; cycles/downstream holders retain them, affecting
copy/key order. Selected clear/free-list publication executes with modeled monotonic
allocation/setup. This is not allocator reuse, VM/GC or nested-source ownership parity.

ADR 0074 adds staged populated copies for current primitives/shared structs.
All 20 native alias/COW states now replay with explicit host-owned aliases;
retained-cycle topology/markers remain unimplemented. Thirteen integration/two
private tests cover order/values/identity, resource failures, an independent model,
wide counts, bounded churn and copy work beside unrelated data. No native
allocation/freeing/ABI/compiler/VM/GC or nested-array parity is claimed.

ADR 0073 adds typed unique-entry storage for current primitives/shared structs.
Ten integration tests and two private tests cover wide keys, bounded churn, a
16,384-operation independent model, exact failures, roots and malformed input.
Three native U/V/F semantic-state replays cover ordering/raw Undefined/removal;
they do not establish native allocation/free-list/cleanup/VM parity. Shared writes
reject; populated copies follow ADR 0074. Nested values/COW and source arrays remain open.

ADR 0072 adds checked host-owned empty-array locations and empty detachment only.
Four zero-key native alias states replay with explicit host aliases; they do not
execute retained-cycle topology. Thirteen managed-value tests plus four key and
three private tests cover wide counts, bounded churn, an independent model and
atomic failures. Complete array source/VM behavior and native allocation/cleanup/
GC parity remain open. Binary host keys are not native interner conformance.

ADR 0069 adds ordinary struct identity equality/inequality. All 108 actual C
ordinary-pointer outcomes agree at O0/O2 with modeled records/ref callbacks;
36 source-role mutations reject. Selected C and archival marker reads use CHILD
namespace, so this corpus makes no array comparison inference. Wide/reference/
resource/source/process tests assess the checked Rust subset; native cleanup,
ABI/compiler/VM/GC parity and full mods remain open.

ADR 0067 adds ordinary struct source reads/simple writes and Undefined deletion.
The selected field-role guard rejects 99 mutations; 35 actual C helper traces
agree at O0/O2 with modeled lookup/ref/removal (27 Rust value replays/eight
helper-only function words). These do not establish native allocator/free/ABI/
compiler/VM parity. Wide/cyclic/deep/resource/source/process tests cover Rust
ownership and admission. ADR 0068 adds compound/postfix fields under selected
compiler/update-role guards (41 mutations), with explicit invalid-postfix behavior,
deep/exact-budget/pressure/failure/process tests. This does not execute the native
compiler/VM. Arrays and full mods remain open.

ADR 0066 adds bound-source empty struct construction. A selected-role guard rejects
22 mutations; 32 actual constructor/type/ref helper traces match O0/O2 with modeled
allocator/output services. Rust tests cover fresh empty references, wide values/
extras, pressure roots and source/process behavior, not native allocation/refcount
ownership, cleanup/ABI/unwinding/whole-VM parity. Arrays, entity lifetimes
and full mod execution remain open.

ADR 0063 assesses a shared struct storage component with 70,000 objects/fields,
100,000 churn cycles, deep/cyclic graph reclamation, exact resource failures and
an independent 4,096-operation graph model. These are Rust storage/ownership tests,
not native object VM/collector conformance. Copied ObjectId values preserve shared
identity without owning their store; executable constants and pure/string-only
invocations reject them. ADR 0064 permits checked host arguments through an explicit
object context. Seven typed/one source tests cover 12,000 frames, 1,000 compiled
loop/call allocations, caller/cache/external/nested roots, exact global limits,
partial completed collections and returned snapshot lifetimes. This does not claim
native refcount/GC/operator parity, arrays, entity
subtypes or scheduling. Those and full source object execution remain open.

The owned string-store component replays 54 native plain-concatenation successes
through literal decoding, storage, concatenation and explicit reclamation; another
54 native capacity failures intentionally succeed under larger budgets. These are
distinct counts, not native interning/reference-cleanup/VM parity. Foreign/stale
references fail explicitly; 70,000 live strings exceed the old pool width; ADR 0054.
The same 54 successes/54 expansions now pass typed VM calls and returns under an
explicit heap context. Source supplied-string arguments additionally exercise
locals/for/calls under pressure, with located errors. Source literal construction
and CLI strings follow in ADR 0057. Byte equality is source grounded; binary host
bytes do not establish native NUL/interner behavior. Runtime/rooting tests cover
12,000 frames, 100,000 loop allocations in four slots, every pressure resource,
exact-one-less work/scratch, failures/lifetimes and hostile streams. See ADR 0055;
translation/mixed coercion, objects/scheduling/full mods stay open.
Typed code literals/cache now pass 13 further tests, including owned binary bytes,
70,000 rows, cached-only roots, complete standalone/module overlap, exact-one-less
proof/runtime limits, independent heaps and repeated large values; see ADR 0056.
These are Rust ownership/resource tests, not native compiler/interner execution.
ADR 0057 now adds bound source plain literals and offline escaped byte output.
Nine source tests cover ownership, 70,000 distinct rows, 100,000-byte values, actual
calls/loops/maps, developer policy and exact compiler/proof budgets. Three process
tests cover JSON/human privacy, byte output, located failures and complete large
results. This is not native interning/ref cleanup, localized/mixed coercion, full
mods or client-visible capacity expansion. ADR 0059 now adds raw localized values,
same-type equality and escaped output, with 3,024 actual C identity/type/arithmetic
outcomes (144 results/2,880 type errors). Six kernel and six source tests plus two
process tests cover tags/ownership/rooting/maps/budgets/privacy, 70,000 mixed rows
and loop pressure. Native byte interning/ref cleanup and actual translation remain
open. Isolated scalar source profiles stay
restricted; the complete GSC and server gates remain open.

Plain-string indexing and Size now replay all 896 native helper outcomes (36
results/860 errors) with synthetic byte/ref/error services, not native interning/
cleanup/unwinding. Selected source guards now reject 46 mutations. Rust/source/process
tests cover first-NUL semantics, raw bytes, wide input, exact work, complete roots
and private located failures; see ADR 0061. ADR 0065 adds ordinary-struct Size=1
with 15 Rust replays from 180 actual pointer-size outcomes; 165 other-subtype rows
remain helper-only. Array/other-object size stays open.

Primitive isdefined follows tag-based definedness and required-first/evaluated-
extra rules. Its 351-row corpus executes isolated archival comparisons with
modeled parameter admission, not the native function/VM/lifetime/error services.
64 rows execute in Rust (56 results/eight missing errors), including ordinary
structs, and 287 stay helper-only. Wide source/typed count tests are capacity
expansion. See ADRs 0062/0065; entity/thread definedness stays open.

Compatibility targets are existing CoD4 and CoD4X clients, maps, and GSC mods;
the future custom client also targets ordinary CoD4X servers. Exact supported
releases must be recorded individually. There is currently no demonstrated live
client/server interoperability.

## Evidence levels

| Level | Meaning | Current result |
| --- | --- | --- |
| Source grounded | Behavior traced to a pinned implementation | Message primitives |
| Differential | Independent upstream bytes match Rust | 64 message vectors |
| Transport | Real peers exchange valid channel traffic | Not implemented |
| Join | Existing client authenticates, loads, and enters a map | Not implemented |
| Gameplay | Recorded movement, scripts, snapshots, and game rules match | Not implemented |
| Operational | Reconnect, rotation, downloads, load, and long sessions pass | Not implemented |
| Custom client | Named upstream releases pass bidirectional interoperability | Future work |

No evidence level implies a later level. A success response or an open UDP port
does not establish a working game server.

## Current byte contract

Source: pinned `src/msg.c`. Integers are little-endian. Bits are
least-significant-first. Byte and bit cursors are separate: byte operations can
occur between operations on the same partial bit byte. Readers support a logical
message spanning two slices, including fields crossing the split. Maximum
logical size is `MAX_MSGLEN = 0x20000`; this is not a UDP datagram size.

Binary32 reads and writes preserve bits, including NaNs and signed zero. This
does not define simulation floating-point arithmetic compatibility. Bit widths
are explicitly 0 through 32; upper value bits are ignored on write.

Deliberate API differences preserve valid wire bytes while changing failures:

- Upstream `MSG_WriteBits` reserves four spare bytes even for smaller writes.
  Rust requires exactly the storage needed, including cached bit space.
- Rust rejects an oversized buffer/message and invalid widths explicitly.
- Rust returns errors rather than `-1` sentinels and a mutable overflow flag.
- Operations fail atomically instead of partially reading/writing. Callers
  must abort the higher-level parse on error.

The message codec does not implement full delta fields or entity/player state
yet. Legacy user-command decoding, bounded strings, static Huffman coding, ordered
legacy entity/client indices, and direction-specific channel XOR are implemented.
Envelope framing, command-line extraction, and info strings are implemented.
No assumption is made that bytes above ASCII are UTF-8.

## Envelope, commands, and info strings

Channel headers preserve direction-dependent qports, 31-bit sequences, 32-bit
fragment offsets, 16-bit lengths, and the 1300-byte canonical fragment size.
Exact multiples require an empty terminal fragment. Special markers `0xffffffff`,
`0xfffffffe`, and `0xfffffff0` are classified before ordinary fragmentation.
Reliable and update-proxy bodies remain uninterpreted. Ordinary channel state now
supports single-owner send packetization and bounded receive reassembly.
The decoder rejects trailing fragment bytes and noncanonical fragment sizes
instead of accepting upstream's permissive malformed-input handling.

Command extraction stops at NUL/newline, bounds the line to 1023 bytes, and applies
the original percent-to-dot normalization to connectionless lines. The tokenizer
preserves original quote/comment boundaries, including empty/unclosed quotes.
Oversized lines are rejected instead of silently truncating the command.

Info strings preserve byte values, pair order, duplicate keys, optional initial
backslashes, and empty values. Lookup is ASCII-case-insensitive; builder replacement
removes the first exact-case key and prepends its new pair, matching upstream.
Profiles reserve the C terminator (1023/8191 encoded bytes). Structural errors,
ASCII controls, quotes, and semicolons are rejected. Failed replacement preserves
the old value instead of upstream's remove-before-capacity-check behavior.

The local discovery service implements the info/status wire response shapes with
an explicit query-only identity, zero slots, challenge echo, and stricter bounded
reply budgets. Real loopback process tests pass. It does not establish live game
client compatibility or implement connect/authentication/RCON handlers.

## Ordinary channel state

Send packetization matches original transmission fixtures after substituting the
explicit initial sequence field. A transmission borrows its channel and reserves
its sequence at begin, so cancellation never reuses partially sent message IDs.
Insufficient packet storage can be retried without advancing packetization.
Reconnection is required near reserved/wrapped sequence ranges.

Receive state checks route/qport, ignores delivered duplicates, waits for a short
terminal fragment, and preserves a prefix when a fragment is missing. It accepts
a retransmitted fragment at the expected offset. Capacity failures and invalid
new offsets preserve existing state. A newer partial message is protected from
older fragments; this deliberately avoids upstream's ability to replace newer
in-flight progress with an older sequence. Complete older packets can still be
borrowed without overwriting the newer prefix. Applications must authenticate
peers and implement timeout, pacing, and reliable-command policy separately.

## Static compression and ordered indices

The Huffman codebook is generated from the pinned original tree, including its
reserved NYT leaf. Encoding is byte-identical for all 256 byte symbols and the
recorded sample streams. The original decoder may finish a code beyond the
physical byte boundary; the safe decoder uses at most ten virtual zero bits for
that final code. Fixtures record this behavior, including extra padding symbols.
Logical message EOF must be validated by the higher-level parser. Compression
without an original decoded length cannot independently detect every truncation.
The reserved NYT symbol is rejected instead of being silently narrowed to zero.
Output validation precedes mutation and expansion is bounded by the wire profile.

Legacy ordered indices retain increment, short entity delta, and full-index forms.
Entity/client namespaces are explicitly named legacy types with 10/6-bit bounds;
they do not constrain engine storage. Decoder reads and encoder writes are atomic
on failure. Malformed non-increasing/overflowed indices are rejected. The 64
source-generated index streams include boundary and randomized ascending cases.

Legacy string writers append NUL and reject embedded terminators/oversized fields
without silently truncating. Reads preserve raw high bytes and replace percent
signs as the original reader does. Maximum-length valid strings consume their
NUL, correcting the original exactly-full-buffer cursor bug. Missing terminators,
capacity failures, and field overflow leave both output and reader unchanged.

The channel XOR implementation matches 48 original canonical cases. It transforms
only the payload after a caller-validated profile-specific plaintext header;
it is neither encryption nor authentication. A separate original-source fixture
records an empty-command bug where the C routine reads stale bytes after NUL.
The Rust implementation deliberately excludes those bytes from key material.
This safety difference remains explicit pending full peer interoperability tests.

## Legacy player commands

The decoder ports `MSG_ReadDeltaUsercmdKey` and its field/movement helpers from
the pinned source. It preserves branch-specific key/timestamp order, 10/-10
movement thresholds, conditional selected-location/melee fields, signed melee
short angles, and unchanged field bits. It is allocation-free and commits reader
cursors only after the complete command succeeds. The data type describes this
peer profile, not a C struct ABI or engine-sized asset/entity namespace.

Caller baselines must have representable 21-bit buttons, seven-bit weapon/offhand
slots, and a finite yaw within defined float-to-int conversion range. Invalid
baselines are rejected before reading instead of reproducing C conversion UB or
fallback logging. Timestamp byte deltas explicitly wrap signed 32-bit values.
The C oracle uses `-fwrapv`, signed char, and explicit binary32 evaluation settings
to record that behavior; this does not establish original x86 gameplay rounding.

Five tests include 512 original C results at all 65 storage splits, cursor-tail
comparisons, every command truncation, invalid baselines, and exhaustive 65,536
melee decode results checked through a C-produced digest. This implements a
decoder only. Command emission, packet opcodes/authentication, timing policy,
server acceptance, and movement simulation remain separate work.

## Release matrix to establish

Unsigned `IWffu100` version-5 envelopes use the pinned `db_load.cpp` twelve-byte
outer header and zlib-compressed 44-byte `XFile` header plus declared payload.
The decoder validates every declared accounting budget before output growth and
requires exact length, checksum, end-of-stream, and no compressed trailer. Signed
`IWff0100` remains unsupported, matching the baseline's explicit rejection. Extra
output and trailers are rejected deliberately. Fifteen independent Python fixtures
and two read-only stock samples establish envelope behavior, not graph/pointer
validity, decoded maps, authenticity, collision, or gameplay compatibility.

Root metadata follows `Load_XAssetListCustom`, `Load_ScriptStringList`,
`Load_TempStringArray`, and `Load_XAssetArrayCustom` plus the 32-bit `xassets.h`
layout. Eleven independent Python fixtures and two stock samples establish prefix
layout/virtual-allocation behavior, not original-C graph parity. Nonzero counts
with null tables fail explicitly; source code would omit those tables. Unknown
types and unresolved reference words remain preserved and uninterpreted. Expanded
root-table tests do not establish old-client string-handle or asset-pool expansion.

Serialized block-address classification matches ninety original-C in-bounds
relative offsets across nine blocks. Raw zero/-1/-2 remain distinct; other words
subtract one before decoding the four-bit block tag/28-bit offset. Invalid tags
and overflow/out-of-block spans fail explicitly. An admitted logical span is not
proof of loaded bytes, bound aliases, alignment, zone ownership, or target type.

The first typed collection supports only raw-file roots (type 31). It preserves
null entries, backward aliases, forward references to known null cells, -2 inserted
aliases, and direct/suffix names within known loaded string regions. Raw buffers
use the source's nonzero-presence rule and length + 1 read. Negative lengths,
inconsistent absent buffers, missing final NULs, and excess body bytes fail
explicitly. Null zero-length buffers and present empty buffers remain distinct;
interior NULs/binary bytes are preserved. All other root kinds and unsupported
reference targets are rejected before publishing a collection. Twenty independent
Python fixtures establish this subset; full loader/registry parity remains open.

The borrowed connect parser interprets only protocol, challenge, and qport; unknown
userinfo stays exact. It uses original tokenizer/first-userinfo-token behavior,
including ignored trailing tokens. Signed-short qport spellings emitted by upstream
and unsigned 16-bit spellings normalize to the same raw channel value. Four tests
include twelve independent C tokenizer/lookup/decimal cases. This is field parsing,
not a port of `SV_DirectConnect`, cookie verification, or connection admission.

Malformed decimals, missing required fields, overflow, and duplicate critical keys
(including case aliases) are rejected explicitly instead of `atoi` default/prefix
behavior or ambiguous first-key selection. Unknown passwords/names/extensions are
preserved; no claimed client IP/GUID is trusted. The request's Debug output exposes
only byte count. Raw userinfo views can contain credentials and must never be logged.

An explicit diagnostic handshake probe now produces the pinned challenge-response
shape, backed by an opaque modern endpoint cookie. Forty original complete-function
datagrams cover signed boundaries and client echoes, for 168 total wire fixtures.
Real UDP tests prove an exchange with the test peer, silent bad-cookie handling,
and gameplay-unavailable refusal after valid proof. No successful connectResponse
is sent, and no account is authenticated. This does not establish an unmodified
game-client join or full release/profile interoperability.

Record server revision, client build, negotiated protocol, platform, map/content
hashes, mod revision, plugin presence, and test result for every conformance run.
Keep credentials and proprietary assets out of fixtures. Cover baseline CoD4,
current CoD4X, and explicitly selected older CoD4X releases; expand the matrix
before claiming additional coverage.

Pinned upstream `net_game_conf.h` defines legacy protocol 6 and derives its other
protocol number from the server version; `version/version.c` currently specifies
21.2. Query responses may advertise protocol 6 while other version fields refer
to the CoD4X protocol. A single hard-coded protocol integer is insufficient for
the future client. Do not infer compatibility from marketing version text.

## Compatibility suites required before release

The script kernel has original-function numeric/wait vectors and a new bounded
source API. Legacy raw lexing additionally has 724 recovered-table/action replays,
not original x86/parser execution. It preserves raw spelling/trivia and terminal
codes without interning names or decoding literals. Embedded NULs/invalid bytes,
unclosed comments, and exhausted work budgets fail explicitly; no input tail is
silently dropped. New keywords are not activated in this profile. Full semantic
values, compiler/VM, scheduling, builtins, modern extensions, and stock-mod parity
remain open release gates, regardless of successful source indexing or lexing.

Bounded literal conversion and structural syntax are now separate implemented
stages. Ninety-nine recovered-grammar replays compare acceptance and successful
reduction order without original parser/semantic-action execution. Script and
fragment trees retain source spans and support configurable deep stacks. No names
are bound, no complete semantic AST is produced, and no script executes; passing
structural syntax alone does not imply valid literals or compilable/runnable GSC.
Local function binding now has explicit developer modes and source-grounded region/
duplicate checks. It retains inactive definitions and borrows canonical ASCII keys;
125 original conversion-helper cases cover short folding/long exact-case fallback.
String/ref services are stubbed in that oracle, and native compiler execution is
not established. Imports, calls, parameters/locals, builtins, bytecode, and full
GSC semantics remain open; see ADR 0019.
An additional 12,416 original-helper cases now cover scalar coercion, six numeric
comparisons, and float-result division under explicit nearest/SSE conditions.
Arithmetic NaN payloads and native error unwinding are excluded. Approximate
equality and negated unordered predicates must not be replaced by Rust operators.
Plain-string concatenation additionally preserves 54 native helper results while
intentionally admitting 54 old scratch-capacity errors under configurable result
budgets. The source harness stubs string/ref services; neither full interning nor
VM error unwinding is asserted. Larger owned strings do not widen wire/client
profiles. See ADR 0021 for the explicit compatibility change.
Scalar add/subtract/multiply and integer bitwise/remainder/shift behavior now has
10,701 defined original C cases plus 818 isolated instruction cases. The latter
record an archival x86 profile, not execution of full original assembly routines.
Signed overflow wraps, shift counts use five low bits, and MIN%-1 becomes a typed
error rather than a native quotient fault. See ADR 0022; this is not VM conformance.
Typed expression views additionally expose original operator/operand structure
without executing semantic actions. Source profiles guard wrappers, sixteen eager
operators, two logical branches, and two prefixes. Primary descendants stay unresolved,
including structurally accepted literals that later value validation may reject.
See ADR 0023; no compiler execution is established.
Separate primary views additionally classify all 22 primitive productions without
native semantic actions or value conversion. Signed literal tokens, grouped lists,
unresolved call/reference subtrees, and animation/debugger forms retain their exact
roles/spans. These are local frontend views, not runtime feature support; see ADR 0024.
Expression lists now preserve complete source order with budgeted fallible indices.
The native actions prepend nodes; runtime argument evaluation/push order still
needs compiler/VM evidence. Empty/single/nested lists and 70,000 retained entries
establish frontend representation only, not call/vector arity or execution. See ADR 0025.
Named/external/indirect call views additionally retain original function/module
tokens, optional receivers, immediate/thread syntax, and argument-list locations.
Constructors do not fold case, link names, interpret virtual paths as host access,
evaluate arguments/receivers, or create threads. See ADR 0027; this is not call/VM parity.
Local reference views preserve five source forms, including reserved .self versus
ordinary .Self fields. Names and nested index/object operands remain unbound and
unevaluated; writable slots or debugger authority are not inferred. Source profiles
and typed views do not establish native reference execution. See ADR 0028.
The owned scalar kernel now executes validated typed instructions through the
existing numeric helpers. Its 22,383 source-grounded cases cover arithmetic/
instruction-profile results, comparisons, division, and numeric failures. A
separate 4,096-stream hostile corpus exercises verifier/runtime contracts. Typed
instructions have no native/serialized ABI; undefined is returned/copied/discarded,
not a claim of complete legacy value operators. Source compilation, native VM
cleanup/scheduling, stock scripts, and GSC/client interoperability remain open.
See ADR 0029 for scope, fuel accounting, and FP-profile assumptions.
A restricted source compiler additionally connects typed legacy functions to that
kernel for zero-parameter empty/single-scalar-return bodies. Numeric/undefined/bool
literals, single-value groups, numeric operators, and lazy OR/AND are supported;
other parameters/bodies/values fail explicitly. Code/source maps own no source
references, and verification plus bounded pure validation precede publication.
Signed -2147483648 uses an explicit checked domain policy, not undefined native
atoi-overflow evidence. Twelve integration tests exercise this restricted path,
including resource limits and wide branches; pinned body/action profiles do not
execute the original compiler. Full module/stock-script compilation, native
diagnostic ordering, calls/objects/scheduling/host services and interoperability
remain open; see ADR 0030.
Typed statement views additionally retain all 37 direct statement productions and
pinned wrappers without native semantic actions. Specialized waittill/match/notify
lists remain distinct from ordinary expression lists; endon retains one expression.
Ordered body indices preserve immediate source order and labels without flattening
nested bodies. Fourteen tests cover roles, wrappers/source privacy, independent
trees, deep/wide input, and complete body-list admission. These do not establish
scope/lvalue legality, event argument semantics, developer/debugger availability,
general statement compilation/execution, scheduler or stock-mod compatibility.
The existing restricted compiler reuses return classification; see ADR 0031.
The scalar executor now additionally handles typed local load/store instructions
and immutable parameter/local layouts. Runtime uninitialized reads differ from
initialized Undefined; source binding must still enforce native compile-time
initialization rules. Ten tests exercise exact internal arity, caller isolation,
fresh selected-path initialization, operand/header/budget failures, 70,000 internal
arguments/slots, and another 4,096 hostile streams. These internal capacities do
not establish source parameter/local support or client-visible expansion. Native
missing/extra-argument policy, full local binding/definite assignment, call frames,
source general-body execution, and VM parity remain open; see ADR 0032.
A separate scalar-local source profile now compiles parameters, ordinary name
assignments/reassignments, empty statements, and final/fallthrough returns. It
uses legacy identifier identity (including long-name exact-case fallback), checks
initialization in source order, and keeps stored Undefined distinct from missing
assignment. Eleven integration tests cover real source invocation, source maps,
privacy/ownership, unsupported syntax, budgets, 70,000 local names, 100,000-byte
names and 12,000 groups. Publication verifies without made-up parameter values;
runtime numeric errors remain located. Duplicate-parameter/nonfinal-return
rejections are explicit restricted policies, not complete native compiler
acceptance/error-order claims. Source control-flow binding, native arity, calls,
objects/threads/events/builtins, modules/stock mods and full VM parity remain open;
see ADR 0033. The original pure-return entry point retains its accepted syntax and
validation behavior; optional validation accounting makes the profile difference explicit.
Ten plain-name compound assignments now compile through the same scalar numeric
instructions. Three tests exercise all operators, wrap/shift/division domains,
mixed floats, aliased RHS/prior target values, parameter mutation isolation,
uninitialized/non-name targets, runtime numeric/type errors, compound source spans,
and exact compiler/name accounting. Source rules/opcode guards and prior numeric
fixtures ground the reuse. Richer overloads, object/index references, side-effecting
RHS ordering, postfix updates and whole VM/compiler parity remain open; see ADR 0034.
Integer-only plain-name postfix statements now compile/execute through a dedicated
unary operation that reuses established wrapping arithmetic. Five tests cover
integer boundaries, float classes, kernel proof/fuel/type failures, source locals/
parameters, target admission, exact budgets and update-token error maps. Both
bounded hostile instruction corpora now include updates. Pinned compiler/VM
source guards establish selected type/word-operation roles, not original handler
execution or complete reference/unwinding behavior. The native VM dispatch ends
at 0x86 while the header also lists debugger opcodes through 0x89; this is not
complete profile equivalence. Postfix expressions, object/index references, modern
numeric updates and full GSC/VM/stock-mod acceptance remain open; see ADR 0035.
An explicit scalar legacy script-argument profile now initializes missing parameter
suffixes to Undefined and rejects excess values. Supplied arrays are already in
source parameter order; nonparameter locals remain uninitialized. Seven tests
cover prefixes/defaults/strict-API regression, float bits/caller isolation, wide
prefixes, source functions/maps, resource limits and huge checked failures. The
hostile local corpus additionally invokes both binding profiles. Source guards
tie selected parameter/safe-set/checkclearparams/creation roles to the pinned
code without executing original compiler/VM routines. Builtin arity, native
argument evaluation/push order, frames/receivers, cleanup/unwinding and whole
GSC/mod compatibility remain open; see ADR 0036.
Scalar-local ordinary blocks and if/if-else now lower to owned verified branches.
Stable function-local slots use conservative initialization joins over structural
paths reaching a read; returning paths are excluded, incoming facts are restored
before else, and constant-condition source is still checked on both sides.
Immediate nonfinal returns and later structurally unreachable statements fail.
Eleven added tests cover actual path results, initialization/return policy, maps,
exact resource boundaries, 70,000 local snapshots, long forward jumps and 12,000
nested branches with bounded failure cleanup. Flag payload/work is explicitly
admitted. This is not original compiler execution or full native scope/constant
folding/diagnostic-order parity; loops/switch and richer values remain open. See ADR 0037.
Scalar while now uses consuming numeric conditions, verified absolute backward
jumps and deterministic instruction fuel. Snapshot exits conservatively restore
incoming initialization; body-only writes never authorize later reads. Nine
tests cover execution/returns/errors, exact compiler/fuel admission, wide/deep
loops and failed cleanup. A selected pinned source guard connects while emission
and native branch/backward opcode roles; nineteen in-memory mutations are rejected.
Original compiler/VM routines and the native timing watchdog are not executed.
Constant folding/scope/diagnostic order, for/break/continue/switch, general values
and full GSC/mod compatibility remain open; see ADR 0038.
Scalar while break/continue now preserve nearest-loop targeting, terminating-path
joins and conservative exit initialization. Illegal contexts/unreachable source
fail with original keyword/statement spans. Tests cover actual nested transfers,
caller isolation/fuel/maps, exact compiler quotas, 20,000 linked break fixups with
wide destinations and 12,000 nested contexts with failed cleanup. Selected pinned
compiler context/legality/fixup/restore and VM jump roles are guarded; 38 in-memory
profile mutations are rejected. This does not execute native compiler/VM routines
or establish native block cleanup/scope/watchdog/error-order parity. For/switch
contexts and full GSC/mod acceptance remain open; see ADR 0039.
Scalar for now lowers optional phases and directs continue through the update.
Update initialization intersects reaching continue/fallthrough paths, excluding
break/return. Clauses accept scalar assignment/update/empty forms. Unreachable
updates check incoming facts; exit initialization conservatively allows zero
iterations. Tests cover optional combinations, actual phase/transfer behavior,
joins/errors/maps/fuel/caller isolation, exact compiler admission, wide deferred
fixups and deep contexts/failed cleanup. Selected grammar/compiler phase guards
reject 35 in-memory mutations. Native compiler/VM routines are not executed;
scope/constant folding/error ordering, switch, richer values and whole GSC/mod
parity remain open. See ADR 0040 for the restricted diagnostic policy.
Standalone owned legacy vector helpers now match construction, weaker numeric
broadcast, component-wise operators and exact equality under the named FP profile.
Addition rejects mixed vector/numeric inputs through its distinct coercion rule;
ordering fails and any signed-zero denominator coordinate returns a typed error.
The corpus has 19,852 original-helper cases: 11,580 successful value/coercion/math
outcomes and 8,272 intercepted error-category outcomes, with bounded stubbed
pool/ref/string/error services. Fourteen original functions and math macros run at
O0/O2; construction/coercion bits and finite arithmetic bits match, arithmetic NaN
payloads do not. q_math.c was verified against the existing pinned Git blob before
hash locking. Three tests cover the corpus, boundary distinctions, data isolation,
domain rejection and redaction. Source evaluation/push order, native constructor
failure/ref/cleanup/unwinding, kernel/source vectors and whole VM/mod acceptance
remain open; see ADR 0041.
Primitive-kernel vectors now execute construction and numeric/vector operations
under independently verified stack effects, actual payload byte limits and fuel.
18,116 original construction/arithmetic/comparison cases execute through owned
programs; 1,736 coercion-only rows remain standalone helper evidence. New tests
cover copy/default/local isolation, invalid components/truth/integer operations,
underflow, exact stack/local/fuel limits, 70,000 vector argument slots and redaction.
Both hostile corpora include vectors/MakeVector; local hostile streams also use
vector arguments. Existing restricted source accepts vector parameters at its
low-level program API and retains source operation maps. Source three-element
groups remain explicitly unsupported, and internal coordinate stack order is
not native/source evaluation-order evidence. Native VM error indices/cleanup,
heap/calls/scheduling and full GSC/mod parity remain open; see ADR 0042.
Compiler Rust APIs now use function names and explicit PureScalarReturn/
PrimitiveStatements profile names, with no accepted-syntax/runtime/quota/proof/
map change. This is an unreleased API rename, without compatibility wrappers or
a wire/content/serialized-bytecode change; see ADR 0043. Existing suites retain
their coverage and unsupported GSC gates.
Primitive source vector triples now lower with explicit z/y/x evaluation and
reversed constructor slots, under one builder-owned source profile. Singleton
groups and PureScalarReturn restrictions remain. Tests execute vectors/locals/
operations/loops, retain boundary bits, check rightmost component errors, source
maps and skipped/invalid syntax, exact quotas, wide locals and deep flat cleanup.
The parse-tree helper was verified against pinned Git before hashing; selected
grammar/prepend/compiler/VM/constructor roles reject 33 in-memory mutations.
These do not execute original parser/compiler/VM code or prove full side-effect/
constant-folding/error-index/ref-cleanup parity. Richer values, vector indexing,
calls/scheduling and whole GSC/mod acceptance remain open; see ADR 0044.
Vector index reads now preserve strict integer 0..2 bounds and exact returned
float bits through helper, primitive kernel and source. Kernel operand order and
source index-before-object traversal follow selected guarded roles; invalid
objects/types/ranges fail without retaining data. 299 actual Scr_EvalArray vector
outcomes execute at O0/O2 with bounded stubbed runtime services: 39 reads/260 errors.
All replay through owned verified programs; the numeric-only helper covers 286
rows, with 13 nonnumeric vector-index cases outside its typed domain. Source and
typed tests cover maps/order/default/caller isolation, proof/byte/fuel/compiler
quotas, wide reconstruction and deep flat failure. Both hostile corpora include
Index. Selected grammar/tree/compiler/VM/helper/write-rejection roles reject 27
mutations. Native parser/compiler/VM, service/ref/error cleanup and general
indexable heaps/mod parity remain open. See ADR 0045.
Undefined comparison dispatch now matches 402 actual pinned C-helper outcomes at
O0/O2: two equality/inequality successes and 400 mixed/ordering errors. Every row
runs through an owned verified program. Source tests cover both profiles, lazy
branches, missing arguments, stored Undefined versus uninitialized locals,
operator maps and private errors. Existing numeric/vector corpora retain their
distinct language rules. Native object/string/ref/error services remain stubbed;
this does not prove native cleanup/VM execution or complete value parity. ADR 0046.
Typed direct-call inventories now verify all target operands and per-function
flow before atomic proof publication. Runtime tests cover forward/mutual/recursive
calls, ordered/copied/default arguments, caller operands/locals and fresh error
recovery. Exact/one-less global verification/instruction/setup/frame/operand/local
count/byte limits, repeated shallow calls, 12,001 flat frames, 66,000 functions,
wide locals, 70,000-argument calls/missing suffixes and 4,096 hostile inventories
exercise configured expansion. These
are Rust typed-runtime tests, not original compiler/VM execution. Call argument
stack order is an internal convention. Excess count checks now run at invocation,
after caller argument instructions and before callee allocation/body; skipped
calls do not fail. Proof still requires reachable stack operands, including huge
counts; parameter count remains distinct from complete local slots. Three added
tests and hostile inventories retain those errors/order/maps/private boundaries.
Selected original argument source roles do not prove complete native diagnostic/
evaluation/error-index/ref-cleanup parity; see ADR 0048.
Imports, methods/receivers, scheduling/native cleanup
and whole-GSC/mod acceptance remain open. See ADR 0047.
Local bound source calls now compile into complete primitive inventories under
shared canonical name resolution, explicit developer inclusion/caller visibility
and atomic code/layout/map publication. Fifteen source tests cover real forward/
mutual/recursive calls, discarded results, loops/vector values, rightmost argument
error/name order, missing/excess values, privacy/ownership/maps, disabled/empty
metadata, unsupported forms and exact/one-less global/individual/nested scopes.
Source expansion tests reach 70,000 arguments/parameters, 66,000 functions and
12,000 nested call expressions. Selected resolved/deferred developer reference
roles reject 11 mutations; argument roles reject 38 without executing native code.
Existing 99 structural fixtures remain unchanged. Original compiler/VM/linker,
native constant folding/scope/diagnostic/ref/unwind parity, imports/animation
resolution, richer heap values, receivers/builtin/thread/host behavior,
whole mods and Linux remain open. ADR 0049 preserves isolated profile rejection
and allows empty inventories without fabricated functions; individual empty
instruction streams still fail. No source-role or Rust subset test closes full
GSC compatibility by itself.
Local function-reference values and immediate indirect source calls now share
canonical target/developer resolution, source maps and compiler quotas. Opaque
module identities never wrap/reuse; invocation rejects foreign/stale owners and
shares runtime arity/global frame/fuel/setup/count/byte admission. Tests cover
copy/return/factories, higher-order recursion, evaluation/type/error order,
parallel uniqueness/exhaustion, wide/deep/hostile/recovery and exact limits.
All 2,664 actual C comparisons pass with synthetic identity labels mapped to valid
Rust references (72 successes/2,592 errors). Selected source-role guards reject
31 mutations, without native compiler/VM/code-pointer execution. ADR 0050 leaves
self/receivers, heap/closures, imports/reload, builtins/scheduler/game services,
full-mod/native cleanup/diagnostic and Linux gates open.
Full strings/vectors/objects/arithmetic, compiler/VM, and gameplay remain open.
An explicit --execute-gsc offline mode now compiles all active local primitive
bodies and executes canonical main under existing budgets. It rejects unresolved
include/animation directives and missing/disabled entries, preserves declaration
mapping/developer policy and supplies the legacy empty argument prefix. Real
process tests cover success and located compiler/runtime/fuel failures, every
primitive result category, output/log privacy, unsupported services/whole bodies,
JSON/human diagnostics and offline/conflicting options. Results go to stdout;
function results omit identity, and write/flush errors propagate. See ADR 0053;
inspection remains structural, and full GSC/mod/gameplay/Linux acceptance stays open.
Primitive integer switch now supports default placement, adjacent labels,
fallthrough, nested break/continue ownership and path initialization joins. Strict
integer admission preserves words without float coercion. Valid native in-range
cases retain their results; full signed words and more than 65,535 cases are
explicit capacity expansions, not native invalid-input parity. Tests cover 66,000
cases with balanced dispatch, 12,000 nested switches/flat failed cleanup, all quota/
join/nesting/type/privacy/unsupported/recovery paths and global fuel. Selected source
guards reject 39 mutations without executing native compiler/VM/switch routines.
Case/fact payload shares compiler control/metadata admission; typed code uses
existing proof/fuel/stack limits and wide addresses. See ADR 0052; string switch,
full heap/scope/native diagnostics/unwind/mod and Linux gates stay open.
Four pure builtin calls execute from bound source: isint/isfloat/isvector/isstring.
Active script definitions shadow them; existing developer policy and canonical
names remain enforced. Exact-one arity checks run after supplied arguments, so
unselected wrong counts do not fail and argument errors retain precedence. Typed
instructions share existing resource limits without native 255-count ceilings,
new host/plugin authority or frames. Of 832 actual C O0/O2 count/tag outcomes, 224
currently representable/bounded cases execute in Rust; 608 are explicitly helper-
only, not full native value/VM coverage. Selected source guards reject 46 mutations.
Source tests reach 256 arguments and 12,000 nested calls; typed fixtures include
70,000 arguments. Exact count/byte/fuel/proof, unsupported authority, maps/privacy,
developer/shadowing/order/arity/hostile/recovery checks remain required. isstring
checks plain tag 2 and rejects the now-implemented localized tag 3. See ADRs 0051/0058/0059;
general builtins, heap/isdefined/entity lifetime, scheduler/game services, full-mod
and Linux gates stay open.

- Connectionless and sequenced packets in both directions, exact framing,
  sequence handling, NAT qport behavior, fragmentation, and compression.
- Authentication, challenges, reconnects, reliable commands, downloading,
  pure checks, file checksums, master listing, and legacy fallback.
- Snapshot baselines/deltas, user commands, entity limits, voice, demo behavior,
  and any supported CoD4X-specific messages.
- Stock and custom maps, archives/fastfiles, GSC compilation/VM/builtins,
  mod assets, expected engine quirks, and replayable prediction comparisons.
- Duplicate/truncated/malformed traffic, exhausted budgets, abusive downloads,
  replay/spoof attempts, shutdown, restart, and long-running memory behavior.

Native plugins require an explicit ABI/architecture compatibility decision and
are tracked separately from GSC mods. Optional ApX features require successful
negotiation and fallback tests with unmodified peers before use.
