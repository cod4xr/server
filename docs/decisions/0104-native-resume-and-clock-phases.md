# Assessment 0104: Native resume queues and script-clock phases

Date: 2026-11-01

Status: selected native reference cases pass on Windows. Rust source waits,
continuations, thread/event scheduling and the S5 acceptance gate remain open.

Two independent executable families establish requirements for resumable execution.
Both read only the hash-pinned upstream snapshot, build temporary tools outside
Cargo, compare driver O0/O2 output and admit complete traces against independent
Python expectations. They neither load a game nor replace stock/mod acceptance.

The first executes actual x86 `VM_Resume` in ten cases. It removes the current
queue entry, restores its archived scalar values and program/local identifiers,
clears its wait state, requests archive release, then invokes the modeled body.
The body can insert more entries into the modeled list. Native resume re-reads
the list head after each body, rather than iterating a once-captured inventory.
Finite self-requeue, modeled front/tail insertions, interleaved new work, empty
queues, zero/one/three archived values, Undefined payload words and timestamp
matches/mismatches are distinct cases. A mismatched archived time byte refreshes
the watchdog clock. The time bucket is cleared after the queue drains.

The native function's queue/global/service boundary is explicit and fail closed.
The existing freestanding scope runner now accepts a selected pinned source name
and explicitly modeled native BSS symbols; all old defaults stay unchanged.
Modeled symbols must be both referenced and declared as native zero-initialized
storage, and are excluded from copied data definitions. Five admission tests guard
the selection/service/storage boundary. All seven earlier scope generators still
reproduce their accepted fixtures byte for byte.

List lookup, initial queue order and subsequent front/tail insertion are modeled
services: these cases do **not** establish which native insertion function puts
work at either end. Archives are synthetic scalar buffers, not `VM_ArchiveStack`
results. Local-variable inventories are empty. `VM_ExecuteInternal`, general
archived call frames, reference ownership/freeing and error unwinding do not
execute. Asserted release calls are not a proof of native heap lifetimes.

The second family executes the copied C `VM_SetTime`, `Scr_RunCurrentThreads`
and `Scr_IncTime` bodies in thirty cases. Their observed field names, cvars,
lookup, clock, logging, `VM_Resume` and entity-cleanup services are modeled; this
family does not use or claim native structure ABI layouts. Cases cross disabled
scripts, absent time arrays, missing buckets, present/logged buckets, ordinary
time and the native mask boundary with run, increment and run-then-increment.

`Scr_IncTime` first invokes current-thread processing, then entity cleanup, then
advances time. Disabling scripts suppresses resume/performance accounting but
does not suppress entity cleanup or clock advancement. Running current threads
alone leaves time unchanged. A second pass observes an already-removed bucket.
The native 24-bit wrap is recorded evidence, not an internal Rust clock limit.

Source inspection separately connects wait opcode 0x4c to the numeric conversion
helper and current-time-plus-delay bucket; exact zero is not clamped to one.
Frame-end opcode 0x4d selects the current bucket through the distinct reverse
insertion service. Those opcode paths and their insertion implementations are
not executed by either family. Do not claim complete `wait 0`/frame-end ordering
from modeled queue direction or collapse the two source constructs.

Nine trace-admission tests per family reject incomplete/duplicate/unknown cases,
wrong phase/order/scalar restoration, lost requeued work, premature bucket clears,
changed release requests, disabled-script behavior, logging/performance counts and
incorrect wrap observations. CI now runs all 23 new admission/boundary tests and
regenerates both families. Local checks pass; hosted CI/Linux remain unrun.
Rust behavior is unchanged from ADR 0103's complete Windows gates (428 frontend,
273 legacy, 22 private script tests). No full-GSC workstream closes here.

The next component must own flat frames and a verified module lifetime across
suspension, retain cumulative fuel/admission, keep every suspended root alive,
and release prepaid array owners exactly once on return/error/cancellation/drop.
An exclusive borrowed heap can support one suspended invocation; interleaving
threads will require a scheduler-wide root inventory and ownership review.
