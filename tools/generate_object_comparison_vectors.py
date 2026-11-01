#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run ordinary-pointer equality helpers with modeled records/ref callbacks, not native VM execution."""
import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

from generate_numeric_vectors import PRELUDE
from reference_source import ReferenceSource, declaration
import script_object_comparison_reference as roles

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/object_comparison_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    roles.verify(roles.inputs(reference))
    header = reference.files["src/scr_vm.h"]
    variables = reference.files["src/cscr_variable.cpp"]
    macros = [declaration(variables, rf"(?m)^#define {name} [^\n]+") for name in
              ("VAR_MASK", "VAR_STAT_EXTERNAL", "VARIABLELIST_PARENT_BEGIN", "VARIABLELIST_CHILD_BEGIN")]
    parts = [PRELUDE,
             declaration(header, r"enum \$[A-Z0-9]+\s*\{\s*VAR_UNDEFINED.*?\};"),
             declaration(header, r"union VariableUnion\s*\{.*?\};"),
             declaration(header, r"#pragma pack\(push, 1\).*?\}VariableValueInternal;"),
             "#pragma pack(pop)", "\n".join(macros),
             (ROOT / "tools/oracles/object_comparison_services.c").read_text(encoding="utf-8")]
    for name in ("Scr_CastWeakerPair", "Scr_EvalEquality", "Scr_EvalInequality"):
        function = reference.function("src/cscr_variable.cpp", name)
        if name == "Scr_EvalEquality":
            # The unchanged native function intentionally falls through to its
            # type-error case. Limit this warning exception to that one helper.
            function = ('#pragma GCC diagnostic push\n'
                        '#pragma GCC diagnostic ignored "-Wimplicit-fallthrough"\n'
                        + function + '\n#pragma GCC diagnostic pop')
        parts.append(function)
    parts.append((ROOT / "tools/oracles/object_comparison_cases.c").read_text(encoding="utf-8"))
    outputs = []
    with tempfile.TemporaryDirectory(prefix="cod4x-object-comparison-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        for optimization in ("-O0", "-O2"):
            subprocess.run([args.cc, "-std=c11", optimization, "-Wall", "-Wextra", "-Werror",
                            str(source), "-o", str(executable)], check=True)
            outputs.append(subprocess.check_output([str(executable)], timeout=60).replace(b"\r\n", b"\n"))
    if outputs[0] != outputs[1]:
        raise ValueError("Native ordinary-pointer comparisons differ between O0/O2")
    expected = (
        "# Actual C weaker-pair/equality/inequality helpers; ordinary parent type 18 only; O0/O2.\n"
        "# Modeled records/ref callbacks; no native allocation/free/ABI/compiler/VM/GC parity claim.\n"
        "# C and selected archival guards inspect CHILD namespace for array markers.\n"
        "# Corpus uses only Undefined/integer/function child markers; no array equality inference.\n"
        "# O evaluate child_marker unequal left_id right_id result\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + outputs[0]
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Ordinary pointer comparison fixtures differ from extracted helpers")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(outputs[0].splitlines())} ordinary-pointer helper outcomes at O0/O2")


if __name__ == "__main__":
    main()
