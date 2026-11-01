# Assessment 0096: Selected native string-switch case emission

Date: 2026-11-01

Status: selected Windows evidence accepted. No Rust runtime or
source string-switch activation; complete Rust workspace acceptance remains 0095.

Four additional scenarios extend the existing executed switch oracle. They use
synthetic plain-string AST tags and bounded opaque string identities with the
actual pinned EmitSwitchStatement, CompareCaseInfo and selected scope/slot/transfer
instructions. Cases cover one string/default, mixed integer/string labels, and
two string identities declared in either order. Independent block states,
opcode requests, serialized key words and modeled user-string transfer records
agree at C driver O0/O2. All previous eleven scenario rows are unchanged, as are
the other four scope fixture families.

Integer 1 encodes to 0x800001 while modeled string identity 1 stays 1. The actual
comparator sorts unsigned key words descending, with the zero default sentinel
last. Reversing string declaration order changes modeled transfer order without
changing the sorted case keys. This establishes no string-byte ordering: the
identities are supplied by the model, not obtained from a native interner.

The bounded modeled qsort continues to call the actual selected comparator. It
now records completed key words using byte copies, without unaligned C loads.
SL_TransferRefToUser validates the selected ID/user profile and records calls;
it does not execute native string ownership, interning, release or reload. Earlier
integer scenarios explicitly reject any newly introduced string transfer.
Condition/body dispatch, opcode output, allocation and integer-key encoding
services remain modeled. No parser, whole compiler, VM dispatch, value freeing,
native sorting/allocator/alignment/optimization, error unwinding or developer-mode
execution claim follows from these scenarios.

The selected source-role guard additionally checks plain-string case payload and
compiler-mode branches, transfer setup, the comparator's unsigned descending
direction, and the VM string-key path that saves the identity then releases the
operand before searching. These are source-role checks, not executed VM lifetime
evidence. The guard rejects 59 mutations, up from 39. Three new admission tests
reject integer/string identity collapse, incorrect key/source order, missing
default sentinels and missing/duplicate transfers. Five scope admission suites
now contain 38 tests, including twelve switch-family tests.

Required tools are all five native scope generators, 38 admission tests, 59
switch/98 local-removal mutations, 99 structural replays, suite registration,
repository policy and diff checks. No Rust source, dependency, crate, target
directory or earlier test identity changes. Linux/hosted CI remain unrun.

All five generators, 38 admission tests, 59/98 mutations, 99 structural replays,
twelve suite-policy regressions and policy/diff checks pass on Windows. Policy
still checks 1054 sources/eight crates/largest 287 lines. Rust code is unchanged;
no additional Rust workspace run was needed beyond accepted ADR 0095.

Next, design checked string/mixed switch-key dispatch using complete owned label
bytes and explicit heap access, without embedding native string IDs in immutable
code or treating integer and string labels as interchangeable. Keep numeric
profiles usable without fabricated heaps. Duplicate labels, invalid key types,
default/fallthrough, decoded byte identity, private locations, cleanup, exact
budgets, large labels/counts and package/CLI paths need separate Rust acceptance.
