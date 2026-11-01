# SPDX-License-Identifier: AGPL-3.0-or-later
"""Record selected actual constant-loop planning and literal-true emission helpers."""
import argparse
from pathlib import Path

from local_scope_constant_model import validate
from local_scope_constant_reference import execute
from reference_source import ReferenceSource

OUTPUT = Path(__file__).resolve().parents[1] / "crates/script/tests/fixtures/local_scope_constant_vectors.txt"


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
        "# Actual selected x86 planning/primitive/slot/branch/loop/transfer helpers and pinned C Scr_CastBool.\n"
        "# Modeled dispatch/AST/string/opcode writer/temp allocation/nonreference release; driver O0/O2 agree.\n"
        "# No parser/VM/native value freeing/opcode optimization/alignment/unwinding execution.\n"
        "# K 0 true while break; 1 false while ANALYSIS ONLY; 2 true for break;\n"
        "# 3 false for ANALYSIS ONLY; 4 true while mixed exits; 5 true for mixed exits.\n"
        "# 6 true while disjoint break paths; 7 true for disjoint break paths.\n"
        "# Q expression-emitter dispatches (not runtime evaluation); D modeled scalar release callbacks.\n"
        "# False-loop emitters/diagnostics intentionally not executed; other fields follow scope fixtures.\n"
    ).encode("ascii") + output
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Native constant-scope fixtures differ")
    else:
        OUTPUT.write_bytes(expected)
    print("Verified 8 selected native constant-scope cases (6 emitted/2 analysis-only) at driver O0/O2")


if __name__ == "__main__":
    main()
