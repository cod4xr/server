#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run native constructor/type/ref helpers with bounded modeled allocator/output services."""
import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

from generate_numeric_vectors import PRELUDE
from reference_source import ReferenceSource, declaration
import script_struct_reference as roles

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/struct_vectors.txt"


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
              ("VAR_MASK", "VAR_STAT_MASK", "VAR_STAT_EXTERNAL", "VAR_STAT_FREE", "VAR_NAME_BITS",
               "VARIABLELIST_PARENT_BEGIN", "VARIABLELIST_PARENT_SIZE")]
    macros += [declaration(variables, rf"(?m)^#define {name}\([^\n]+") for name in ("VAR_TYPE", "IsObject")]
    parts = [PRELUDE,
             declaration(header, r"enum \$[A-Z0-9]+\s*\{\s*VAR_UNDEFINED.*?\};"),
             declaration(header, r"union VariableUnion\s*\{.*?\};"),
             declaration(header, r"#pragma pack\(push, 1\).*?\}VariableValueInternal;"),
             "#pragma pack(pop)", "\n".join(macros),
             (ROOT / "tools/oracles/struct_services.c").read_text(encoding="utf-8")]
    for name in ("AllocObject", "AddRefToObject", "RemoveRefToObject", "Scr_GetObjectType"):
        parts.append(reference.function("src/cscr_variable.cpp", name))
    parts += [reference.function("src/cscr_vm.cpp", name) for name in ("Scr_AddObject", "Scr_AddStruct")]
    parts.append((ROOT / "tools/oracles/struct_cases.c").read_text(encoding="utf-8"))
    outputs = []
    with tempfile.TemporaryDirectory(prefix="cod4x-struct-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        for optimization in ("-O0", "-O2"):
            subprocess.run([args.cc, "-std=c11", optimization, "-Wall", "-Wextra", "-Werror",
                            str(source), "-o", str(executable)], check=True)
            outputs.append(subprocess.check_output([str(executable)], timeout=60).replace(b"\r\n", b"\n"))
    if outputs[0] != outputs[1]:
        raise ValueError("Native struct constructor outcomes differ between O0/O2")
    expected = (
        "# Actual C struct constructor/type/add/remove-ref helpers and selected declarations; O0/O2.\n"
        "# Modeled fresh-slot allocator/output slot; no native pool, freeing, entity cleanup, VM or ABI claim.\n"
        "# Debug leak counters exercised; final excess refcount zero denotes the constructor's implicit owner.\n"
        "# S debug sequence output_tag parent_tag excess_refs total_refs debug_refs output_calls\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + outputs[0]
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Struct fixtures differ from extracted constructor helpers")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(outputs[0].splitlines())} native struct constructor traces at O0/O2")


if __name__ == "__main__":
    main()
