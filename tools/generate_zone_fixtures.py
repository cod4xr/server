#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Encode synthetic IW3 root tables with independent Python struct/zlib."""

import argparse
from pathlib import Path
import struct
import zlib


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/content/tests/fixtures/zones.txt"
OUTER = b"IWffu100" + struct.pack("<I", 5)


def envelope(payload: bytes, virtual_bytes: int) -> bytes:
    blocks = [0] * 9
    blocks[4] = virtual_bytes
    return OUTER + zlib.compress(struct.pack("<11I", len(payload), 0, *blocks) + payload, 9)


def root(strings: list[int | bytes], assets: list[tuple[int, int]],
         remaining: bytes = b"") -> tuple[bytes, int]:
    words = [0xffffffff if isinstance(slot, bytes) else slot for slot in strings]
    table = struct.pack(f"<{len(words)}I", *words)
    inline = b"".join(slot + b"\0" for slot in strings if isinstance(slot, bytes))
    virtual_bytes = len(table) + len(inline)
    if assets:
        virtual_bytes = (virtual_bytes + 3) & ~3
    virtual_bytes += len(assets) * 8
    header = struct.pack("<4I", len(strings), bool(strings), len(assets), bool(assets))
    records = b"".join(struct.pack("<2I", kind, pointer) for kind, pointer in assets)
    return header + table + inline + records + remaining, virtual_bytes


def fixtures() -> list[tuple[str, bytes]]:
    mixed, mixed_virtual = root(
        [0, b"", b"secret\xff\x80", 0x40000001, 0xfffffffe],
        [(3, 0xffffffff), (0xffffffff, 0), (3, 0x40000001)], b"secret-body")
    wide, wide_virtual = root([0] * 70000, [(3, 0xffffffff)] * 70000, b"tail")
    aligned, aligned_virtual = root([b"ab"], [(31, 0xfffffffe)], b"tail")
    missing_strings = struct.pack("<4I", 1, 0, 0, 0)
    missing_assets = struct.pack("<4I", 0, 0, 1, 0)
    negative = struct.pack("<4I", 0x80000000, 1, 0, 0)
    short_string = struct.pack("<5I", 1, 1, 0, 0, 0xffffffff) + b"unterminated"
    short_table = struct.pack("<4I", 3, 1, 0, 0) + b"\0" * 8
    short_assets = struct.pack("<4I", 0, 0, 2, 1) + b"\0" * 8
    valid_empty = struct.pack("<4I", 0, 0, 0, 0)
    return [
        ("empty", envelope(valid_empty, 0)),
        ("mixed", envelope(mixed, mixed_virtual)),
        ("wide", envelope(wide, wide_virtual)),
        ("alignment", envelope(aligned, aligned_virtual)),
        ("short_block", envelope(aligned, aligned_virtual - 1)),
        ("missing_strings", envelope(missing_strings, 1024)),
        ("missing_assets", envelope(missing_assets, 1024)),
        ("negative_count", envelope(negative, 1024)),
        ("short_string", envelope(short_string, 1024)),
        ("short_table", envelope(short_table, 1024)),
        ("short_assets", envelope(short_assets, 1024)),
    ]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    rows = ["# Synthetic root tables only: name fastfile_hex"]
    rows += [f"{name} {encoded.hex()}" for name, encoded in fixtures()]
    expected = "\n".join(rows) + "\n"
    if args.check:
        if OUTPUT.read_text(encoding="utf-8") != expected:
            raise SystemExit("Zone prefix fixtures differ")
    else:
        OUTPUT.parent.mkdir(parents=True, exist_ok=True)
        OUTPUT.write_text(expected, encoding="utf-8", newline="\n")
    print(f"Verified {len(rows) - 1} independent synthetic zone prefixes")


if __name__ == "__main__":
    main()
