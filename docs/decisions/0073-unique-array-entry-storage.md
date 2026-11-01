# ADR 0073: Unique-value typed array entries

Date: 2026-11-01

Status: implemented storage component; populated COW/nested values and source arrays open.

## Decision

Extend managed data from ADR 0072 with integer/byte-key entries containing existing
KernelValue primitives or ordinary shared structs. Reads accept nonowning data
snapshots. Writes/removals require a checked managed location with exactly one
owner; shared values return SharedArray without mutation or detachment. This
explicit unique-entry API is not a source assignment instruction or populated
copy-on-write implementation. No array KernelValue tag or source activation is added.

Reuse the existing flat field arena, metered index, sibling links, key/value quotas,
fallible insertion and staged reclamation. This avoids a duplicate array entry
collector/index or growing every struct field's key/value record. A private borrowed
FieldKey distinguishes unchanged struct byte queries from typed array queries.
Array integer keys use a lossless zero tag plus eight little-endian bytes; byte keys
use a one tag plus the complete payload. This is an internal owned-key representation,
not packed native names, a wire format or an inherited engine limit. u64::MAX and
arbitrary binary/NUL byte keys remain representable. Native input validation remains
the separately named legacy conversion from ADR 0072.

Lookups borrow queries without temporary encoded-key allocation. Hashing includes
owner and key domain; comparisons inspect complete encoded content after prepaid
byte visits. Integer/byte keys cannot alias, including equal-looking raw encodings.
The toolchain-private hash is not a stable ABI or a digest. Struct hashes, raw key
behavior and logical operation costs remain unchanged. Collision probes and complete
key copies remain separately metered. Key quotas include the private tag: nine
bytes per integer or one plus byte payload. Parent classification prevents ordinary
field APIs from reading/writing array encodings.

New keys prepend; replacement preserves position; explicit removal unlinks and
reuses field slots. Raw Undefined remains a stored entry until explicit removal.
Legacy literal/computed Undefined assignment, autovivification, target evaluation
and error-cleanup order remain future source/VM work. Shared writes reject even
when a key is absent. The empty-only detachment API now returns ExpectedEmptyArray
on populated data, preserving its content instead of treating valid state as corrupt.

Key enumeration returns checked borrowed ArrayKey rows in sibling order, with
fallible vector allocation and complete row/work/scratch admission. Heap borrowing
prevents mutation/collection while key payload slices are live. Partial lists never
escape errors. Data/entry snapshots do not acquire owners; ordinary object/string
results need complete supplied roots before later collection.

## Ownership, collection and alternatives

Managed locations remain roots. Existing graph tracing follows ordinary struct
references stored in array entries, including cycles; composite string collection
includes both plain/localized entry values. Sweeping preflights typed key encodings
and deletes dead entries/data together. Owner/location counters and nonwrapping
generations retain ADR 0072's contracts. Nested array values and array-containing
struct fields cannot yet be represented through supported APIs, so cycle-alias
marker behavior remains unimplemented.

A separate array arena/index would duplicate ownership and collection logic. A
general owned-key enum would grow all field records. Canonical encoding retains
existing record sizes and fallible allocation paths at the cost of one owned tag
byte and a narrowly private key-query/decode layer. Public APIs expose typed keys,
never encoded buffers. This component deliberately requires unique data until
budgeted populated/nested COW is implemented; it does not silently choose shared
mutation or copy-free array semantics. No crate/dependency or source instruction changes.

## Evidence and remaining gate

Ten entry integration tests cover exact typed/binary keys, ordering, raw Undefined,
shared-write rejection, struct/string roots, resource/work/scratch boundaries,
hostile references, 70,000 mixed entries, 100,000 insert/replace/remove cycles within
one field slot, and 16,384 operations against an independent map/order model.
Private tests force hash collisions across domains and malformed key encodings.
Mixed collection now exhausts every failed prefix with populated live/dead arrays.
Three native U/V/F semantic states replay replacement, raw Undefined and
delete/reinsert keys. Native allocator/free-list/cleanup or VM parity is not claimed.

The runnable array_entry_workload reports logical storage/work/scratch only.
One Windows debug observation retained 70,000 fields, 262,144 buckets, 630,000
encoded-key bytes and 11,474,392 logical metadata bytes. Insertions charged
2,745,184 steps; enumeration charged 140,001 steps/1,120,000 scratch bytes.
Churn retained one field slot, zero keys/buckets and 192 logical metadata bytes,
charging 8,199,999 operation steps with 128 peak scratch bytes. Observed times
were 254,566/658,515 microseconds; these are not native-performance/RSS guarantees.

Complete populated/nested COW, cycle-alias markers, ownership transfer/unwind across
frames/fields/returns/suspension and source lvalues before activating arrays.
Proposal 0071 and the complete GSC/S5 gate remain open. Windows workspace debug/
release, all-target Clippy, rustdoc, formatting and policy passed: 268 frontend,
216 legacy tests, 11 script unit tests and two doctests; 819 source files/eight
crates, largest source 287 lines. Original 306 array and 20 alias traces, selected
write-role guards (141 mutations) and 99 syntax replays passed. Linux/hosted CI
remains unrun locally.
