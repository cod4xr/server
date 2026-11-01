#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run isolated archival isdefined comparison instructions; parameter access is modeled."""

import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

from generate_numeric_vectors import PRELUDE
from reference_source import ReferenceSource, declaration
import script_defined_reference as roles

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/defined_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    roles.verify(*roles.inputs(reference))
    parts = [PRELUDE, declaration(reference.files["src/scr_vm.h"],
                                  r"enum \$[A-Z0-9]+\s*\{\s*VAR_UNDEFINED.*?\};"),
             (ROOT / "tools/oracles/defined_instructions.c").read_text(encoding="utf-8")]
    outputs = []
    with tempfile.TemporaryDirectory(prefix="cod4x-defined-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        for optimization in ("-O0", "-O2"):
            subprocess.run([args.cc, "-std=c11", optimization, "-Wall", "-Wextra", "-Werror",
                            "-msse2", str(source), "-o", str(executable)], check=True)
            outputs.append(subprocess.check_output([str(executable)], timeout=60).replace(b"\r\n", b"\n"))
    if outputs[0] != outputs[1]:
        raise ValueError("Archival definedness instructions differ at O0/O2")
    expected = (
        "# Isolated archival x86 isdefined comparison instructions; O0/O2, guarded source roles.\n"
        "# Modeled first-parameter admission; not original function/VM/ABI/error/lifetime execution.\n"
        "# Unavailable tags/raw parent words/UINT32_MAX counts remain helper-only evidence.\n"
        "# D value_tag parent_word supplied_count missing|ok [defined_boolean]\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + outputs[0]
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Definedness fixtures differ from archival instruction evidence")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(outputs[0].splitlines())} isolated instruction/modeled-admission outcomes at O0/O2")


if __name__ == "__main__":
    main()
