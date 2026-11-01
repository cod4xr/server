# SPDX-License-Identifier: AGPL-3.0-or-later
"""Record selected original C script-clock phases, without full VM execution."""
import argparse
from pathlib import Path

from reference_source import ReferenceSource
from scheduler_phase_model import validate
from scheduler_phase_reference import execute

OUTPUT = Path(__file__).resolve().parents[1] / "crates/script/tests/fixtures/scheduler_phase_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--cc", default="gcc")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    output = execute(reference, args.cc)
    validate(output)
    expected = (
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
        "# Actual copied VM_SetTime/Scr_RunCurrentThreads/Scr_IncTime; O0/O2 agree.\n"
        "# Modeled C field names (not ABI), cvars, lookup, clock, logging, VM_Resume and entity cleanup.\n"
        "# T mode boundary operation; modes disabled/no-array/no-bucket/hit/logged-hit.\n"
        "# Operations run/increment/run-then-increment; B raw clock/L lookup/O log/R resume/D remove/F entity cleanup.\n"
        "# G final-time accumulated-execution raw-clock-calls resumes logs entity-cleanups bucket-present.\n"
        "# The native 24-bit wrap is an observation, not a new Rust engine cap.\n"
    ).encode("ascii") + output
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Native scheduler phase fixtures differ")
    else:
        OUTPUT.write_bytes(expected)
    print("Verified 30 native scheduler phase cases at O0/O2")


if __name__ == "__main__":
    main()
