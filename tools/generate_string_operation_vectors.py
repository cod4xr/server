#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Record native plain-string addition and explicit scratch-capacity expansion cases."""

import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

from reference_source import ReferenceSource, declaration


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/string_operation_vectors.txt"
PRELUDE = r"""
#include <assert.h>
#include <inttypes.h>
#include <limits.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifndef __cdecl
#define __cdecl
#endif
"""
STUBS = r"""
typedef struct { union VariableUnion u; int type; } VariableValue;
static char left_bytes[100001], right_bytes[100001], result_bytes[8192];
static unsigned int left_length, right_length, result_length, removed[2];
static int script_error;
static const char *SL_ConvertToString(unsigned int value) {
    if (value == 1) return left_bytes;
    if (value == 2) return right_bytes;
    abort();
}
static unsigned int SL_GetStringLen(unsigned int value) {
    if (value == 1) return left_length;
    if (value == 2) return right_length;
    abort();
}
static unsigned int SL_GetStringOfSize(const char *text, unsigned int user, unsigned int length, int type) {
    if (user != 0 || type != 15 || length == 0 || length > sizeof(result_bytes)) abort();
    memcpy(result_bytes, text, length);
    result_length = length - 1;
    return 3;
}
static void SL_RemoveRefToString(unsigned int value) {
    if (value != 1 && value != 2) abort();
    removed[value - 1] += 1;
}
static float *Scr_AllocVectorInternal(void) { abort(); }
static void RemoveRefToVector(const float *value) { (void)value; abort(); }
static void VectorAdd(const float *left, const float *right, float *result) {
    (void)left; (void)right; (void)result; abort();
}
static unsigned int SL_GetStringForVector(const float *value) { (void)value; abort(); }
static unsigned int SL_GetStringForFloat(float value) { (void)value; abort(); }
static unsigned int SL_GetStringForInt(int value) { (void)value; abort(); }
static void Scr_UnmatchingTypesError(VariableValue *left, VariableValue *right) {
    (void)left; (void)right; abort();
}
static const char *va(const char *format, ...) { return format; }
static void Scr_Error(const char *message) { (void)message; script_error = 1; }
"""


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    header = reference.files["src/scr_vm.h"]
    parts = [PRELUDE, declaration(header, r"enum \$[A-Z0-9]+\s*\{\s*VAR_UNDEFINED.*?\};"),
             declaration(header, r"union VariableUnion\s*\{.*?\};"), STUBS]
    parts += [reference.function("src/cscr_variable.cpp", name)
              for name in ("Scr_CastWeakerStringPair", "Scr_EvalPlus")]
    parts.append((ROOT / "tools/oracles/string_operation_driver.c").read_text(encoding="utf-8"))
    with tempfile.TemporaryDirectory(prefix="cod4x-string-operation-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        subprocess.run([args.cc, "-std=c11", "-O0", "-Wall", "-Wextra", "-Werror",
                        str(source), "-o", str(executable)], check=True)
        rows = subprocess.check_output([str(executable)]).replace(b"\r\n", b"\n")
    expected = (
        "# Original plain-string addition; string/ref services are stubs, mixed services abort.\n"
        "# Native capacity errors are intentional Rust expansion cases, NOT parity successes.\n"
        "# seed left_length right_length native_error native_fnv64 complete_fnv64\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + rows
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("String operation fixtures differ from original helpers")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(rows.splitlines())} native plain-string/expansion cases")


if __name__ == "__main__":
    main()
