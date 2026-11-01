# ADR 0103: Reuse an array-owner audit within an exclusive VM context

Date: 2026-11-01

Status: bounded VM collector optimization accepted on Windows.

The developer-block stress profile exposed repeated collection work proportional
to all retained managed array-value slots. Object tracing currently enumerates
every location twice, then array preflight checks every occupied location again.
70,001 aliases of one host array therefore dominate repeated collection even when
only two temporary data slots churn. The shared work budget correctly stops this
cost, but increasing the budget does not improve the algorithm.

Public ObjectHeap::reclaim and cycle analysis keep their existing validation
paths. The VM uses a sealed witness tied to one object-store identity and the
KernelHeapContext's exclusive mutable borrow. The first successful object
collection executes the complete existing owner-location audit. Only that
operation mints the witness; cached collections retain it. Rebinding object
storage discards it. A foreign witness forces full validation. The witness's
private field and minting operation are confined to the reclamation module;
there is no public constructor, serialization, Clone or mutable-heap escape.

ObjectHeap enters this path through its checked constructor and public mutation
APIs; it has no raw-state import. Their ownership relation is constructive: a new
location/data pair starts at one owner, retaining adds one location and owner,
releasing invalidates that location and decrements its data, and COW moves one
location while transferring one owner between data records. All fallible work
precedes those publication steps. array_allocation, array_values, ArrayValuePlan,
empty COW and CopyPlan publication were reviewed against that relation, including
failed admission and exhausted prepaid cleanup. The existing full scan validates
every location/reference and aggregate owner consistency; it is not a repair
mechanism for arbitrary privately forged stores. Per-data ownership is maintained
by those checked mutation paths. New mutators, raw imports, nested ownership or
host mutable escapes require another invariant review or witness invalidation.

After a valid witness, collection enumerates owned data records rather than all
alias locations. It still checks current data generations, complete caller/cache/
world roots, graph fields, owner-count sums, global counters, marks, limits and
sweep atomicity. Only the stable location invariant is reused. No array edge,
logical reference-count rule or nested authority is added. A completed object
collection/witness can persist when subsequent string collection fails; checked
repair and later cached collection work on that same context. Runtime success/
failure logs expose array_owner_audits and array_owner_audit_reuses as counts,
without identities or payloads. Counter overflow checks precede collection.

The original stress now succeeds: 100,000 private constructor iterations beside
70,001 host locations fit the unchanged default work budget and 4 KiB scratch,
using three data/object slots, one full audit and at least 49,998 reuses. Every
host owner/payload remains unchanged and no VM owner escapes. A second test keeps
host-only struct cycles and uncached strings alive across hundreds of cached
collections, then reclaims them after the last host root is released. Context
reuse/rebinding and object-success/string-failure/checked-repair are tested.

Five private tests cover public full-audit cost and fault checks, every shorter
initial/cached collection prefix, one-less cached scratch, unchanged failed state,
same-store retry, foreign witnesses and checked allocation/copy/empty/populated
COW/release after an empty-store audit. The earlier 64,000-step first-audit failure
test remains unchanged. These are domain-specific work guarantees; collection
still scales with reachable graphs and object/field/index inventories.

Debug workspace passes 427 frontend/273 legacy/22 private script tests; the latest
complete frontend suite passes 428 after the additional graph test. The updated
real-process suite and all-target Clippy pass. Native checks retain 306 primitive,
20 cycle-alias/COW, 210 lifetime/COW and 100 selected read traces at driver O0/O2;
141 write/61 lifetime/39 constructor mutations reject. They are selected modeled-
service helper evidence, not native whole-GC execution. Documentation/release/
final inventory gates pass. Complete release workspace passes 428 frontend/273
legacy/22 private script tests and two script doctests; all original test names
remain. Formatting, denied-warning rustdoc, twelve suite-policy regressions and
repository/diff checks pass (1110 sources/eight crates/largest 287 lines).
Linux/hosted CI remain unrun. No dependency,
crate, Cargo test binary or additional target directory is introduced.
