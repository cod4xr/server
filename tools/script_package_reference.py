# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected file-qualified/local lookup roles, not native compiler execution."""

import argparse
from pathlib import Path

from reference_source import ReferenceSource
from script_update_reference import block


ROLES = {
    "EmitFunction": (
        "cmp dword [eax], 0x14\n\tjz EmitFunction_20\n",
        "mov eax, [eax+0x4]\n",
        "call SL_ConvertToString\n",
        "call Scr_CreateCanonicalFilename\n",
        "mov esi, eax\n",
    ),
    "EmitFunction_190": (
        "mov [esp+0x4], esi\n",
        "mov eax, [gScrCompilePub+0x8]\n",
        "call FindVariable\n",
        "call Scr_EvalVariableExtern\n",
    ),
    "EmitFunction_180": (
        "mov [esp+0x4], esi\n",
        "mov eax, [gScrCompilePub+0xc]\n",
        "call GetVariable\n\tmov [esp], eax\n\tcall SGetObjectA\n",
        "jnz EmitFunction_50\n",
        "mov eax, [ebx+0x8]\n",
    ),
    "EmitFunction_50": (
        "mov eax, [ebx+0x8]\n",
        "call FindVariable\n",
        "jz EmitFunction_100\n",
        "call GetValueType\n",
        "sub eax, 0x1\n\tjz EmitFunction_110\n",
        "_cstring_unknown_function\n",
        "call CompileError\n",
    ),
    "EmitFunction_20": (
        "mov eax, [eax+0x4]\n",
        "mov eax, [scrCompileGlob+0x8]\n",
        "call GetVariable\n",
    ),
    "EmitFunction_100": ("_cstring_unknown_function\n", "call CompileError\n"),
    "EmitCall_110": (
        "cmp dword [ebx], 0x14\n\tjnz EmitCall_130\n",
        "mov eax, [scrCompileGlob+0x8]\n",
        "call FindVariable\n\ttest eax, eax\n\tjnz EmitCall_130\n",
        "call SL_ConvertToString\n",
        "mov eax, [gScrCompilePub+0x10]\n",
    ),
}


def verify(compiler: str) -> None:
    """Check canonical file identity, per-file names and local-only builtin fallback."""
    for label, needles in ROLES.items():
        body = block(compiler, label)
        position = 0
        for needle in needles:
            found = body.find(needle, position)
            if found < 0:
                raise ValueError("Selected script package lookup roles changed")
            position = found + len(needle)


def verify_mutations(compiler: str) -> int:
    """Reject scoped role removals and preserved-needle reorderings in memory."""
    cases = []
    for label, needles in ROLES.items():
        body = block(compiler, label)
        original = f"{label}:\n{body}"
        for needle in needles:
            if body.count(needle) != 1:
                raise ValueError("Package fault injection requires one scoped role")
            changed = body.replace(needle, "mutated_role\n", 1)
            cases.append(compiler.replace(original, f"{label}:\n{changed}", 1))
        changed = body.replace(needles[0], "", 1) + needles[0]
        cases.append(compiler.replace(original, f"{label}:\n{changed}", 1))
    for changed in cases:
        try:
            verify(changed)
        except ValueError:
            continue
        raise ValueError("Package source guard accepted an injected role change")
    return len(cases)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    compiler = ReferenceSource(args.reference).files["src/asmsource/_scr_compiler.asm"]
    verify(compiler)
    print(f"Verified selected package lookup roles; rejected {verify_mutations(compiler)} mutations")


if __name__ == "__main__":
    main()
