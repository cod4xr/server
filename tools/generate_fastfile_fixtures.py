#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Generate synthetic IW3 fastfile envelopes using independent Python zlib."""

import argparse
from pathlib import Path
import struct
import zlib


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/content/tests/fixtures/fastfiles.txt"
OUTER = b"IWffu100" + struct.pack("<I", 5)
BLOCKS = (164, 0, 0, 0, 223, 0, 0, 0, 0)
EXTERNAL = 1223352
SMALL = b"synthetic-secret-asset\x00\xff\x80"


def envelope(payload: bytes, declared: int | None = None) -> bytes:
    size = len(payload) if declared is None else declared
    header = struct.pack("<11I", size, EXTERNAL, *BLOCKS)
    return OUTER + zlib.compress(header + payload, level=9)


def fixtures() -> list[tuple[str, bytes, bytes]]:
    valid = [
        ("empty", b""),
        ("small", SMALL),
        ("pattern", bytes(index % 251 for index in range(150123))),
        ("expansion", b"Z" * 200003),
    ]
    rows = [(name, envelope(payload), payload) for name, payload in valid]
    good = envelope(SMALL)
    damaged = bytearray(good)
    damaged[-1] ^= 1
    rows += [
        ("bad_magic", b"invalid!" + good[8:], b""),
        ("bad_version", good[:8] + struct.pack("<I", 6) + good[12:], b""),
        ("signed", b"IWff0100" + good[8:], b""),
        ("bad_checksum", bytes(damaged), b""),
        ("bad_compression", OUTER + b"not-zlib", b""),
        ("short_header", OUTER + zlib.compress(b"short"), b""),
        ("short_payload", envelope(SMALL, len(SMALL) + 1), b""),
        ("long_payload", envelope(SMALL, len(SMALL) - 1), b""),
        ("huge_declaration", envelope(b"", 0xffffffff), b""),
        ("trailer", good + b"\x00", b""),
        ("concatenated", good + zlib.compress(b"second stream"), b""),
    ]
    return rows


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    rows = ["# Synthetic envelopes only: name encoded_hex payload_length payload_crc32"]
    for name, encoded, payload in fixtures():
        rows.append(f"{name} {encoded.hex()} {len(payload)} {zlib.crc32(payload)}")
    expected = "\n".join(rows) + "\n"
    if args.check:
        if OUTPUT.read_text(encoding="utf-8") != expected:
            raise SystemExit("Fastfile envelope fixtures differ")
    else:
        OUTPUT.parent.mkdir(parents=True, exist_ok=True)
        OUTPUT.write_text(expected, encoding="utf-8", newline="\n")
    print(f"Verified {len(rows) - 1} independent synthetic fastfile envelopes")


if __name__ == "__main__":
    main()
