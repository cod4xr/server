#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Exercise original ASCII name conversion; native string/ref services are stubbed."""

import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

from reference_source import ReferenceSource


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/identifier_vectors.txt"
PRELUDE = r"""
#include <ctype.h>
#include <inttypes.h>
#include <limits.h>
#include <locale.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifndef __cdecl
#define __cdecl
#endif
static char input_bytes[100001];
static char converted[8192];
static unsigned int removed;
static unsigned int SL_GetStringLen(unsigned int value) {
    if (value != 17) abort();
    return (unsigned int)strlen(input_bytes);
}
static const char *SL_ConvertToString(unsigned int value) {
    if (value != 17) abort();
    return input_bytes;
}
static unsigned int SL_GetStringOfSize(const char *text, unsigned int user,
                                      unsigned int length, int type) {
    if (user != 1 || type != 14 || length > sizeof(converted)) abort();
    memcpy(converted, text, length);
    return 23;
}
static void SL_RemoveRefToString(unsigned int value) {
    if (value != 17) abort();
    removed += 1;
}
"""


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    parts = [PRELUDE, reference.function("src/cscr_stringlist.cpp", "SL_ConvertToLowercase"),
             (ROOT / "tools/oracles/identifier_driver.c").read_text(encoding="utf-8")]
    with tempfile.TemporaryDirectory(prefix="cod4x-identifier-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        subprocess.run([args.cc, "-std=c11", "-O0", "-Wall", "-Wextra", "-Werror",
                        str(source), "-o", str(executable)], check=True)
        rows = subprocess.check_output([str(executable)]).replace(b"\r\n", b"\n")
    expected = (
        "# Original SL_ConvertToLowercase, C locale/ASCII; native string/ref services stubbed.\n"
        "# kind pattern_or_ascii length canonical_fnv64 normalized_branch\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + rows
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Identifier fixtures differ from original conversion")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(rows.splitlines())} original ASCII name conversions")


if __name__ == "__main__":
    main()
