# SPDX-License-Identifier: AGPL-3.0-or-later
"""Build/run actual vector helpers with shared bounded harness service boundaries."""

from pathlib import Path
from enum import Enum
import re
import subprocess
import sys
import tempfile

from generate_numeric_vectors import PRELUDE
from reference_source import ReferenceSource, declaration

ROOT = Path(__file__).resolve().parents[1]


class StringReferences(Enum):
    """Concrete test-service boundaries; neither creates a native string pool."""
    UNAVAILABLE = "unavailable"
    SYNTHETIC_IDENTITIES = "synthetic_identities"


def services(profile: StringReferences) -> str:
    source = (ROOT / "tools/oracles/vector_services.c").read_text(encoding="utf-8")
    if profile == StringReferences.UNAVAILABLE:
        return source
    if profile != StringReferences.SYNTHETIC_IDENTITIES:
        raise ValueError("Unknown string-reference oracle profile")
    original = "static void SL_RemoveRefToString(unsigned int v) { (void)v; abort(); }"
    replacement = (
        "static unsigned int string_reference_removals;\n"
        "static unsigned int removed_string_labels[2];\n"
        "static void SL_RemoveRefToString(unsigned int v)\n"
        "{ if (v < 1 || v > 6 || string_reference_removals >= 2) abort();\n"
        "  removed_string_labels[string_reference_removals++] = v; }"
    )
    if source.count(original) != 1:
        raise ValueError("Original strict string-reference service changed")
    return source.replace(original, replacement, 1)


def run(reference: ReferenceSource, driver_names: tuple[str, ...], cc: str, *,
        string_references: StringReferences = StringReferences.UNAVAILABLE) -> bytes:
    header = reference.files["src/scr_vm.h"]
    variables = reference.files["src/cscr_variable.cpp"]
    macros = []
    for name in ("VAR_MASK", "VARIABLELIST_CHILD_BEGIN", "VARIABLELIST_PARENT_BEGIN", "VAR_STAT_MASK", "VAR_STAT_FREE"):
        match = re.search(rf"(?m)^#define {name} [^\n]+", variables)
        if not match:
            raise ValueError("Missing vector comparison variable macro")
        macros.append(match[0])
    for name in ("IsObject", "VAR_TYPE"):
        macros.append(declaration(variables, rf"(?m)^#define {name}\([^\n]+"))
    for name in ("VectorAdd", "VectorSubtract", "VectorClear"):
        macros.append(declaration(reference.files["src/q_math.h"],
                                  rf"(?m)^#define {name}\([^\n]+"))
    parts = [PRELUDE, "#include <setjmp.h>\ntypedef float vec3_t[3];",
             declaration(header, r"enum \$[A-Z0-9]+\s*\{\s*VAR_UNDEFINED.*?\};"),
             declaration(header, r"union VariableUnion\s*\{.*?\};"), "\n".join(macros),
             services(string_references),
             reference.function("src/q_math.c", "VectorCompare")]
    names = ("Scr_CastVector", "Scr_CastWeakerPair", "Scr_CastWeakerStringPair",
             "Scr_EvalPlus", "Scr_EvalMinus", "Scr_EvalMultiply", "Scr_EvalDivide",
             "Scr_EvalEquality", "Scr_EvalInequality", "Scr_EvalLess", "Scr_EvalGreater",
             "Scr_EvalLessEqual", "Scr_EvalGreaterEqual", "Scr_EvalArray")
    for name in names:
        function = reference.function("src/cscr_variable.cpp", name)
        if name == "Scr_EvalEquality":
            # Preserve the source's explicit "Fall in case of error" path verbatim.
            function = ('#pragma GCC diagnostic push\n'
                        '#pragma GCC diagnostic ignored "-Wimplicit-fallthrough"\n'
                        + function + '\n#pragma GCC diagnostic pop')
        parts.append(function)
    parts += [(ROOT / f"tools/oracles/vector_{name}.c").read_text(encoding="utf-8")
              for name in driver_names]
    outputs = []
    with tempfile.TemporaryDirectory(prefix="cod4x-vector-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        for optimization in ("-O0", "-O2"):
            executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
            subprocess.run([cc, "-std=c11", optimization, "-Wall", "-Wextra", "-Werror",
                            "-msse2", "-ffp-contract=off", "-fexcess-precision=standard",
                            str(source), "-lm", "-o", str(executable)], check=True)
            outputs.append(subprocess.check_output([str(executable)]).replace(b"\r\n", b"\n"))
    if outputs[0] != outputs[1]:
        raise ValueError("Original vector helper outputs differ between O0/O2 profiles")
    return outputs[0]
