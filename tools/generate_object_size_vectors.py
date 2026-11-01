#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Execute native pointer-size helper with actual declarations and modeled ref services."""
import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

from generate_numeric_vectors import PRELUDE
from reference_source import ReferenceSource, declaration
import script_string_access_reference as roles

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/object_size_vectors.txt"


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
              ("VAR_MASK", "VAR_STAT_EXTERNAL", "VARIABLELIST_PARENT_BEGIN")]
    macros.append(declaration(variables, r"(?m)^#define VAR_TYPE\([^\n]+"))
    parts = [PRELUDE,
             declaration(header, r"enum \$[A-Z0-9]+\s*\{\s*VAR_UNDEFINED.*?\};"),
             declaration(header, r"union VariableUnion\s*\{.*?\};"),
             declaration(header, r"#pragma pack\(push, 1\).*?\}VariableValueInternal;"),
             "#pragma pack(pop)", "\n".join(macros),
             (ROOT / "tools/oracles/object_size_services.c").read_text(encoding="utf-8"),
             reference.function("src/cscr_variable.cpp", "Scr_EvalSizeValue"),
             (ROOT / "tools/oracles/object_size_cases.c").read_text(encoding="utf-8")]
    outputs = []
    with tempfile.TemporaryDirectory(prefix="cod4x-object-size-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        for optimization in ("-O0", "-O2"):
            subprocess.run([args.cc, "-std=c11", optimization, "-Wall", "-Wextra", "-Werror",
                            str(source), "-o", str(executable)], check=True)
            outputs.append(subprocess.check_output([str(executable)], timeout=60).replace(b"\r\n", b"\n"))
    if outputs[0] != outputs[1]:
        raise ValueError("Native pointer size outcomes differ between O0/O2")
    expected = (
        "# Actual C pointer-size helper and packed declarations on host ABI; O0/O2.\n"
        "# Modeled parent records/ref callbacks, not native pool/cleanup/unwinding/VM or pointer ABI.\n"
        "# Struct type 18 and array type 21 replay through Rust; other subtypes remain helper-only.\n"
        "# Union size word is native u16 data, not a struct field count.\n"
        "# O parent_type_word union_size_word result; all rows consume one modeled object ref.\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + outputs[0]
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Pointer size fixtures differ from extracted helper")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(outputs[0].splitlines())} native pointer-size outcomes at O0/O2")


if __name__ == "__main__":
    main()
