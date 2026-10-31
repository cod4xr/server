# ADR 0009: Bounded unsigned IW3 fastfile stream validation

Date: 2026-10-31

Status: accepted and implemented; full-workspace validation recorded separately.

## Context

Stock maps are stored in fastfiles rather than raw BSP entries in the installed
stock IWDs. The pinned `db_load.cpp` accepts IWffu100/version 5, inflates ordinary
zlib data, reads a 44-byte XFile header, then interprets an asset graph. It explicitly
rejects authenticated IWff0100 files. Compression integrity and graph validity are
different trust boundaries and must not be presented as a loaded playable zone.

## Decision

Add `FastFileStream` in content. Validate the twelve-byte outer header, decode the
fixed XFile header into widened accounting fields, admit payload/external/block
budgets before growing owned output, and require exact declared output, valid zlib
termination/checksum, and no compressed trailing data. Reject signed files without
attempting an unsigned fallback. Nine block fields are this serialized profile's
shape, not the engine's global allocator or asset capacities.

Use the already-resolved flate2 1.1.10 with only its zlib-rs backend (MIT/Apache-2.0;
zlib-rs is Zlib licensed). This adds a direct policy entry, not another backend or
version. Output grows fallibly after actual bounded chunks decode, with geometric
capacity capped by the admitted declaration. Forged huge headers with little output
must not cause eager giant allocation. Decoder workspace is bounded by its format
but dependency allocation failures are not universally recoverable.

Do not allocate declared stream blocks, follow encoded pointers, load external
images, execute scripts, register assets, authenticate content, or mark a zone ready.
The owned payload remains an unvalidated serialized asset graph. Debug/errors expose
only metadata/counts. Semantic graph decoding needs its own schemas and budgets.

The operator-facing `--inspect-fastfile` mode uses the same bounded immutable file
snapshot boundary as IWD verification, opens no listener, and emits a distinct
`content.envelope_verified` event. Snapshot failures retain a typed I/O cause while
rendering only fixed categories/I/O kinds to avoid paths in logs or stderr.

## Alternatives and migration

Reusing the original loader would retain global native-pointer state and prevent a
safe 64-bit engine. Treating decompression as a loaded zone would erase the next
validation boundary. Eager allocation from the declared size would amplify forged
headers. Accepting trailers/multiple streams would make the consumed content
ambiguous. No existing engine ownership is migrated: graph readers will borrow the
owned stream and publish typed assets only after their own validation succeeds.

## Validation

Use independent Python zlib fixtures for valid streams, header/version/signed cases,
declared-size mismatches, corruption/truncation, expansion, trailers, exact budgets,
and forged huge declarations without backing output. Validate stock map/load
envelopes read-only outside Git and compare header/byte counts independently.
No successful asset load, geometry decode, game-client join, or timing improvement
follows from these checks. Asset graph/pointer resolution remains next work.
