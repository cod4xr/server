#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run original string index/size helpers with explicit byte/ref/error test services."""

import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

from generate_numeric_vectors import PRELUDE
from reference_source import ReferenceSource, declaration
import script_string_access_reference as roles

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/string_access_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    roles.verify(*roles.inputs(reference))
    header = reference.files["src/scr_vm.h"]
    variables = reference.files["src/cscr_variable.cpp"]
    macros = [declaration(variables, rf"(?m)^#define {name} [^\n]+") for name in
              ("VAR_MASK", "VAR_STAT_MASK", "VAR_STAT_FREE", "VARIABLELIST_PARENT_BEGIN")]
    macros += [declaration(variables, rf"(?m)^#define {name}\([^\n]+")
               for name in ("VAR_TYPE", "IsObject")]
    parts = [PRELUDE, declaration(header, r"enum \$[A-Z0-9]+\s*\{\s*VAR_UNDEFINED.*?\};"),
             declaration(header, r"union VariableUnion\s*\{.*?\};"), "\n".join(macros),
             (ROOT / "tools/oracles/string_access_services.c").read_text(encoding="utf-8"),
             reference.function("src/cscr_variable.cpp", "Scr_EvalArray"),
             reference.function("src/cscr_variable.cpp", "Scr_EvalSizeValue"),
             (ROOT / "tools/oracles/string_access_cases.c").read_text(encoding="utf-8")]
    outputs = []
    with tempfile.TemporaryDirectory(prefix="cod4x-string-access-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        for optimization in ("-O0", "-O2"):
            subprocess.run([args.cc, "-std=c11", optimization, "-Wall", "-Wextra", "-Werror",
                            "-msse2", str(source), "-o", str(executable)], check=True)
            outputs.append(subprocess.check_output([str(executable)], timeout=60).replace(b"\r\n", b"\n"))
    if outputs[0] != outputs[1]:
        raise ValueError("String access results differ between O0/O2")
    expected = (
        "# Actual C string index/size functions; O0/O2, explicit terminated byte services.\n"
        "# Synthetic refs/output; returning error callbacks, not native interning/cleanup/unwinding/VM.\n"
        "# String indexing/size use strlen: embedded NUL ends the visible string.\n"
        "# R case index_tag index_word status [byte]; Z case value_tag status [integer_size]\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + outputs[0]
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("String access fixtures differ from actual helpers")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(outputs[0].splitlines())} original string access outcomes at O0/O2")


if __name__ == "__main__":
    main()
