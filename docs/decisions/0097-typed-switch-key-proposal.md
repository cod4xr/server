# ADR 0097: Typed switch keys and immutable label tables

Date: 2026-11-01

Status: typed component accepted on Windows. Source string switches remain
rejected pending a separate compiler/dispatch acceptance.

Assessment 0096 proves selected native plain-string/mixed case identities and
emission roles, with string services modeled. Native global string IDs must not
become runtime handles embedded in Rust code. Function definitions/proofs now
carry a distinct immutable label-byte table, with complete global/per-function
work and metadata admission. It stays separate from string-literal caches:
labels reserve no invocation cache slots and allocate no runtime strings merely
to be compared. Module-table accounting moves to a small dedicated helper.

Typed data comprises an integer key or a wide code-label operand, and
sealed equality/less-than comparisons. RequireSwitchKey admits only int32/plain
strings, validating string handles through an explicit heap. SwitchCompare
compares a stack key against immutable code data without cloning an array or
allocating/collecting/yielding. Both have unary one-to-one stack effects, and
failures leave operands available to prepaid frame cleanup.

Use an explicit internal normalized order: integers before strings, signed int32
ordering within integers, complete byte ordering within plain strings. Integer
and string categories never compare equal. This is an internal search contract,
not native global-ID serialization or ordering. Preserve existing full-int32
capacity expansion; no native string/short case-count/local limit is introduced.

Complete proof must check label operands even in unreachable code and own all
label bytes after input drop. Metadata/work counts include unused labels. Numeric
keys remain usable without a fabricated heap, even when code contains string
labels. String access prepays validation/byte visits; foreign/stale handles fail
with private categories. Localized strings, floats, Undefined, vectors, objects,
functions and Array keys remain outside the switch-key domain. Existing array
owners must be released on every rejected-key/work/instruction prefix.

Sixteen integration tests cover key/value/category matrices, label/input ownership,
empty/prefix/case/NUL/high-byte identities, invalid/unreachable wide operands,
exact/every-shorter global/function verification and runtime work/setup/instruction
budgets, one-less resources and repeated same-context recovery. Rejected Array
keys release every VM owner at all shorter work/instruction/setup prefixes. A
runtime key read from the last independent array survives paired collection before
matching, without a fourth string allocation for the label. Pressure work/scratch
prefixes and private errors are checked.

Wide tests cover 70,000 labels, a 70,000-byte key and operands above short limits.
Integer keys execute with zero string slots and zero scratch even when code owns
70,000 label rows. Plain-string comparisons use one preexisting input slot and
zero scratch. Per-function label operands remain local across calls, independent
of literal-cache offsets. 100,000 comparisons beside 70,001 host owners use less
than one million heap steps and fit 4 KiB scratch without collection/history scans.

KernelFunctionDefinition gains a separate borrowed switch_labels table; proofs
copy/own every row and payload. Program/module usage reports label counts and
retained bytes separately from literal/field tables. Table rows, payload/copy work,
global retained prefixes and current verifier overlap pay existing metadata/work
budgets, including unused labels. Operand-catalog counts use a small typed struct
instead of extending an eight-argument verifier helper. These are internal typed
Rust API/metadata changes, with no serialized/native bytecode or game-wire ABI.

Full Windows debug/release workspace passes 390 frontend/273 legacy tests,
fifteen private script tests and two script doctests. All earlier test identities
remain. All-target denied-warning Clippy, denied-warning rustdoc, formatting,
five native scope generators, 38 admission tests, 59/98 mutations, 99 structural
replays, twelve suite-policy regressions and policy/diff checks pass. Policy
checks 1067 sources/eight crates/largest 287 lines. Linux/hosted CI remain unrun.
No new dependency, crate or target directory.

Source activation is a subsequent review: decoded plain labels, deterministic
duplicate handling, metered mixed-key sorting/search, numeric-only standalone
profiles, empty/default/fallthrough, private locations, package/CLI behavior,
wide/deep budgets and context cleanup all require acceptance. No language or
heap authority is added merely by staging the typed instructions/table.
