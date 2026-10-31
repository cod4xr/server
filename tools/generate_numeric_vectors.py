#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run original scalar coercion/comparison/division; nonnumeric services abort."""

import argparse
from pathlib import Path
import re
import subprocess
import sys
import tempfile

from reference_source import ReferenceSource, declaration


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/numeric_vectors.txt"
PRELUDE = r"""
#include <assert.h>
#include <float.h>
#include <inttypes.h>
#include <limits.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <xmmintrin.h>
#ifndef __cdecl
#define __cdecl
#endif
#ifndef __regparm2
#define __regparm2
#endif
"""
STUBS = r"""
typedef struct { union VariableUnion u; int type; } VariableValue;
static struct { struct { struct { unsigned int type; } w; } variableList[1]; } gScrVarGlob;
static struct { int evaluate; } gScrVarPub;
static int script_error;
static float *Scr_AllocVectorInternal(void) { abort(); }
static void Scr_UnmatchingTypesError(VariableValue *left, VariableValue *right) {
    (void)left; (void)right; abort();
}
static void RemoveRefToVector(const float *vector) { (void)vector; abort(); }
static void VectorClear(float *vector) { (void)vector; abort(); }
static int VectorCompare(const float *left, const float *right) {
    (void)left; (void)right; abort();
}
static void SL_RemoveRefToString(unsigned int value) { (void)value; abort(); }
static void RemoveRefToObject(unsigned int value) { (void)value; abort(); }
static void Scr_Error(const char *message) { (void)message; script_error = 1; }
"""


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if sys.byteorder != "little":
        raise SystemExit("Numeric oracle requires little-endian x86/SSE")
    reference = ReferenceSource(args.reference)
    header = reference.files["src/scr_vm.h"]
    source = reference.files["src/cscr_variable.cpp"]
    macros = []
    for name in ("VAR_MASK", "VARIABLELIST_CHILD_BEGIN"):
        match = re.search(rf"(?m)^#define {name} [^\n]+", source)
        if not match:
            raise ValueError("Missing original variable macro")
        macros.append(match[0])
    parts = [PRELUDE, declaration(header, r"enum \$[A-Z0-9]+\s*\{\s*VAR_UNDEFINED.*?\};"),
             declaration(header, r"union VariableUnion\s*\{.*?\};"), "\n".join(macros), STUBS]
    names = ("Scr_CastWeakerPair", "Scr_EvalLess", "Scr_EvalGreater", "Scr_EvalGreaterEqual",
             "Scr_EvalLessEqual", "Scr_EvalEquality", "Scr_EvalInequality", "Scr_EvalDivide")
    parts += [reference.function("src/cscr_variable.cpp", name) for name in names]
    parts.append((ROOT / "tools/oracles/numeric_driver.c").read_text(encoding="utf-8"))
    with tempfile.TemporaryDirectory(prefix="cod4x-numeric-oracle-") as scratch:
        directory = Path(scratch)
        source_file = directory / "oracle.c"
        source_file.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        subprocess.run([args.cc, "-std=c11", "-O0", "-msse2", "-ffp-contract=off",
                        "-fexcess-precision=standard", str(source_file), "-lm", "-o", str(executable)],
                       check=True)
        rows = subprocess.check_output([str(executable)]).replace(b"\r\n", b"\n")
    expected = (
        "# Original scalar coercion/comparison/division; nearest SSE, FTZ/DAZ off.\n"
        "# Nonnumeric services abort; errors recorded, not native unwinding/stack cleanup.\n"
        "# Division NaNs record class only; nonarithmetic coercion retains exact input bits.\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + rows
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Scalar numeric fixtures differ from original functions")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(rows.splitlines())} original scalar numeric cases")


if __name__ == "__main__":
    main()
