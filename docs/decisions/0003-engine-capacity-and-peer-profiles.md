# ADR 0003: Expanded engine capacity independent of peer compatibility limits

Date: 2026-10-31

Status: accepted architectural constraint; expanded subsystems remain unfinished.

## Context

The user explicitly requires removal of CoD4X's limitations, especially the asset,
image/material, scripting, memory, map, entity, and configstring ceilings that
previously forced character and prestige content cuts. Existing clients/servers
remain compatibility targets, and a custom client will later enable new features.

## Decision

Do not build the new engine around legacy fixed arrays or serialized index widths.
Use growable, accounted stores with configurable budgets and wide internal IDs.
Treat each legacy/extended peer profile as a representation boundary: explicitly
map engine state into its supported index, content, and message namespaces.
Keep the authoritative catalog intact. Define compatible subsets/fallbacks only
where their semantics are valid and visible to the operator; otherwise reject
incompatible admission before loading/playing rather than silently losing content.

Expanded client-visible namespaces/formats require explicit negotiated support.
Old clients retain their proven codec path. Current message/fragment/info limits
remain bounds of the implemented compatibility codec, not future engine-wide
allocation ceilings. File-size/code-quality and abuse-prevention limits remain
engineering safeguards and are not inherited game capacity restrictions.

## Alternatives and consequences

Blindly increasing constants fails when IDs are encoded in fewer bits or client
arrays/readers remain smaller. Silently truncating indices corrupts state.
Deleting content to match the smallest peer defeats the user's objective.
Unbounded allocation replaces a predictable ceiling with instability. Keeping
engine capacity and per-peer representation separate requires explicit content
planning, mapping, capability checks, diagnostics, and mixed-peer tests.

## Validation

Maintain docs/limits.md against actual source evidence. Test every implemented
store beyond its prior ceiling, prove round-trip mapping without narrowing, and
keep the full character/prestige regression catalog. Verify legacy peers against
their supported content view and extended peers against negotiated larger views.
No unchanged-client claim is established solely by enlarging a Rust allocation.
