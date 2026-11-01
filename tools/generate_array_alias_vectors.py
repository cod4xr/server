#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Execute selected native cycle-reference drops and resulting array COW/key order."""
import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

import array_reference
from reference_source import ReferenceSource
import script_array_alias_reference as roles

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/array_alias_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    roles.verify(roles.inputs(reference))
    parts = array_reference.core(reference)
    parts.append((ROOT / "tools/oracles/array_alias_services.c").read_text(encoding="utf-8"))
    for name in ("RemoveRefToObject", "Scr_EvalArrayRef"):
        parts.append(reference.function("src/cscr_variable.cpp", name))
    parts.append((ROOT / "tools/oracles/array_alias_cases.c").read_text(encoding="utf-8"))
    outputs = []
    with tempfile.TemporaryDirectory(prefix="cod4x-array-alias-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        for optimization in ("-O0", "-O2"):
            subprocess.run([args.cc, "-std=c11", optimization, "-Wall", "-Wextra", "-Werror",
                            str(source), "-o", str(executable)], check=True)
            outputs.append(subprocess.check_output([str(executable)], timeout=60).replace(b"\r\n", b"\n"))
    if outputs[0] != outputs[1]:
        raise ValueError("Native cycle-alias COW traces differ between O0/O2")
    expected = (
        "# Actual selected native add/remove-ref, COW, copy and key enumeration helpers; O0/O2.\n"
        "# Modeled collision-free pool/root/field ownership setup/output; no native allocation/free/VM/GC claim.\n"
        "# Cycle roots drop through actual nonfreeing decrements; clear/free/entity paths abort.\n"
        "# L holders key_count old_excess_after_detach new_excess old_keys | copied_keys\n"
        "# A second write through the copied slot stays unique without another allocation/copy.\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + outputs[0]
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Array alias fixtures differ from extracted helper execution")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(outputs[0].splitlines())} native cycle-alias/COW traces at O0/O2")


if __name__ == "__main__":
    main()
