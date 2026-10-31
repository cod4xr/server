# ADR 0005: Bounded immutable IWD content reading

Date: 2026-10-31

Status: accepted and implemented; full game-content interoperability remains open.

## Context

Existing maps/mods use IWD ZIP archives. The future server and client need shared
content validation without host filesystem access, extraction, or graphics APIs.
Large catalogs require configurable entry/name/output budgets and ZIP64 metadata
support, independently of legacy asset pools. Encoded asset ownership is separate.

## Decision

Introduce `cod4x-content`, depending on `zip` 9.2.0 (MIT) and `crc32fast` 1.5.3
(MIT/Apache-2.0, already required by ZIP). Disable ZIP default
features; enable only `deflate-flate2-zlib-rs` for stored/DEFLATE IWD data. Do not
enable encryption, extraction, parallelism, or other codec features. The library
supports extraction but this boundary exposes none of those APIs.

Use immutable borrowed archive bytes. A streaming `ZipIterable` pass checks entry
count and directory-through-footer length before per-entry/index allocation. It
then validates all paths, local/central header consistency, supported compression,
and declared byte/name totals before `ZipArchive` builds its index. Both passes
use identical bytes/configuration. One ZIP metadata record may transiently allocate
its format-bounded name/extra/comment fields. Third-party metadata allocation is
not universally fallible; admitted counts/metadata constrain it, without promising
an allocator/RSS quota or recovery from every dependency allocation failure.
Validate compressed payload spans against the pre-directory byte region. Bound
the sum of compressed lengths separately, including repeated/overlapping spans,
so repeated references cannot multiply scans outside the admitted work budget.

Virtual paths preserve raw high bytes, fold ASCII case and backslashes, and reject
absolute/traversal/control/colon/empty components. They never grant host access.
Reject symlinks/encryption and contradictory headers. These deliberate invalid-
input restrictions differ from the old filesystem's permissive behavior.

The pinned `src/filesystem.c` builds hash chains by prepending directory entries;
`FS_FOpenFileReadDir` opens the saved position of the first matching chain node.
Preserve that last-entry precedence for exact duplicates and case/separator
aliases using original central-directory offsets. Do not rely on library index
order after exact-name deduplication. Preserve full source bytes; visibility does
not delete overridden content. Archive stack/mount order is a separate policy.

Each read fallibly allocates the admitted declared size, verifies exact output
length and CRC through EOF, and returns no partial output on failure. No cache,
extraction or asset decoding is implied. Retain one CRC32 per nonempty original
directory entry to compute the exact legacy IWD checksum with a caller-supplied
feed, including all overridden entries in order. The pinned implementation uses
seeded CRC32 over those little-endian values. This checksum validates legacy
metadata identity; it does not authenticate content or verify unread payloads.

An explicit full verification pass streams all original entries through a 16 KiB
scratch buffer, including hidden duplicates. The application exposes this as
`--verify-iwd`, using a bounded owned snapshot from a regular file and emitting
aggregate sanitized events. It does not bind sockets or extract files. Current
CLI budgets are library defaults; operator configuration and cancellation during
long content scans remain future runtime integration work.

## Dependency review

The maintained zip-rs project publishes the exact package with `build = false`;
the selected DEFLATE backend is Rust, avoiding a bundled C toolchain requirement.
Review the resolved dependency tree/licenses/build scripts with Cargo.lock before
accepting validation. Dependencies may contain unsafe internals; first-party
content code retains workspace unsafe prohibition. New releases require review.
API/source references: https://github.com/zip-rs/zip2 and the registry source for
the pinned version. `ZipIterable` is new in 9.2 and exposed through the library's
`unstable::read` namespace; the exact dependency pin and boundary tests are
mandatory before any upgrade. Rust itself remains on the stable toolchain.

Native resolved additions: crc32fast 1.5.3, flate2 1.1.10, zlib-rs 0.6.8,
getrandom 0.4.3, indexmap 2.14.2, equivalent 1.0.3, hashbrown 0.17.1, and
typed-path 0.13.0. All offer MIT/Apache-2.0 except zlib-rs (Zlib). Existing
memchr/platform dependencies are reused; wasm-only dependencies remain lockfile
targets, not native runtime code. crc32fast's build script probes rustc version;
getrandom's emits a sanitizer cfg. Neither downloads code or invokes a C compiler.
zip's extraction-related platform/random dependencies are unconditional in this
release even though this boundary never calls extraction. No prior dependency
versions changed in the lockfile. This is an explicit transitive cost decision.

## Alternatives and validation

Handwriting ZIP/DEFLATE duplicates a security-sensitive format implementation.
Calling `ZipArchive` before checking metadata can allocate a hostile whole index.
Extracting files introduces host-path and symlink risks with no current benefit.
Immutable slices prevent metadata/content mutation between validation and reads;
mapping/streaming very large archives remains future work with the same contract.

Use independent Python ZIP fixtures, original-source normalization evidence,
duplicate/alias precedence, ZIP64, exact budgets, truncation, CRC corruption,
decompression expansion, inconsistent headers, traversal, and repeated reads.
No stock map or gameplay compatibility claim follows from synthetic archive tests.
