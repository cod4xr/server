# ADR 0059: Localized string values without implicit translation

Date: 2026-11-01

Status: accepted for localized values, equality and raw offline output.

## Semantic assessment and decision

The pinned source distinguishes VAR_STRING (2) from VAR_ISTRING (3). The existing
literal decoder preserves this category. Scr_EvalEquality/inequality support
same-type interned string identity; valid canonical byte contents determine the
owned-store equivalent. Scr_CastWeakerPair rejects plain/localized mixed types,
Scr_EvalPlus rejects localized operands and Scr_CastBool accepts numbers only.
GScr_IsString returns false for localized values. Keep these distinctions.

Add an opaque LocalizedString value and a separately checked localized literal
instruction, using the same owned byte store and per-row invocation cache. One
table row may be loaded with either explicit tag without coupling immutable code
to one heap. Reject runtime localized references as code constants, including
unreachable code. Validate and root both string categories through arguments,
stacks, locals, caches and external roots; copied IDs remain nonowning handles.

Bound source adopts &-quoted decoded bytes under the existing exact encoded/
decoded/compiler/module/proof budgets, preserving original spans. Support copied
values, calls/returns and same-type equality only. No localization catalog lookup,
translation/interpolation, game/HUD service or client-visible capacity is implied.
Offline output uses a distinct escaped localized_string byte category; operational
logs expose only fixed types/counts. Existing isolated scalar profiles stay explicit.

## Validation

Require native predicate tag replays, source decoder fixtures and source-grounded
comparison/operator roles, checked/stale/foreign/copy/rooting behavior for localized
handles, pressure collection with cached/localized-only roots, mixed-type rejection,
same-row cache tags, source ownership/maps/privacy, exact resource admission and
real process output/errors. Full native interning/ref cleanup/compiler/VM, actual
localization services, objects/scheduling/mod and platform acceptance stay open.

## Implemented evidence

Actual extracted comparison/arithmetic functions now produce 3,024 O0/O2 outcomes
with canonical synthetic string labels: 144 equality/inequality results and 2,880
type errors. Rust replays every row through checked arguments and owned handles.
The opt-in oracle service checks both reference callback labels/order/counts; all
other string services remain aborting. Default vector/Undefined/function/index
oracles retain their original strict services and byte-identical fixtures. These
callbacks model identities, not native byte allocation/interning/ref cleanup.

The predicate corpus now replays 196 rows (28 results/168 arity failures) out of
832 actual C cases; 636 remain helper-only. Localized tag 3 is a real owned Rust
value and isstring correctly returns false. Six kernel tests cover copies, same-row
different tags, uncanonicalized content equality, invalid operands/constants,
foreign/stale handles and isolated caller/cache/external roots under pressure.
Six source tests cover ownership, exact prefix/decoded quotas, calls/equality/type
rules, error privacy/maps, 998 loop pressure collections in six slots and 70,000
mixed plain/localized literal rows. Two process tests cover distinct escaped
output and private located failures. The localized_demo.gsc example returns
localized_string b"MENU_EXAMPLE", without loading a translation catalog.

A separate 1 MiB alias test proves validated identical handles compare in six
logical work units including argument checks, for either category; distinct handles
still require budgeted content visits. This is constant-work evidence for immutable
aliases, not a native interner or comparative timing/RSS claim. All previous full
workspace and independent fixture gates remain required.
