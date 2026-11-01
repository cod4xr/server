# SPDX-License-Identifier: AGPL-3.0-or-later
"""Record actual selected native VM_Resume, without bytecode bodies or ownership."""
import argparse
from pathlib import Path

from reference_source import ReferenceSource
from vm_resume_model import validate
from vm_resume_reference import execute

OUTPUT = Path(__file__).resolve().parents[1] / "crates/script/tests/fixtures/vm_resume_vectors.txt"


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
        "# Actual selected x86 VM_Resume; driver O0/O2 agree.\n"
        "# Modeled lists, scalar archives, locals, clock, execution bodies, ref/free services.\n"
        "# No native wait opcode, insertion direction, archive creation, general frame/RC/unwinding claim.\n"
        "# R scenario; P dequeue/W clear wait/F free archive/E modeled execution/C clear bucket.\n"
        "# S restored key program-position call-depth local-count value-count, then scalar word/type pairs.\n"
        "# G bodies clock-calls wait-clears frees thread-decrements value-drops object-drops final-starttime.\n"
    ).encode("ascii") + output
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Native resume fixtures differ")
    else:
        OUTPUT.write_bytes(expected)
    print("Verified 10 selected native resume cases at driver O0/O2")


if __name__ == "__main__":
    main()
