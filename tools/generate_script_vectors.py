#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Record numeric and nearest-even wait behavior using pinned original functions."""

import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

from reference_source import ReferenceSource, declaration


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/legacy_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--cc", default="gcc")
    args = parser.parse_args()
    if sys.byteorder != "little":
        raise SystemExit("Original script oracle requires little-endian x86/SSE")
    reference = ReferenceSource(args.reference)
    header = reference.files["src/scr_vm.h"]
    parts = [
        "#include <stdint.h>\n#include <stdio.h>\n#include <stdlib.h>\n"
        "#include <string.h>\n#include <limits.h>\n#include <math.h>\n#include <xmmintrin.h>\n"
        "#ifndef __cdecl\n#define __cdecl\n#endif",
        declaration(header, r"enum \$[A-Z0-9]+\s*\{\s*VAR_UNDEFINED.*?\};"),
        declaration(header, r"union VariableUnion\s*\{.*?\};"),
        "typedef struct { union VariableUnion u; int type; } VariableValue;\n"
        "static struct { int framerate; } level;\n"
        "static struct { int error_index; } gScrVarPub;\n"
        "static int script_error;\nstatic const char *var_typename[VAR_COUNT];\n"
        "static void RemoveRefToValue(int type, union VariableUnion value) {\n"
        "    if (type != VAR_FLOAT && type != VAR_INTEGER) abort();\n}\n"
        "static const char *va(const char *format, ...) { return format; }\n"
        "static void Scr_Error(const char *message) { script_error = 1; }",
        reference.function("src/q_math.h", "f2rint"),
        reference.function("src/cscr_variable.cpp", "Scr_CastBool"),
        reference.function("src/cscr_variable.cpp", "Scr_EvalBoolNot"),
        reference.function("src/cscr_variable.cpp", "Scr_EvalBoolComplement"),
        reference.function("src/cscr_vm.cpp", "VM_CalcWaitTime"),
        (ROOT / "tools/oracles/script_driver.c").read_text(encoding="utf-8"),
    ]
    with tempfile.TemporaryDirectory(prefix="cod4x-script-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        subprocess.run([args.cc, "-std=gnu11", "-O0", "-msse2", "-ffp-contract=off",
                        "-fexcess-precision=standard", str(source), "-o", str(executable)], check=True)
        rows = subprocess.check_output([str(executable)]).replace(b"\r\n", b"\n")
    expected = (
        "# Original numeric/wait functions; explicit nearest-even SSE, FTZ/DAZ off.\n"
        "# Valid signed-integer wait domain plus source-rejected negative waits only.\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + rows
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Script fixtures differ from original functions")
    else:
        OUTPUT.parent.mkdir(parents=True, exist_ok=True)
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(rows.splitlines())} original numeric/wait cases")


if __name__ == "__main__":
    main()
