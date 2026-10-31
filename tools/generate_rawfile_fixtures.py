#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Encode synthetic raw-file-only IW3 collections with Python struct/zlib."""

import argparse
from pathlib import Path
import struct
import zlib


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/content/tests/fixtures/rawfiles.txt"
INLINE = 0xffffffff
INSERT = 0xfffffffe
NAME = b"scripts/secret.gsc"
DATA = b"synthetic\0binary\xff"


def words(*values: int) -> bytes:
    return struct.pack(f"<{len(values)}I", *values)


def address(offset: int) -> int:
    return 0x40000000 + offset + 1


def envelope(payload: bytes, virtual_bytes: int, temp_bytes: int = 12) -> bytes:
    blocks = [temp_bytes, 0, 0, 0, virtual_bytes, 0, 0, 0, 0]
    return b"IWffu100" + words(5) + zlib.compress(words(len(payload), 0, *blocks) + payload, 9)


def simple(data: bytes = DATA) -> tuple[bytes, int]:
    body = words(INLINE, len(data), 7) + NAME + b"\0" + data + b"\0"
    return words(0, 0, 1, INLINE, 31, INLINE) + body, 8 + len(NAME) + len(data) + 2


def aliases() -> tuple[bytes, int]:
    # Five root cells: inline, prior root alias, inline+insert, inserted alias, null.
    table_bytes = 5 * 8
    first = words(INLINE, len(DATA), INLINE) + NAME + b"\0" + DATA + b"\0"
    allocation = table_bytes + len(NAME) + len(DATA) + 2
    inserted = (allocation + 3) & ~3
    # Reuse a suffix as a distinct second name; no duplicate-name registration claim.
    second = words(address(table_bytes + 8), 0, 1) + b"\0"
    root = words(0, 0, 5, INLINE)
    table = b"".join(words(31, pointer) for pointer in
                     (INLINE, address(4), INSERT, address(inserted), 0))
    return root + table + first + second, inserted + 4 + 1


def fixtures() -> list[tuple[str, bytes]]:
    small, virtual = simple()
    shared, shared_virtual = aliases()
    binary = DATA
    script_name = b"shared/secret.gsc"
    script_root = words(1, INLINE, 1, INLINE, INLINE) + script_name + b"\0"
    script_virtual = (4 + len(script_name) + 1 + 3) & ~3
    script_body = words(address(4), len(binary), 1) + binary + b"\0"
    script_payload = script_root + words(31, INLINE) + script_body
    short_name = words(0, 0, 1, INLINE, 31, INLINE, INLINE, 0, 0) + b"unterminated"
    invalid_length = words(0, 0, 1, INLINE, 31, INLINE, INLINE, 0xffffffff, 1)
    invalid_name = words(0, 0, 1, INLINE, 31, INLINE, INSERT, 0, 1) + b"\0"
    missing_buffer = words(0, 0, 1, INLINE, 31, INLINE, INLINE, 1, 0) + NAME + b"\0"
    return [
        ("inline", envelope(small, virtual)),
        ("empty", envelope(*simple(b""))),
        ("aliases", envelope(shared, shared_virtual)),
        ("script_name", envelope(script_payload, script_virtual + 8 + len(binary) + 1)),
        ("null", envelope(words(0, 0, 1, INLINE, 31, 0), 8, 0)),
        ("null_buffer", envelope(words(0, 0, 1, INLINE, 31, INLINE, INLINE, 0, 0) + NAME + b"\0", 8 + len(NAME) + 1)),
        ("unsupported", envelope(words(0, 0, 1, INLINE, 3, INLINE), 8)),
        ("unknown_alias", envelope(words(0, 0, 1, INLINE, 31, address(0)), 8)),
        ("forward_null", envelope(words(0, 0, 2, INLINE, 31, address(12), 31, 0), 16, 0)),
        ("forward_alias", envelope(words(0, 0, 2, INLINE, 31, address(12), 31, INLINE), 16)),
        ("bad_temp", envelope(small, virtual, 11)),
        ("bad_virtual", envelope(small, virtual - 1)),
        ("negative_length", envelope(invalid_length, 64)),
        ("invalid_name", envelope(invalid_name, 64)),
        ("missing_buffer", envelope(missing_buffer, 64)),
        ("short_header", envelope(small[:29], virtual)),
        ("short_name", envelope(short_name, 64)),
        ("short_buffer", envelope(small[:-1], virtual)),
        ("bad_terminator", envelope(small[:-1] + b"!", virtual)),
        ("trailer", envelope(small + b"extra", virtual)),
    ]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    rows = ["# Synthetic raw-file collections only: name fastfile_hex"]
    rows += [f"{name} {encoded.hex()}" for name, encoded in fixtures()]
    expected = "\n".join(rows) + "\n"
    if args.check:
        if OUTPUT.read_text(encoding="utf-8") != expected:
            raise SystemExit("Raw-file fixtures differ")
    else:
        OUTPUT.parent.mkdir(parents=True, exist_ok=True)
        OUTPUT.write_text(expected, encoding="utf-8", newline="\n")
    print(f"Verified {len(rows) - 1} independent synthetic raw-file collections")


if __name__ == "__main__":
    main()
