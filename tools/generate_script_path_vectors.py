#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run the actual ASCII canonical-filename helper, intercepting its error callback."""
import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

from reference_source import ReferenceSource
import script_path_cases
import script_path_reference as roles

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/script_path_vectors.txt"
PRELUDE = r'''
#include <assert.h>
#include <ctype.h>
#include <locale.h>
#include <setjmp.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define ERR_DROP 1
static jmp_buf failure;
static _Noreturn void Com_Error(int kind, const char *format, ...)
{
    assert(kind == ERR_DROP && strcmp(format, "Filename '%s' exceeds maximum length") == 0);
    longjmp(failure, 1);
}
'''
DRIVER = r'''
static void show(const unsigned char *bytes, size_t count)
{
    if (!count) { putchar('~'); return; }
    for (size_t index = 0; index < count; ++index) printf("%02x", bytes[index]);
}
static void run(const struct Case *row)
{
    memset(output, 0xff, sizeof output);
    int rejected = setjmp(failure);
    if (!rejected) CreateCanonicalFilename((char *)output, (const char *)row->input, row->capacity);
    printf("P ");
    show(row->input, row->size);
    printf(" %d %c ", row->capacity, rejected ? 'E' : 'S');
    size_t length = rejected ? (size_t)row->capacity : strlen((const char *)output);
    show(output, length);
    assert(output[row->capacity] == 0xff);
    puts("");
}
int main(void)
{
    assert(setlocale(LC_CTYPE, "C") != NULL);
    for (size_t index = 0; index < sizeof cases / sizeof *cases; ++index) run(&cases[index]);
    return 0;
}
'''


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    roles.verify(roles.inputs(reference))
    corpus = script_path_cases.cases()
    declarations = []
    rows = []
    for index, (raw, capacity) in enumerate(corpus):
        values = ",".join(str(byte) for byte in raw + b"\0")
        declarations.append(f"static const unsigned char input_{index}[] = {{{values}}};")
        rows.append(f"{{input_{index}, {len(raw)}, {capacity}}}")
    declarations += ["struct Case { const unsigned char *input; size_t size; int capacity; };",
                     "static const struct Case cases[] = {" + ",\n".join(rows) + "};",
                     f"static unsigned char output[{max(capacity for _, capacity in corpus) + 1}];"]
    parts = [PRELUDE, reference.function("src/cscr_stringlist.cpp", "CreateCanonicalFilename"),
             "\n".join(declarations), DRIVER]
    outputs = []
    with tempfile.TemporaryDirectory(prefix="cod4x-script-path-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        for optimization in ("-O0", "-O2"):
            subprocess.run([args.cc, "-std=c11", optimization, "-Wall", "-Wextra", "-Werror",
                            str(source), "-o", str(executable)], check=True)
            outputs.append(subprocess.check_output([str(executable)], timeout=60).replace(b"\r\n", b"\n"))
    if outputs[0] != outputs[1]:
        raise ValueError("Native filename helper differs between O0/O2")
    script_path_cases.verify(outputs[0], corpus)
    expected = (
        "# Actual CreateCanonicalFilename at O0/O2 in the C locale; ASCII-only inputs.\n"
        "# Error callbacks intercept with setjmp; not native unwinding/interner/loader/filesystem parity.\n"
        "# P full_input_hex native_capacity S_or_E complete_output_or_error_prefix_hex\n"
        "# Embedded NUL rows establish native prefix behavior only; Rust catalog names reject NUL.\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + outputs[0]
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Script path fixtures differ from extracted helper execution")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(corpus)} native ASCII filename-helper outcomes at O0/O2")


if __name__ == "__main__":
    main()
