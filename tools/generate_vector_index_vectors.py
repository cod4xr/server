#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run original Scr_EvalArray vector reads with bounded stubbed runtime services."""

import argparse
from pathlib import Path
import sys

from reference_source import ReferenceSource
import vector_reference

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/vector_index_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if sys.byteorder != "little":
        raise SystemExit("Vector index oracle requires little-endian x86/SSE")
    reference = ReferenceSource(args.reference)
    rows = vector_reference.run(reference, ("data", "index_cases"), args.cc)
    expected = (
        "# Original Scr_EvalArray vector reads; exact float bits; O0/O2.\n"
        "# Object/array/string/pool/ref/error services stubbed; no native unwind/cleanup claim.\n"
        "# I x_bits y_bits z_bits index_kind index_bits padding padding status [result_bits]\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + rows
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Vector index fixtures differ from original helper")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(rows.splitlines())} original vector-index outcomes at O0/O2")


if __name__ == "__main__":
    main()
