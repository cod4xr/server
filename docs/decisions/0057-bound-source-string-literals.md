# ADR 0057: Bound source plain-string literals

Date: 2026-10-31

Status: accepted for bound plain-string literals and offline byte output.

## Decision

Extend bound-module lowering with plain quoted string literals backed by the
implemented immutable tables/cache. Preserve the isolated scalar source profiles.
Use the existing literal decoder and exact shared decoded-length preflight: admit
row/payload metadata and work before fallible row reservation and payload decoding.
Adopt owned bytes without another copy; emit a checked StringLiteral instruction
with the original primary token span. No runtime heap ID belongs to compiled code.

Charge encoded/decoded bytes and possible literal-row relocation work. Include
literal records/payload throughout pending/resolved/staged compiler overlap. Module
prefixes retain prior staged literal storage while lowering later functions, and
separate kernel proof budgets admit their own code/table/payload copies. These
logical scopes exclude source/CST, allocator rounding/spare capacity and RSS; no
aggregate allocator measurement is claimed. Publish only complete bound modules.

Unused/disabled developer code follows existing inclusion policy; reject localized
literals, mixed coercion, string switches/indexing and unknown services explicitly.
Supplied/constructed plain strings use existing addition/equality and global
heap/frame/cache budgets, without native pool widths or a global interner.

## Validation

Require escaped/non-UTF-8/empty/large literals after input drop, real calls/returns/
locals/loops under pressure, comparison/predicate behavior, literal/operator source
maps/privacy, exact-one-less token/compiler/module/proof/heap resources, atomic
failure on unsupported bodies and developer inclusion. Existing scalar/native
fixture and full workspace gates remain required. CLI must separately acquire an
explicit persistent heap and safely escape byte output before claiming string
execution there. Native compiler/interner/ref cleanup, localized/mixed coercion,
full objects/scheduling/game services and stock/mod/platform acceptance stay open.

## Implemented evidence and offline ownership

Nine source tests cover escaped/non-UTF-8/empty payloads, ownership after input
drop, direct/indirect calls, loops, content equality/type predicates, cached roots,
literal/operator locations, developer inclusion, exact-one-less decoded/encoded/
compiler/module/proof resources, 100,000-byte payloads and 70,000 distinct literal
rows. A 1,000-iteration source loop remains within five slots/six payload bytes and
performs 998 pressure collections. These are capacity/lifetime tests, not native
compiler/interner or comparative performance evidence. Localized bodies explicitly
remain unsupported; isolated scalar source profiles retain their prior contract.

The offline application now creates one caller-owned heap/context and keeps it
alive through result resolution/output. ScriptResult resolves checked bytes into
a borrowed presentation value without Debug or another payload copy; stdout uses
escaped ASCII byte syntax and propagates write/flush errors. Logs expose only
fixed result categories and compiler/frame/heap/cache counts, including failed
heap work. Source paths, names and payloads stay out of operational logs.

Three real process tests cover JSON/human privacy, escaped control/non-UTF-8 bytes,
located localized/mixed failures and complete 100,000-byte stdout. A private output
test guards quote/control/NUL/non-UTF-8 escaping; existing write/flush failures stay
required. The strings_demo.gsc example prints string b"[[[Rust]]]". One manual
Windows run used 23 instructions of compiled code, 61 executed instructions and
113 heap work units. These are local observations, not full GSC/mod acceptance.
