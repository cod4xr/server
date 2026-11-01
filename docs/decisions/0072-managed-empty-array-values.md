# ADR 0072: Managed empty-array values

Date: 2026-11-01

Status: accepted component; populated arrays, VM ownership and source activation open.

## Decision and scope

Implement the empty-data portion of proposal 0071 as checked host-owned value
locations separate from nonowning data snapshots. Explicit copy/retain creates a
location; copying the raw handle adds no owner. Each live location is an explicit
collection root until release. Release invalidates its generation and decrements
the data's u64 owner count, but does not immediately free data. Last-owner data
remains snapshot-accessible until explicit collection. This is not native freeing
or refcount/GC parity, nor a frame/scheduler lifetime policy.

Empty detachment creates fresh data only for the selected shared location. Unique
selection does not allocate. Shared-struct access and wrong-tag VM arguments reject
array data, so raw ObjectId mutation cannot bypass COW. No KernelValue array tag,
indexed storage, nested copy, cycle-alias marker or source-array instruction is
introduced. Empty source arrays remain explicitly rejected by a frontend test.

Array data uses the existing parent arena. Lazy parallel metadata classifies
parents and counts owners without growing ObjectSlot or allocating metadata rows
in struct-only heaps. Once enabled, rows track retained parent slots, including
vacancies. A separate generational value arena has its own lazily minted full
HeapIdentity namespace; neither slots nor generations are packed/truncated. Both
arenas reuse vacancies and retire exhausted generations. Public Debug redacts IDs.

Live parents include array nodes. New logical usage counters expose live arrays,
live managed locations, retained value slots and retained classification rows.
Independent configurable value/slot quotas default to one million. Metadata
admission includes actual row sizes; geometric spare capacity, allocator overhead,
fixed fields and RSS are excluded. Every growth/resolution/reference/root scan is
metered, including scans of vacant managed slots. Fallible reserves and complete
work/resource/overflow preflight precede nonfallible publication; failure can retain
spare capacity but changes no logical usage, identities, locations or owner counts.

Collection and read-only cycle analysis include all live managed locations alongside
complete supplied struct roots. Snapshots are nonowning. Released data is collectible;
sweeping clears classification before parent reuse. Struct-only root/work/storage
behavior is preserved. This profile cannot represent array-containing fields/cycles;
it therefore does not implement or validate proposal 0071's retained-cycle markers.

ArrayKey separates wide u64 storage indices from borrowed exact byte keys. The
named legacy conversion accepts integer 0..8,388,608 exclusively, refuses floats
and localized values, and validates plain-string ownership without allocation or
scanning. Full binary/NUL bytes are host extensions, not native interner behavior.
Keys grant no lookup or mutation authority; owning/hashing them remains future work.

## Alternatives and consequences

Raw copied IDs cannot represent managed value ownership; implicit Rust Clone or
reference counting would also hide fallible quota/work admission. Distinct checked
locations make copies/releases explicit and permit bounded reuse. A tagged parent
record would be simpler, but would grow every struct record; lazy parallel rows
keep struct-only logical arena costs unchanged. The consequence is one admitted
classification row per retained parent after the first array, plus an extra value
arena and explicit release obligation for host callers. No existing script value
or immutable-code representation changes in this component. VM/scheduler migration
needs the ownership-transfer/unwind decisions still open in proposal 0071.

## Evidence

Four key tests cover 13 native integer-domain rows, wide u64 keys and exact checked
string bytes. Thirteen managed-value integration tests cover ownership, foreign/
stale handles, exact resource/work/scratch boundaries, mixed collection, 70,000
simultaneous data nodes, 70,000 aliases, 100,000 churn cycles within two parent/value
slots, and 16,384 operations checked against an independent integer-label model.
Three private tests check generation/counter exhaustion and wrong-kind API/VM tags.

Four zero-key states from the 20 native cycle-alias traces replay empty detachment
using explicit host-owned aliases. Those aliases reproduce observed numeric counts,
not retained-cycle topology. The other 16 populated states remain helper-only for
array execution. Existing 306 array traces and native source guards stay unchanged.

The runnable array_value_workload reports logical storage, work, scratch and local
elapsed time. It excludes native VM/GC, RSS, frame latency and stock-mod conformance.
Local Windows debug observation: 70,000 owners plus one detachment retained
2,800,096 logical metadata bytes and charged 341,069 copy/detach work steps.
The 100,000-cycle churn retained 176 logical metadata bytes, two parent/value
slots, charged 2,500,001 copy/detach/release/collection steps and peaked at 18
scratch bytes. Observed elapsed times were 54,852 and 553,878 microseconds;
these are individual component runs, not performance or native-parity guarantees.
Windows workspace debug/release, Clippy, rustdoc and formatting passed, including
268 frontend and 206 legacy tests, nine script unit tests and two doctests.
Repository policy checked 804 source files/eight crates; largest source remains
287 lines. Native array 306/alias 20 traces, selected alias/array/write guards
(14/69/141 mutations) and 99 syntax replays passed. Linux/hosted CI remains unrun locally.

## Remaining gate

Complete typed indexed storage/order, recursive populated COW, checked deletion,
mixed graph/string roots, retained-cycle marker publication, ownership transfer and
bounded failure cleanup across frames/fields/returns/suspension before source arrays.
Proposal 0071 remains a proposal for those parts. Full GSC/S5 remains open.
