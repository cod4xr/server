# ADR 0008: Instance-owned, bounded BSP container metadata

Date: 2026-10-31

Status: accepted and implemented; full Windows workspace checks passed.

## Context

The pinned `com_bsp_load_obj.c` holds one global file/header and declares a fixed
100-chunk header array while traversing variable file metadata. The rewrite needs
validated map bytes and independently owned map instances, without copying those
globals or making directory/format bounds into engine allocation ceilings.

## Decision

Implement borrowed `BspFile` metadata in the existing content trust boundary.
Versions 6..18 use the original fixed per-version length/offset directory counts;
19..45 use counted type/length chunks and four-byte sequential alignment. These
are container layouts only, not claims that every version has a decoded compatible
geometry/game schema. Reject unsupported versions explicitly.

Validate full directory and all nonempty payload spans before publishing a file.
Use checked wide offsets, fallible metadata allocation, and configurable source,
lump-count, single-lump, and summed-payload budgets. Do not retain the 100-chunk
array or a 64-character map-name limit. Filenames/filesystem roots are not part
of this parser. Unknown, duplicate, and empty entries remain in source order;
lookup preserves the first match, including an empty first match.

Zero-length entries have no addressable payload and can carry unused/out-of-file
offsets, as in the source. Final chunk padding is not required when its bytes are
never accessed; any following nonempty payload must have a complete aligned span.
Preserve nonzero padding/trailing bytes in the original checksum. Old nonempty
spans into the header/directory are rejected as an intentional malformed-input
restriction. Repeated/overlapping old payloads count against summed-byte budgets.

Payload views expose encoded records only after nonzero/divisible element-size
validation. No struct casts, native pointers, collision data interpretation, global
current-map state, renderer, or filesystem access is introduced. Independent maps
can coexist as long as their immutable backing bytes remain available.

## Validation

Compile pinned original header/lump/checksum functions against synthetic valid
containers. Cover every source-supported container version, alignment, unknown/
duplicate/empty chunks, first-match behavior, exact budgets, more than 100 entries,
old-format repeated spans, malformed directories/payloads, element-size errors,
and independent instance lifetimes. Proprietary map bytes stay outside Git.
This is a prerequisite to semantic collision/entity/asset loading, not a loaded
playable map. Stock fastfile graphs and decoded map schemas remain separate work.

Ten BSP tests passed in debug/release. The C oracle checks 280 lookups/counts/
presence/checksums across all forty container versions; malformed boundaries,
budgets, 1,000 entries, wide raw kinds, overlapping old spans, independent maps,
and payload Debug redaction are covered. No proprietary map fixtures were added.
