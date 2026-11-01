# ADR 0098: Bound-source string and mixed switches

Date: 2026-11-01

Status: bounded source component accepted on Windows.

Bound modules and supplied packages lower integer/plain-string switch keys using
ADR 0097's checked typed instructions and immutable label table. Standalone
PrimitiveStatements retains integer-only labels and RequireInteger. Bound source
uses RequireSwitchKey even for empty/default-only/integer-only switches. Wrong
key categories now report ExpectedSwitchKey in that profile. Strings require an
explicit checked heap; numeric keys still execute without one, including mixed
case inventories. No heap string is allocated merely to represent a case label.

Only unchanged plain-string literal case productions activate. Localized strings,
booleans, floats, Undefined and grouped labels remain unsupported; binary case
expressions fail the pinned grammar before lowering. Labels decode under the
existing encoded/decoded per-token quotas. Preflight and both decoding passes
prepay work; complete descriptor/payload admission precedes retention. Compiler
function/module usage gains separate switch-label counts and retained/staged
bytes. Package prefixes, current-function overlap and separately owned kernel
proof copies include those bytes. Labels consume no runtime literal-cache slots.

Cases sort through an iterative, fallible in-place heapsort. Each visit, swap,
comparison and complete minimum string-prefix byte allowance pays compiler work.
The normalized order is signed integers before complete plain-string bytes.
This differs from native unsigned descending global-ID serialization; it is an
internal lookup contract with distinct domains, not a native bytecode ABI. Equal
decoded keys sort by source position before the duplicate check, so separately
retained identical strings report the second source case deterministically.

Dispatch remains balanced with checked addresses and shrinking child ranges.
Bound nodes occupy nine instructions, using unary comparisons against immutable
rows. Standalone numeric nodes keep their eleven-instruction footprint. This
changes an existing bound indirect-call fixture's execution count from 19 to 18;
its value, single key evaluation and frame assertions remain unchanged. The old
bound unsupported-string test identity now uses a localized label. All original
test identities remain; the two intentional profile changes are explicit.

Physical scopes and initialization rules remain those of ADR 0094. Source-order
case bodies retain adjacent labels, default placement, fallthrough cleanup and
distinct nearest-break/outer-continue ownership. Matching does not clone array
receivers, collect, yield or retain new managed owners. Failed type/work/setup/
instruction admission releases every VM array owner through prepaid cleanup.

Seventeen source tests cover separate domains, empty/case-sensitive/escaped labels,
decoded duplicates and private locations, standalone gates, default-only/empty
dispatch, fallthrough/loop transfers and observable array COW order. An independent
direct-lookup model checks all keys and misses across 256 permuted mixed cases,
including failed-call recovery on the same heap context. Compiler decoding,
function/global staging/sorting/proof overlap, all shorter compiler-work prefixes
and independent one-less limits are tested. Long common-prefix labels pay byte
comparison work in addition to decoding.

Wide tests compile 66,000 source string cases and use logarithmic dispatch with
one input string slot and zero cache scratch; 70,001-byte labels retain complete
payload after source drop. Supplied includes/direct/indirect calls retain separate
per-function label-zero identities and literal-cache offsets, with exact and
one-less package admission. A runtime string read from the last populated array
survives callee-triggered paired collection while rooted only on a caller stack.
That source test covers every shorter heap/instruction/setup prefix and one-less
scratch. 100,000 source dispatches fit 4 KiB scratch beside 70,001 host owners,
without label allocation, collection or scanning unrelated owner history.

Three real-process tests cover the runnable string_switches.gsc example (integer
42), supplied include packages, pretty/JSON logs, separate numeric label counters,
duplicate-case locations and rejected Array-key cleanup. Logs disclose neither
labels, source paths nor function spelling. The explicit result remains on stdout.

Complete Windows debug/release workspace passes 407 frontend/273 legacy tests,
fifteen private script tests, two script doctests and all three new process tests.
All-target denied-warning Clippy, denied-warning rustdoc, formatting/diff/policy
and prior test inventories pass. Five pinned native scope generators, 38 admissions,
59 switch/98 local mutations, 99 structural replays and twelve suite-policy
regressions pass. This source change adds twelve files to the 1067-source baseline;
eight crates and the largest source (287 lines) remain unchanged. Linux/hosted CI
remain unrun. No new dependency, crate, Cargo test binary or target directory.

This accepts no general logical reference counting, stored/nested Array edges,
root/game array writes, indexed compound/postfix, entity/thread methods, scheduler,
engine builtin library, developer-statement execution or complete GSC/mod
compatibility. No whole legacy workstream closes with this source subset.
