# ADR 0019: Legacy name identity and bounded local function binding

Date: 2026-10-31

Status: accepted and implemented; full-workspace validation recorded separately.

## Context

Typed declarations retain raw names and developer markers but do not resolve local
definitions. The source lowercases identifiers through native string/ref services,
then registers functions according to developer_script mode. Reusing native IDs,
global pools, and scratch buffers would constrain the new compiler/runtime.

## Decision

Provide borrowed LegacyIdentifier textual keys under an explicit ASCII/C-locale
profile. Equality/hash use the actual conversion helper's rules: names shorter
than 8192 bytes fold ASCII case; longer names retain exact case because native
SL_ConvertToLowercase bypasses its scratch buffer. No name is truncated/rejected
by that compatibility branch, and no fixed buffer is allocated. This quirk is
isolated in a named legacy type; modern naming/runtime string storage is separate.
Non-ASCII/ambient-locale native behavior is outside this profile.

Build LegacyModuleSymbols above immutable declarations. Retain all definitions in
source order with ordinary/developer-only visibility and active/developer-disabled
inclusion. An explicit LegacyDeveloperMode controls active lookup and duplicate
checking. Disabled developer functions remain recorded, even when their names
overlap ordinary definitions. Enabled developer functions enter the local index
with visibility retained; caller/link visibility checks are not implemented here.

Reject duplicate active canonical names, nested developer regions, unmatched ends,
and unclosed begins. Reject animation-tree directives inside developer regions
after definition/region checks, following the native compiler's phase ordering.
Keep current/earlier offsets without names. Preserve full directive spans so a
placement diagnostic points at its keyword. Complete native error precedence,
recovery, import resolution, parameter/local scopes, builtins, calls, body lowering,
bytecode, and execution remain separate work.

Preflight counts, borrowed name bytes, per-name size, logical work, and record/map
storage before fallible allocation or name scanning/hashing. Preallocate vectors
and the active map, then bind privately and return only a complete index. Logical
work counts two item visits plus admitted name bytes, not exact CPU instructions.
Metadata counts all definition records and active key/value entries, excluding
source/tree/declarations, spare capacity, hash controls, and allocator overhead.
Use the standard randomized map hasher; source order comes from the vector.
Internal indices are native-wide/private; public results are scoped borrowed
definition views rather than forgeable or cross-module runtime IDs.

Lookup allocates no data and uses canonical textual identity. Queries longer than
every active indexed name return None before hashing. Hash work is otherwise
linear in the query size. No module or engine state is activated by binding.

The offline inspector binds with developer mode disabled by default; the explicit
--gsc-developer enabled/disabled option is valid only with --inspect-gsc. It reports
total/active/visibility counts and separate logical work/storage. Binding errors
include current/previous line and byte-column locations without script names/paths.
Success is not a compilation or execution result.

## Alternatives and migration

Global interning during scanning would publish premature mutable state and retain
old pool/ID assumptions. Unconditionally folding long names would change the
recorded helper's behavior. Dropping inactive declarations would discard source
metadata needed by tooling/reload. Copying canonical strings would add avoidable
payload ownership before a runtime interner exists. Borrowed keys/views preserve
source ownership and keep binding independent of future heap/GC design.

## Evidence and limits

An original-function GCC driver executes SL_ConvertToLowercase under C locale with
native string/ref services stubbed. Its 125 ASCII cases include scratch boundaries
and 100,000-byte names, comparing canonical output checksums and normalization
branch behavior. This is the tenth original-function generator, not an original
interner/compiler execution test. Rust tests additionally check exact small keys,
equality/hash consistency across sources/offsets, long-name identity, and privacy.

Seven symbol tests cover explicit modes, retained inactive definitions, active
duplicates, region/directive phase rules and offsets, empty modules, all five
exact/one-less budgets, 66,000 indexed definitions, and long case-distinct names.
Four new process tests cover mode selection, counts, failure locations, and log
privacy. Hash-pinned compiler assembly/header plus mechanical branch/profile
checks ground the interpreted rules; they do not execute the native compiler.
Full compiler/import/link/stock-mod, modern language/services, and Linux acceptance
remain open. Frontend capacities are not executable function/argument guarantees.
