# ADR 0010: Borrowed IW3 root metadata before typed asset graphs

Date: 2026-10-31

Status: accepted and implemented; full-workspace validation recorded separately.

## Context

The unsigned envelope produces serialized bytes, not native pointers. Pinned
`xassets.h` defines the 16-byte 32-bit `XAssetList` root and eight-byte records;
`db_load.cpp` reads the root, script pointer table/inline strings, and all asset
records before loading their bodies. Its stream-block allocations have alignment
gaps that consume virtual block space without consuming compressed input bytes.
Reproducing native pointer mutation or legacy script-handle narrowing would tie
the new engine to the original architecture and import unsafe assumptions.

## Decision

Add `ZonePrefix` under content/zone, borrowing an integrity-checked `FastFileStream`.
Decode signed nonnegative root counts, preserve pointer-presence behavior, admit
configurable counts/string/metadata budgets, and fallibly allocate a script-slot
index. Inline strings borrow exact non-UTF-8 bytes; null, inline empty, and unresolved
encoded references are distinct. Raw unknown asset kinds and all header words stay
in an ordered borrowed directory. Do not follow or validate those encoded pointers.

Check prefix allocation against declared stream block four. Count alignment gaps
only in logical virtual allocation, never by skipping serialized bytes. Preserve
the unvalidated body/deferred-data remainder for later typed readers. No engine
script IDs, intern pool, asset registration, block allocation, or map activation is
introduced. Root count fields remain this format's signed 32-bit representation;
internal storage and budgets do not inherit u16 handle or fixed pool limits.

Reject nonzero counts with absent tables rather than accepting an omitted catalog.
This is an explicit stricter malformed-input rule. Unknown kinds and unresolved
references are retained for typed schema handling, not silently dropped. A parsed
prefix is not a validated dependency graph. Debug exposes counts and lengths only.

The separate offline `--inspect-zone-prefix` mode emits aggregate root counts and
an explicit unvalidated-body message. Envelope-only inspection retains its existing
contract. No listener opens, content is not extracted, and names/payloads stay out
of logs and Git. Public-server admission must wait for full graph validation.

`ZonePointer` separately classifies null/-1/-2 and checked block-relative offset
words. `ZoneAddress` uses wide checked span arithmetic against declared block
sizes, never native pointers. Classification and capacity do not prove loaded
residency, alias bindings, ownership, alignment, or the target type. Prefix parsing
keeps raw unresolved words; callers request header classification explicitly and
must still apply the asset schema's pointer rules before interpreting any target.

## Alternatives and migration

Native-pointer relocation would violate safe ownership and 64-bit portability.
Copying every string/record would waste memory before semantic loading. Rejecting
all unknown types would prevent useful inventory of expanded content; accepting
them as registered assets would overstate validation. A borrowed prefix is a
concrete intermediate representation; typed graph readers will consume its records
and remainder while retaining stream ownership. Wider engine handles are assigned
only by their actual registries, not by narrowing serialized table indices.

## Validation

Eleven independent Python struct/zlib fixtures cover null/inline/encoded-reference
slots, unknown/repeated types, virtual alignment, truncated tables/strings, missing
tables, negative counts, and insufficient block allocation. Six Rust tests cover
every metadata budget at exact/one-less boundaries, all sample prefix truncations,
deterministic mutations, independent ownership, zero-copy remainders, redacted Debug,
and 70,000 string slots plus 70,000 asset records without truncation. Two additional
process tests cover counts, invalid roots, sanitized failures, and mode separation.

Read-only Rust and independent Python prefix inspection match the two stock
fastfile hashes recorded in docs/validation.md. These are layout/metadata checks,
not original-C graph differential tests or evidence of string interning, reference
resolution, decoded geometry, GSC execution, game-client compatibility, or gameplay.

Four further tests classify pointer words and check logical spans. Ninety relative
offsets come from the original `DB_ConvertOffsetToPointer` over real in-bounds
allocations in all nine blocks. The C function deliberately narrows native pointers
to DWORD; the harness records only unsigned relative offsets modulo 2^32, never
host addresses. Sentinel/28-bit-boundary/invalid-tag/overflow tests are separate
Rust checks. No invalid C pointer/index is exercised. This establishes conversion
arithmetic, not a safe native ABI or full graph relocation.
