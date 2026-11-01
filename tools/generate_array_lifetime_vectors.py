#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Execute selected native clear/free/reference/COW helpers with modeled parent allocation."""
import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

import array_reference
import array_lifetime_model
import script_array_lifetime_reference as roles
from reference_source import ReferenceSource

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/array_lifetime_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    roles.verify(roles.inputs(reference))
    parts = ["#define ARRAY_ORACLE_PARENT_ACCOUNTING 1", *array_reference.core(reference)]
    parts.append((ROOT / "tools/oracles/array_lifetime_services.c").read_text(encoding="utf-8"))
    for name in ("RemoveRefToValue", "MakeVariableExternal", "FreeChildValue", "ClearObjectInternal",
                 "FreeVariable", "RemoveRefToEmptyObject", "ClearObject", "RemoveRefToObject", "Scr_EvalArrayRef"):
        parts.append(reference.function("src/cscr_variable.cpp", name))
    parts.append((ROOT / "tools/oracles/array_lifetime_cases.c").read_text(encoding="utf-8"))
    outputs = []
    with tempfile.TemporaryDirectory(prefix="cod4x-array-lifetime-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        for optimization in ("-O0", "-O2"):
            subprocess.run([args.cc, "-std=c11", optimization, "-Wall", "-Wextra", "-Werror",
                            str(source), "-o", str(executable)], check=True)
            outputs.append(subprocess.check_output([str(executable)], timeout=60).replace(b"\r\n", b"\n"))
    if outputs[0] != outputs[1]:
        raise ValueError("Native array lifetime traces differ between O0/O2")
    array_lifetime_model.verify(outputs[0])
    expected = (
        "# Actual selected reference/clear/child-free/parent-free/COW/key helpers; O0/O2.\n"
        "# Modeled monotonic collision-free allocation, graph setup and output; not native allocator/VM/GC parity.\n"
        "# Parent/child free-list publication executes; allocator reuse/entities/string/vector cleanup remain open.\n"
        "# Q depth cycle_at aliases_per_node key_count freed_parents freed_children retained_aliases detached old_excess old_keys | selected_keys\n"
        "# cycle_at=-1 is acyclic; otherwise one self-cycle retains its downstream chain.\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + outputs[0]
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Array lifetime fixtures differ from extracted helper execution")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(outputs[0].splitlines())} native array lifetime/COW traces at O0/O2")


if __name__ == "__main__":
    main()
