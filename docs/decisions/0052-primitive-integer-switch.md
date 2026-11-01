# ADR 0052: Primitive integer switch compilation

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

Primitive source supports loops and conditionals but rejects switch/case/default.
Switch introduces multiple entries, source-order fallthrough, default anywhere,
nearest-break nesting distinct from nearest-loop continue, and exit initialization
facts. The locked native compiler sorts/detects duplicate cases, saves/restores
break contexts and builds local initialization facts. Native OP_switch 0x7c admits
string/int types; its int path calls IsValidArrayIndex, whose 0x800000 ceiling
comes from encoded variable names. Native case counts are serialized as a short.
Those storage encodings must not become new engine allocation rules.

## Decision

Add integer switch to PrimitiveStatements and bound primitive modules, leaving
PureScalarReturn restricted. Evaluate the switched expression once, then use a
typed integer-admission instruction that preserves the original word and rejects
other value types without float coercion. Case labels must decode to constant
integers; strings remain unsupported until owned heap values exist. Preserve the
full signed int32 domain for keys/labels. Valid native in-range integer cases keep
their meaning; admitting formerly invalid negative/high keys is an intentional
capacity expansion, not native invalid-input parity. This domain is not a global
asset/entity/heap ID format. Keep typed instructions unrelated to serialized ABI.

Lower bodies in source order with separate label tasks, so labels can reopen
control flow after return/break/continue. Adjacent labels share body locations;
default can appear anywhere and fall through normally. Reject duplicate keys/
defaults, misplaced labels, unsupported label values and unlabeled body prefixes.
Keep the current explicit nonfinal-return/unreachable-source policy within each
section; do not claim native diagnostic/scope equivalence.

Use out-of-line balanced integer dispatch built from existing typed comparisons,
branches and jumps. Each case node has a checked fixed instruction footprint;
sorted case indices determine addresses, and child ranges strictly shrink. Admit
all count/address arithmetic, code/tasks/maps/control payload and sorting/search/
emission work. Use wide indices and configurable existing quotas rather than the
native short case count, 64 locals or 32 temporary/frame ceilings. Runtime case
selection takes logarithmic tree steps and shares global fuel; no heap table,
serialized pointer, unaccounted argument array or recursive lowering is added.

Retain a private active switch context and explicit nearest-break owner separately
from the active loop. Enter/leave restores prior ownership; continue still targets
the nearest loop through nested switches, including for-update facts/fixups. Save
outer contexts in flat owned tasks, never recursively linked boxes. Switch breaks
collect exit initialization facts and forward fixups. Each label merges incoming
facts with the prior fallthrough path; each exit merges actual break/fallthrough
facts, adding the unmatched incoming path only without default. Returning and
continuing paths do not establish facts after the switch. Release snapshots and
admitted case records exactly once before complete function/module publication.

## Evidence and limits

Guard pinned grammar/compiler/VM case typing, duplicate/sort, break context and
integer admission roles with fault injection; these do not execute native routines.
Separate valid native-domain examples from full-word/count expansion tests.
Exercise default/miss/empty/fallthrough/adjacent/duplicate labels, source maps/
privacy, type/error/evaluation order, every break/continue nesting direction,
initialization joins/excluded exits, loops/calls/builtins, exact/one-less admission,
wide case inventories, bounded deep source/failed cleanup and hostile typed code.
Retain isolated profile and complete workspace gates. String/object switch, general
heap/VM/diagnostic/cleanup parity, full mods and Linux remain open. No dependencies,
speculative crates, native runtime requirements or alternate target directory.
