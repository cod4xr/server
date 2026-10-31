# Compatibility contract

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
Full strings/vectors/objects/arithmetic, compiler/VM, and gameplay remain open.

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
