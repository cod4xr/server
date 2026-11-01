#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Record actual array key/copy/order helpers under bounded modeled services."""

import argparse
from pathlib import Path

import array_reference
from reference_source import ReferenceSource

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/array_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    rows = array_reference.run(reference, args.cc)
    expected = (
        "# Actual C key/allocation/copy/sibling/key-enumeration helpers; O0/O2.\n"
        "# Bounded modeled pool/hash/ref/output services, not native allocation/interner/GC/VM.\n"
        "# Native packed declarations on host ABI; no native layout/pointer ABI claim.\n"
        "# Native local-slot COW evaluator; entity branches abort; no Rust array/object execution yet.\n"
        "# I word valid [encoded_name decoded_type decoded_word]; K encoded_name type word\n"
        "# C width nested_width rotation object_count added_string_refs keys /copy /nested /nested_copy\n"
        "# R width nested_width rotation twice_copied_keys; U replaced_existing_key_order\n"
        "# A autovivified unique empty array; W width aliases write_index keys /copy /nested_copy\n"
        "# E rejected_value_tag fixed_error_category callback_count; checked process exits, no unwinding\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + rows
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Array helper fixtures differ from extracted functions")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(rows.splitlines())} native array helper traces at O0/O2; helper-only evidence")


if __name__ == "__main__":
    main()
