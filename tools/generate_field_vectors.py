#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run selected actual C struct-field helpers with explicit bounded modeled services."""
import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

from generate_numeric_vectors import PRELUDE
from reference_source import ReferenceSource, declaration
import script_field_reference as roles

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/field_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    roles.verify(*roles.inputs(reference))
    if sys.byteorder != "little":
        raise ValueError("Field oracle requires little-endian union word layout")
    header = reference.files["src/scr_vm.h"]
    variables = reference.files["src/cscr_variable.cpp"]
    macros = [declaration(variables, rf"(?m)^#define {name} [^\n]+") for name in
              ("VAR_MASK", "VAR_STAT_MASK", "VAR_STAT_EXTERNAL", "VAR_STAT_FREE", "VAR_NAME_BITS",
               "VARIABLELIST_PARENT_BEGIN", "VARIABLELIST_CHILD_BEGIN", "FIRST_OBJECT",
               "FIRST_NONFIELD_OBJECT", "FIRST_DEAD_OBJECT")]
    macros += [declaration(variables, rf"(?m)^#define {name}\([^\n]+") for name in
               ("VAR_TYPE", "IsObject", "IsObjectVal")]
    parts = [PRELUDE,
             declaration(header, r"enum \$[A-Z0-9]+\s*\{\s*VAR_UNDEFINED.*?\};"),
             declaration(header, r"union VariableUnion\s*\{.*?\};"),
             declaration(header, r"#pragma pack\(push, 1\).*?\}VariableValueInternal;"),
             "#pragma pack(pop)", "\n".join(macros),
             (ROOT / "tools/oracles/field_services.c").read_text(encoding="utf-8")]
    for name in ("Scr_EvalVariable", "SetVariableValue", "Scr_FindVariableField",
                 "Scr_GetVariableFieldIndex", "ClearVariableField"):
        parts.append(reference.function("src/cscr_variable.cpp", name))
    parts.append((ROOT / "tools/oracles/field_cases.c").read_text(encoding="utf-8"))
    outputs = []
    with tempfile.TemporaryDirectory(prefix="cod4x-field-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        for optimization in ("-O0", "-O2"):
            subprocess.run([args.cc, "-std=c11", optimization, "-Wall", "-Wextra", "-Werror",
                            str(source), "-o", str(executable)], check=True)
            outputs.append(subprocess.check_output([str(executable)], timeout=60).replace(b"\r\n", b"\n"))
    if outputs[0] != outputs[1]:
        raise ValueError("Native field outcomes differ between O0/O2")
    expected = (
        "# Actual C field get/index/set/clear helpers; O0/O2; low 32-bit union words only.\n"
        "# Modeled lookup, reference callbacks and removal; no native allocator/free/ABI/compiler/VM claim.\n"
        "# Raw setter stores Undefined; source/VM Undefined assignment deletes (guarded separately).\n"
        "# Vector pointers/entities excluded. Function words are helper-only, not callable Rust references.\n"
        "# F input_tag input_word output_tag output_word\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + outputs[0]
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Field fixtures differ from extracted helpers")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(outputs[0].splitlines())} native field helper traces at O0/O2")


if __name__ == "__main__":
    main()
