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
