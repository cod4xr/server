#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Execute selected original vector helpers with explicit stubbed runtime services."""

import argparse
from pathlib import Path
import re
import subprocess
import sys
import tempfile

from generate_numeric_vectors import PRELUDE
from reference_source import ReferenceSource, declaration


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/vector_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if sys.byteorder != "little":
        raise SystemExit("Vector oracle requires little-endian x86/SSE")
    reference = ReferenceSource(args.reference)
    header = reference.files["src/scr_vm.h"]
    variables = reference.files["src/cscr_variable.cpp"]
    macros = []
    for name in ("VAR_MASK", "VARIABLELIST_CHILD_BEGIN"):
        match = re.search(rf"(?m)^#define {name} [^\n]+", variables)
        if not match:
            raise ValueError("Missing vector comparison variable macro")
        macros.append(match[0])
    for name in ("VectorAdd", "VectorSubtract", "VectorClear"):
        macros.append(declaration(reference.files["src/q_math.h"],
                                  rf"(?m)^#define {name}\([^\n]+"))
    parts = [PRELUDE, "#include <setjmp.h>\ntypedef float vec3_t[3];",
             declaration(header, r"enum \$[A-Z0-9]+\s*\{\s*VAR_UNDEFINED.*?\};"),
             declaration(header, r"union VariableUnion\s*\{.*?\};"), "\n".join(macros),
             (ROOT / "tools/oracles/vector_services.c").read_text(encoding="utf-8"),
             reference.function("src/q_math.c", "VectorCompare")]
    names = ("Scr_CastVector", "Scr_CastWeakerPair", "Scr_CastWeakerStringPair",
             "Scr_EvalPlus", "Scr_EvalMinus", "Scr_EvalMultiply", "Scr_EvalDivide",
             "Scr_EvalEquality", "Scr_EvalInequality", "Scr_EvalLess", "Scr_EvalGreater",
             "Scr_EvalLessEqual", "Scr_EvalGreaterEqual")
    for name in names:
        function = reference.function("src/cscr_variable.cpp", name)
        if name == "Scr_EvalEquality":
            # Preserve the source's explicit "Fall in case of error" path verbatim.
            function = ('#pragma GCC diagnostic push\n'
                        '#pragma GCC diagnostic ignored "-Wimplicit-fallthrough"\n'
                        + function + '\n#pragma GCC diagnostic pop')
        parts.append(function)
    parts += [(ROOT / f"tools/oracles/vector_{name}.c").read_text(encoding="utf-8")
              for name in ("driver", "cases")]
    outputs = []
    with tempfile.TemporaryDirectory(prefix="cod4x-vector-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        for optimization in ("-O0", "-O2"):
            executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
            subprocess.run([args.cc, "-std=c11", optimization, "-Wall", "-Wextra", "-Werror",
                            "-msse2", "-ffp-contract=off", "-fexcess-precision=standard",
                            str(source), "-lm", "-o", str(executable)], check=True)
            outputs.append(subprocess.check_output([str(executable)]).replace(b"\r\n", b"\n"))
    if outputs[0] != outputs[1]:
        raise ValueError("Original vector helper outputs differ between O0/O2 profiles")
    expected = (
        "# Original vector helpers + VectorCompare/macros; nearest SSE, FTZ/DAZ off.\n"
        "# Pool/ref/string/error services stubbed; errors intercepted, not native unwinding.\n"
        "# V is coordinate order, not source evaluation/push order; P/V bits exact.\n"
        "# Arithmetic NaNs record class only; scalar-only pairs are outside this corpus.\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + outputs[0]
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Vector fixtures differ from original functions")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(outputs[0].splitlines())} original vector helper cases at O0/O2")


if __name__ == "__main__":
    main()
