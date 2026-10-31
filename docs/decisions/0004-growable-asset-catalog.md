# ADR 0004: Growable engine asset catalog with generation-checked handles

Date: 2026-10-31

Status: accepted and implemented in `cod4x-assets`.

## Context

Asset/image/model limits previously forced the user to remove character and
prestige content. The server/client rewrite needs a full authoritative catalog
whose storage capacities do not inherit legacy client namespaces. Asset format
decoders and GPU residency are distinct future layers.

## Decision

Introduce an implemented asset-storage boundary, independent of protocol codecs.
It owns opaque encoded asset bytes and exact byte names, with explicit kinds,
fallible allocation, growable indexed storage, and configurable entry/payload/name
budgets. Budget counters describe logical stored bytes; allocator overhead and
decoded/GPU residency require their own accounting in the respective layers.

Use wide opaque handles containing catalog identity, slot, and generation.
Reusing a freed slot increments its generation; stale/cross-catalog handles must
never resolve to another asset. Handle namespace allocation must not wrap. A
generation that cannot advance retires its slot. Do not expose narrowing to a
legacy wire index as a catalog operation.

Keep name lookup and ownership separate. Names are opaque asset identifiers, not
filesystem paths; archive/path validation belongs to a future VFS type. Legacy
case-folding or alias rules belong to import/compatibility adapters and must be
validated without silently renaming or deleting catalog entries.

Registration must validate budgets and prepare fallible allocations before
publishing a new ID. Duplicate names, invalid names, and allocation/budget
failures preserve existing entries and accounting. Removal returns owned data,
invalidates the handle, and recycles metadata slots without allocating.

## Alternatives and consequences

Copying fixed XAsset arrays recreates inherited ceilings. Using raw vector
indices permits stale handles to identify reloaded assets. Global wire-sized IDs
couple storage to the smallest peer. Unbounded allocation makes content loading
unreliable. Explicit storage budgets and generation checks add bookkeeping while
keeping the complete catalog and predictable failure behavior.

This boundary does not decode images/models, load fastfiles, render assets, or
raise old clients' limits. It provides the owned registry those layers will use.

## Validation

Register synthetic encoded assets beyond audited old pool counts, including at
least 50 prestige entries and the full intended character roster. Test configurable
budgets, exact bytes, duplicates, stale IDs, cross-catalog IDs, unload/reload,
allocation failure paths where deterministically injectable, and usage accounting.
Keep source catalogs intact; legacy visibility/admission tests remain separate.
