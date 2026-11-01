#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run actual legacy comparisons involving Undefined, with bounded stub services."""

import argparse
from pathlib import Path
import sys

from reference_source import ReferenceSource
import vector_reference

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/undefined_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if sys.byteorder != "little":
        raise SystemExit("Undefined comparison oracle requires little-endian x86/SSE")
    reference = ReferenceSource(args.reference)
    rows = vector_reference.run(reference, ("data", "undefined_cases"), args.cc)
    expected = (
        "# Actual legacy coercion/comparison helpers involving Undefined; O0/O2.\n"
        "# Object/string/pool/ref/error services stubbed; no native cleanup/VM claim.\n"
        "# U operation left_kind left_bits[3] right_kind right_bits[3] status [boolean]\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + rows
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Undefined fixtures differ from original helpers")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(rows.splitlines())} original Undefined outcomes at O0/O2")


if __name__ == "__main__":
    main()
