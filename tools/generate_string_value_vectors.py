#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Execute actual plain/localized comparisons and localized arithmetic with canonical labels."""

import argparse
from pathlib import Path
import sys

from reference_source import ReferenceSource
import vector_reference

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/string_value_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if sys.byteorder != "little":
        raise SystemExit("String value oracle requires little-endian x86/SSE")
    reference = ReferenceSource(args.reference)
    rows = vector_reference.run(reference, ("data", "string_identity_cases"), args.cc,
                                string_references=vector_reference.StringReferences.SYNTHETIC_IDENTITIES)
    expected = (
        "# Actual C plain/localized comparisons and localized arithmetic; O0/O2.\n"
        "# Canonical synthetic labels 1..6; ref callbacks counted, not native interning/cleanup.\n"
        "# Plain concatenation/mixed formatting use separate helpers or remain unsupported.\n"
        "# No native compiler/VM/game/localization/byte-content allocation claim.\n"
        "# S operation left_kind left_bits[3] right_kind right_bits[3] status [boolean]\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + rows
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("String value fixtures differ from actual helpers")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(rows.splitlines())} original string value outcomes at O0/O2")


if __name__ == "__main__":
    main()
