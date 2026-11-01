# ADR 0040: Scalar for phases and continue initialization joins

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

Scalar while transfers cannot be reused as a simple textual rewrite of for:
continue must reach the update before reevaluating the condition. Initialization
at that update must include every continuing body path, excluding break/return.
The pinned compiler patches continue sites before emitting the update and merges
its continue analysis blocks. Its fixed arrays/short offsets are not Rust limits.

## Decision

Extend the existing scalar-local profile with flat staged for lowering. The
initializer runs once, then condition/body/update/backward condition jump. Lower
optional condition productions 26/27 and update productions 95/96 explicitly;
absent conditions emit numeric true and absent updates emit no source operation.
Initializer/update clauses accept only scalar plain-name assignment, compound,
postfix or empty forms. Other clause statements fail explicitly under this
restricted profile; no initializer/update transfer or host authority is inferred.

Generalize the private active loop context for actual while/for use. While
continue retains a known condition destination. For continue owns a deferred
instruction-record fixup chain plus an optional admitted initialization snapshot.
Each continue contributes reaching facts; merge them with body fallthrough before
checking the update. Exclude break/return paths. Bind continues to the actual
update start, or the real backward jump when update is empty; bind breaks to the
exit. All chains remain single-use, monotonic, flat and globally work-charged.

Snapshot capture/merge/release uses existing control/metadata/work admission;
context ownership stays inside admitted end tasks. No extra unaccounted fixup
array or recursive IR is introduced. Restore post-initializer incoming facts at
the exit, conservatively allowing zero iterations even for omitted/constant true
conditions. A syntactically present update with no reaching body/continue path is
still checked against incoming facts; its code remains unreachable. This explicit
restricted diagnostic policy may reject body-only names in unreachable updates
and does not establish native constant folding/scope/diagnostic-order parity.

Keep existing numeric/lazy lowering, wide slots/targets, original clause/keyword
maps, fresh arguments and verified bounded execution. The pure-return profile
stays unchanged. Function-local slot allocation is deterministic in lowering
order; no native local-slot ordering or serialized bytecode ABI is promised.

## Evidence required

Guard selected grammar optional roles and compiler initializer/body/continue-fixup/
update/backward roles against pinned source without executing native routines.
Test all omitted-clause combinations, once-only initialization, reevaluation,
continue-before-update, nested mixed loops, break/return exits, update initialization
joins, unreachable-update policy, invalid clauses/reads, keyword/operator maps,
fuel and caller isolation. Cover snapshot/context/fixup quotas, wide destinations,
deep flat cleanup and unchanged existing profiles. Complete native compiler/VM,
watchdog/scopes/constant folding, switch, richer values/calls and stock-mod
acceptance remain open.
