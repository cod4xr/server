# ADR 0039: Scoped scalar while break and continue

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

Scalar while currently rejects break/continue. These transfers must belong to the
nearest active loop, terminate their current structural path, and preserve nested
conditionals' initialization joins. Native compiler contexts carry separate linked
break/continue fixups and fixed-size analysis arrays; neither array ceilings nor
native variable-stack cleanup should be copied into a function-wide scalar layout.

## Decision

Extend only the existing scalar-local while profile with break and continue.
Keep an active private loop context while lowering its body; save/restore an outer
context in the already admitted flat end task. Continue targets the current
loop's real condition head. Break targets a checked forward loop exit. Outside
an active loop, return located fixed illegal-transfer errors. Both transfers
terminate the current structural path, so later source is rejected and such paths
do not constrain conditional joins. Conservative zero-iteration loop exit facts
remain incoming initialization; do not infer body-only assignment after a break.

Represent break fixups as a compiler-owned chain linked inside pending instruction
records, with a single-use relocation token per link. This avoids an extra dynamic
fixup array or unaccounted allocation. Existing instruction/metadata admission
includes the complete pending record size. Resolve each chain iteratively once,
charge every visited fixup globally, and check monotonic links/real forward targets.
No unresolved link or compiler-private record may enter a verified program.
Runtime jumps share existing verification, instruction fuel and original keyword
maps. Keep emission/admission and relocation resolution in separate leaf modules
as the builder grows. Ownership/drop remains flat at arbitrary admitted nesting.

## Evidence required

Guard selected compiler dispatch/context legality, transfer emission and native
while fixup/restore roles against the pinned source without executing native code.
Test nested nearest-loop targeting, conditional transfers, terminating-path joins,
return interactions, illegal/unreachable source, zero-iteration initialization,
caller isolation, original source maps and fuel exhaustion. Cover multiple fixups,
compiler exact/one-less quotas, deep nesting/failure cleanup and wide targets.
Native block cleanup, complete scopes/constant folding/watchdog/error-order parity,
for/switch transfer contexts, richer values/calls and full GSC acceptance remain open.
