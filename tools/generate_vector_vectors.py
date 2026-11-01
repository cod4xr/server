#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Execute selected original vector helpers with explicit stubbed runtime services."""

import argparse
from pathlib import Path
import sys

from reference_source import ReferenceSource
import vector_reference


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/vector_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if sys.byteorder != "little":
        raise SystemExit("Vector oracle requires little-endian x86/SSE")
    reference = ReferenceSource(args.reference)
    rows = vector_reference.run(reference, ("data", "driver", "cases"), args.cc)
    expected = (
        "# Original vector helpers + VectorCompare/macros; nearest SSE, FTZ/DAZ off.\n"
        "# Pool/ref/string/error services stubbed; errors intercepted, not native unwinding.\n"
        "# V is coordinate order, not source evaluation/push order; P/V bits exact.\n"
        "# Arithmetic NaNs record class only; scalar-only pairs are outside this corpus.\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + rows
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Vector fixtures differ from original functions")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(rows.splitlines())} original vector helper cases at O0/O2")


if __name__ == "__main__":
    main()
