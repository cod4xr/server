# SPDX-License-Identifier: AGPL-3.0-or-later
"""Record selected actual x86 local-slot/branch/transfer emission calls and state."""
import argparse
from pathlib import Path

from local_scope_emission_model import validate
from local_scope_emission_reference import execute
from reference_source import ReferenceSource

OUTPUT = Path(__file__).resolve().parents[1] / "crates/script/tests/fixtures/local_scope_emission_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--assembler", default="nasm")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    output = execute(reference, args.cc, args.assembler)
    validate(output)
    expected = (
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
        "# Actual selected x86 slot/if/if-else/break/continue emitters; driver O0/O2 agree.\n"
        "# Modeled condition/body dispatch/string IDs/opcode writer/temp allocation; no parser/VM/freeing.\n"
        "# O opcode requests, colon local-name/remove-count operands; p source-position calls.\n"
        "# Native EmitOpcode optimizer and native code alignment are not executed.\n"
        "# P/B/D pre parent/then/else; R/F/G post; block fields follow local_scope_vectors.txt.\n"
        "# I 0 fresh b; 1 later b; 2 prior b; 3 fresh b,c; 4 prior b; 5 prior c; 6 inherited a.\n"
        "# E 0 common b; 1 disjoint; 2 common b,c with different orders; 3 prior c; 4 inherited else.\n"
        "# T continuing created target_public: synthetic count profiles, not whole source loops.\n"
    ).encode("ascii") + output
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Native local-scope emission fixtures differ")
    else:
        OUTPUT.write_bytes(expected)
    print("Verified 36 selected native scope-emission cases against independent expectations at driver O0/O2")


if __name__ == "__main__":
    main()
