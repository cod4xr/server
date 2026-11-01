# SPDX-License-Identifier: AGPL-3.0-or-later
"""Record selected actual x86 loop/local/transfer emission calls and block state."""
import argparse
from pathlib import Path

from local_scope_loop_emission_model import validate
from local_scope_loop_emission_reference import execute
from reference_source import ReferenceSource

OUTPUT = Path(__file__).resolve().parents[1] / "crates/script/tests/fixtures/local_scope_loop_emission_vectors.txt"


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
        "# Actual selected x86 while/for/slot/branch/transfer emitters; driver O0/O2 agree.\n"
        "# Modeled condition/body dispatch/string IDs/opcode writer/temp allocation; no parser/VM/freeing.\n"
        "# P/R parent before/after; B blocks; O opcode requests as local_scope_emission_vectors.txt.\n"
        "# Q dynamic condition-emission calls; native opcode optimization/alignment are not executed.\n"
        "# L 0 while fallthrough; 1 break; 2 continue; 3 common branches; 4 mixed exits;\n"
        "# 5 for fallthrough; 6 for continue; 7 for break; 8 nested while break;\n"
        "# 9 for break with omitted condition. Present for conditions use native tag 0x41.\n"
    ).encode("ascii") + output
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Native loop-scope emission fixtures differ")
    else:
        OUTPUT.write_bytes(expected)
    print("Verified 10 selected native loop-emission cases against independent expectations at driver O0/O2")


if __name__ == "__main__":
    main()
