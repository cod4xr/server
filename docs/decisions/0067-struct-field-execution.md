# ADR 0067: Bound-source ordinary struct fields

Date: 2026-11-01

Status: implemented for ordinary struct reads, simple assignment and deletion.

## Problem and decision

An empty shared struct cannot retain script state without source field operations.
Lower ordinary field reads and simple assignments in the bound-module profile to
checked LoadField/StoreField instructions. Preserve shared identity and evaluate
the RHS before the receiver, including nested expressions that replace an alias.
Missing fields return Undefined. Both literal and computed Undefined assignments
remove a field; an Undefined receiver raises a located error rather than creating
a struct. Compound field assignments and postfix updates remain rejected.

Own immutable field-name tables separately from string literal tables. Field names
need no runtime interning, string allocation or literal-cache slots. Typed tables
retain exact binary bytes; source names use the existing guarded ASCII legacy
canonicalization, including exact case at lengths of 8192 bytes or more. Names
and operands use wide indices, never native canonical-string/packed pool widths.
Repeated source occurrences retain separate admitted rows; interning is deferred
until measurements justify its lookup/work/storage costs.

The verifier copies and admits all rows/payloads, including unused ones, checks
unreachable operands and shares complete proof work/metadata across functions.
Compiler accounting includes simultaneous string/name staging and proof copies.
Each function owns its field-name namespace. Add a descriptor-based standalone
verification entry point without inventing a bytecode ABI or implicit module.

Heap dispatch now distinguishes an unhandled instruction from an applied operation
that produces zero or one value. Check the declared produced stack count, and
collapse operands only after successful heap publication. Field writes retain
receiver/RHS and complete caller/cache/external graphs through pressure collection.
Retry live-field, key-byte, field-slot, index-bucket or metadata pressure once after
a complete object/string collection pair; work, scratch, single-key and reference
errors do not retry. Existing operations remain individually atomic; a completed
collection or earlier instruction may persist after invocation failure.

## Evidence and limits

The field guard rejects 99 scoped changes to selected grammar/compiler/VM/helper
roles. Existing assignment guards retain RHS-before-lvalue and computed-Undefined
deletion evidence. Actual extracted C field get/index/set/clear helpers execute
35 traces at O0/O2 with bounded modeled lookup/reference/removal services. Rust
replays 27 value/read/missing/delete cases; eight native function words stay
helper-only. Rust function references and vector fields have separate ownership/
source tests. Native vector pointers/entities are excluded, and native refcounts,
collision relocation, allocation/freeing, pointer ABI, unwinding and whole VM
execution are not established by this corpus. The raw C setter can retain a
stored Undefined; source/VM assignment deletes it. General native assignment can
create a temporary child before deleting it, which this Rust operation skips.
Native temporary allocation/error-cleanup parity is not claimed.

Tests cover binary owned tables, absent versus stored Undefined, source-drop
ownership, per-function namespaces, void-stack proof, exact/one-less compiler and
proof budgets, all failed pressure-work prefixes, exact scratch, each independent
memory pressure resource, and operand-only receiver/RHS roots with nested strings.
Source tests retain 70,000 distinct live fields without runtime strings, preserve
8191/8192/100000-byte name behavior, execute 100,000 cyclic updates in four object
slots and construct/reclaim a 12,000-edge graph iteratively. These are capacity and
ownership tests, not native capacity limits or compiler/VM conformance claims.

The offline CLI logs field-name counts/staged bytes and existing object metrics;
field names, values, IDs and private paths stay out of operational events. Process
tests check real results and private failures. The field_demo.gsc command returns
integer 71 by preserving an old alias while the RHS replaces a nested receiver.

No new crate, dependency, native runtime linkage or target directory is introduced.
Array COW, general object operators, methods/self, entity lifetimes, scheduling,
imports, game services and full stock/mod acceptance remain separate work. This
does not close the S5 acceptance gate or make the server joinable.

Windows verification passed workspace debug/release tests, warnings-denied
Clippy/rustdoc, formatting, diff checks and repository policy (744 source files,
eight crates, largest 287 lines). The script suites contain 257 frontend and 178
legacy tests, six private units and two doctests. Both new process tests pass;
35 native helper traces, 99 field-role mutations and 99 structural replays
regenerate. Linux/hosted CI and native game interoperability remain unverified.

```powershell
cargo run -p cod4x-server --locked -- --execute-gsc examples/gsc/field_demo.gsc --color never
```
