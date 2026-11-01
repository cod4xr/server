#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Execute pinned pure builtin predicates under explicit count/type/result/error stubs."""

import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

from reference_source import ReferenceSource, declaration
import script_predicate_reference as predicates

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/predicate_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    profiles = predicates.inputs(reference)
    predicates.verify(*profiles)
    functions = "\n\n".join(profiles[1])
    types = declaration(reference.files["src/scr_vm.h"], r"enum \$[A-Z0-9]+\s*\{\s*VAR_UNDEFINED.*?\};")
    driver = (ROOT / "tools/oracles/predicate_driver.c").read_text(encoding="utf-8")
    source = driver.replace("/* ORIGINAL_FUNCTIONS */", types + "\n\n" + functions)
    outputs = []
    with tempfile.TemporaryDirectory(prefix="cod4x-predicate-oracle-") as scratch:
        directory = Path(scratch)
        path = directory / "oracle.c"
        path.write_text(source, encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        for optimization in ("-O0", "-O2"):
            subprocess.run([args.cc, "-std=c11", optimization, "-Wall", "-Wextra", "-Werror",
                            str(path), "-o", str(executable)], check=True)
            outputs.append(subprocess.check_output([str(executable)]).replace(b"\r\n", b"\n"))
    if outputs[0] != outputs[1]:
        raise ValueError("Predicate helpers disagree between O0/O2")
    expected = (
        "# Actual C isint/isfloat/isvector/isstring; O0/O2; count/type/result/error services stubbed.\n"
        "# Enum tag codes include markers; only U/plain-string/vector/float/int/function have Rust values.\n"
        "# No native compiler/VM/plugin/error-unwinding execution claim.\n"
        "# P predicate type_tag supplied_count status [boolean]\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + outputs[0]
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Predicate fixtures differ from original helpers")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(outputs[0].splitlines())} original predicate outcomes at O0/O2")


if __name__ == "__main__":
    main()
