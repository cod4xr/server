# SPDX-License-Identifier: AGPL-3.0-or-later
"""Record selected actual x86 switch/local/transfer emission and modeled string IDs."""
import argparse
from pathlib import Path

from local_scope_switch_model import validate
from local_scope_switch_reference import execute
from reference_source import ReferenceSource

OUTPUT = Path(__file__).resolve().parents[1] / "crates/script/tests/fixtures/local_scope_switch_vectors.txt"


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
        "# Actual selected x86 switch/comparator/slot/branch/loop/transfer helpers; driver O0/O2 agree.\n"
        "# Modeled AST/dispatch/condition/string-ID/opcode writer/temp allocation/sorting/integer-key services.\n"
        "# No parser/VM/value freeing/native sort/allocator/code alignment/EmitOpcode optimizer execution.\n"
        "# P/R parent before/after; B blocks; O requests/Q condition calls as other scope fixtures.\n"
        "# S 0 no default; 1 disjoint default; 2 common default; 3 fallthrough; 4 empty case;\n"
        "# 5 existing outer local; 6 nested common if/else; 7 extra private name; 8 loop continue.\n"
        "# 9 nested both-break arms (unreachable remove count 255); 10 partial nested break/fallthrough.\n"
        "# 11 single string ID; 12 mixed integer/string; 13/14 distinct string IDs in opposite source order.\n"
        "# J serialized sorted keys; T modeled user-string transfers in source order (11..14 only).\n"
    ).encode("ascii") + output
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Native switch-scope fixtures differ")
    else:
        OUTPUT.write_bytes(expected)
    print("Verified 15 selected native switch-scope cases against independent expectations at driver O0/O2")


if __name__ == "__main__":
    main()
