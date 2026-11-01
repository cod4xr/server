# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run native array helpers against explicit bounded pool/reference test services."""

from pathlib import Path
import re
import subprocess
import sys
import tempfile

from generate_numeric_vectors import PRELUDE
from reference_source import ReferenceSource, declaration
import script_array_reference as roles
import script_array_write_reference as writes

ROOT = Path(__file__).resolve().parents[1]


def run(reference: ReferenceSource, cc: str) -> bytes:
    if sys.byteorder != "little":
        raise ValueError("Array oracle requires little-endian native union word layout")
    roles.verify(*roles.inputs(reference))
    writes.verify(*writes.inputs(reference))
    header = reference.files["src/scr_vm.h"]
    source = reference.files["src/cscr_variable.cpp"]
    macros = []
    for name in ("VAR_MASK", "VAR_STAT_MASK", "VAR_STAT_FREE", "VAR_STAT_HEAD", "VAR_STAT_MOVABLE",
                 "VAR_STAT_EXTERNAL", "VARIABLELIST_CHILD_BEGIN", "VARIABLELIST_PARENT_BEGIN",
                 "VARIABLELIST_CHILD_SIZE", "VARIABLELIST_PARENT_SIZE", "SL_MAX_STRING_INDEX",
                 "MAX_ARRAYINDEX", "VAR_ARRAYINDEXSTART", "VAR_NAME_BITS", "VAR_NAME_HIGH_MASK"):
        macros.append(declaration(source, rf"(?m)^#define {name} [^\n]+"))
    for name in ("IsObject", "IsObjectVal", "VAR_TYPE"):
        macros.append(declaration(source, rf"(?m)^#define {name}\([^\n]+"))
    macros.append(declaration(header, r"(?m)^#define VAR_NAME_LOW_MASK [^\n]+"))
    parts = [PRELUDE, "#define __regparm3\n#include <stdbool.h>",
             declaration(header, r"enum \$[A-Z0-9]+\s*\{\s*VAR_UNDEFINED.*?\};"),
             declaration(header, r"union VariableUnion\s*\{.*?\};"),
             "typedef union VariableUnion VariableUnion;",
             declaration(header, r"#pragma pack\(push, 1\).*?\}VariableValueInternal;"),
             "#pragma pack(pop)\ntypedef struct Variable Variable;", "\n".join(macros)]
    parts.append((ROOT / "tools/oracles/array_services.c").read_text(encoding="utf-8"))
    for name in ("Scr_GetArrayIndexValue", "IsValidArrayIndex", "GetInternalVariableIndex",
                 "AllocObject", "Scr_AllocArray", "AddRefToObject", "AddRefToValue",
                 "GetNewVariableIndexInternal2", "GetVariableIndexInternal",
                 "GetArrayVariableIndex", "GetArrayVariable", "GetArraySize",
                 "FindFirstSibling", "FindNextSibling", "CopyArray", "Scr_AddArrayKeys"):
        parts.append(reference.function("src/cscr_variable.cpp", name))
    parts.extend((ROOT / f"tools/oracles/array_{name}.c").read_text(encoding="utf-8")
                 for name in ("pool", "lvalue_services"))
    parts.append((ROOT / "tools/oracles/array_removal_services.c").read_text(encoding="utf-8"))
    for name in ("MakeVariableExternal", "FreeChildValue", "Scr_EvalArrayRef", "Scr_EvalVariable", "SetVariableValue", "RemoveVariableValue",
                 "SafeRemoveVariable", "SafeRemoveArrayVariable", "ClearArray"):
        parts.append(reference.function("src/cscr_variable.cpp", name))
    parts.extend((ROOT / f"tools/oracles/array_{name}.c").read_text(encoding="utf-8")
                 for name in ("lvalue_cases", "removal_cases", "cases"))
    outputs = []
    with tempfile.TemporaryDirectory(prefix="cod4x-array-oracle-") as scratch:
        directory = Path(scratch)
        path = directory / "oracle.c"
        path.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        for optimization in ("-O0", "-O2"):
            subprocess.run([cc, "-std=c11", optimization, "-Wall", "-Wextra", "-Werror",
                            "-msse2", str(path), "-o", str(executable)], check=True)
            traces = []
            # Error callbacks validate state then exit their process. No native or modeled unwinder.
            cases = [()] + [(str(tag),) for tag in range(1, 14) if tag != 10]
            cases += [("clear-undefined",)]
            cases += [("clear-index", str(tag), word) for tag, word in
                      ((6, "ffffffff"), (6, "800000"), (0, "0"), (1, "0"),
                       (3, "1"), (4, "0"), (5, "0"), (9, "0"))]
            for arguments in cases:
                result = subprocess.run([str(executable), *arguments], capture_output=True, timeout=60)
                if result.returncode:
                    raise ValueError(f"Array helper driver {optimization} {arguments} failed "
                                     f"({result.returncode}): {result.stderr.decode('ascii', errors='replace')}")
                traces.append(result.stdout.replace(b"\r\n", b"\n"))
            outputs.append(b"".join(traces))
    if outputs[0] != outputs[1]:
        raise ValueError("Native array helper results differ between O0/O2")
    return outputs[0]
