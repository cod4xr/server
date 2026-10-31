#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Test original defined scalar arithmetic and explicit archival integer instructions."""

import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

from arithmetic_reference import verify_machine_profile
from generate_numeric_vectors import PRELUDE
from reference_source import ReferenceSource, declaration


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/arithmetic_vectors.txt"
STUBS = r"""
typedef struct { union VariableUnion u; int type; } VariableValue;
static int script_error;
static float *Scr_AllocVectorInternal(void) { abort(); }
static void RemoveRefToVector(const float *v) { (void)v; abort(); }
static void VectorAdd(const float *a, const float *b, float *r) {
    (void)a; (void)b; (void)r; abort();
}
static void VectorSubtract(const float *a, const float *b, float *r) {
    (void)a; (void)b; (void)r; abort();
}
static void Scr_UnmatchingTypesError(VariableValue *a, VariableValue *b) {
    (void)a; (void)b; script_error = 1;
}
static unsigned int SL_GetStringForVector(const float *v) { (void)v; abort(); }
static unsigned int SL_GetStringForFloat(float v) { (void)v; abort(); }
static unsigned int SL_GetStringForInt(int v) { (void)v; abort(); }
static const char *SL_ConvertToString(unsigned int v) { (void)v; abort(); }
static unsigned int SL_GetStringLen(unsigned int v) { (void)v; abort(); }
static unsigned int SL_GetStringOfSize(const char *v, unsigned int u, unsigned int n, int t) {
    (void)v; (void)u; (void)n; (void)t; abort();
}
static void SL_RemoveRefToString(unsigned int v) { (void)v; abort(); }
static const char *va(const char *format, ...) { return format; }
static void Scr_Error(const char *message) { (void)message; script_error = 2; }
"""


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if sys.byteorder != "little":
        raise SystemExit("Arithmetic oracle requires little-endian x86/SSE")
    reference = ReferenceSource(args.reference)
    verify_machine_profile(reference.files["tools/cod4_dasm/src/_scr_variable.asm"])
    header = reference.files["src/scr_vm.h"]
    parts = [PRELUDE, declaration(header, r"enum \$[A-Z0-9]+\s*\{\s*VAR_UNDEFINED.*?\};"),
             declaration(header, r"union VariableUnion\s*\{.*?\};"), STUBS]
    names = ("Scr_CastWeakerPair", "Scr_CastWeakerStringPair", "Scr_EvalPlus", "Scr_EvalMinus",
             "Scr_EvalMultiply", "Scr_EvalOr", "Scr_EvalAnd", "Scr_EvalExOr", "Scr_EvalMod",
             "Scr_EvalShiftLeft", "Scr_EvalShiftRight")
    parts += [reference.function("src/cscr_variable.cpp", name) for name in names]
    parts += [(ROOT / "tools/oracles" / name).read_text(encoding="utf-8")
              for name in ("arithmetic_machine.c", "arithmetic_driver.c")]
    with tempfile.TemporaryDirectory(prefix="cod4x-arithmetic-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        rows = None
        for optimization in ("-O0", "-O2"):
            subprocess.run([args.cc, "-std=c11", optimization, "-Wall", "-Wextra", "-Werror",
                            "-msse2", "-ffp-contract=off", "-fexcess-precision=standard",
                            str(source), "-lm", "-o", str(executable)], check=True)
            actual = subprocess.check_output([str(executable)]).replace(b"\r\n", b"\n")
            if rows is not None and rows != actual:
                raise ValueError("Arithmetic fixtures depend on C optimization level")
            rows = actual
    expected = (
        "# C: original defined arithmetic; no signed overflow/invalid shifts/trapping remainder.\n"
        "# M: isolated x86 integer instructions after archival block verification, NOT full routines.\n"
        "# Nearest SSE, FTZ/DAZ off; nonnumeric services abort, errors record without unwinding.\n"
        "# origin operation left_kind left_bits right_kind right_bits error result_kind result_bits\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + rows
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Arithmetic fixtures differ from source/profile evidence")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(rows.splitlines())} defined-helper/instruction arithmetic cases")


if __name__ == "__main__":
    main()
