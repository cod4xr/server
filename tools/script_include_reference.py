# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected native include roles before choosing a closed-package import policy."""
import argparse
from pathlib import Path
import re

from reference_source import ReferenceSource
from script_update_reference import block


ROLES = {
    "ScriptCompile_30": (
        "call SL_ConvertToString\n", "call Scr_CreateCanonicalFilename\n", "mov esi, eax\n"),
    "ScriptCompile_80": (
        "mov byte [eax+0x2], 0x1\n", "add dword [scrCompileGlob+0x54], 0x8\n",
        "mov eax, [gScrCompilePub+0xc]\n", "call GetVariable\n", "call SGetObjectA\n"),
    "ScriptCompile_300": (
        "call SL_ConvertToString\n", "call Scr_LoadScriptInternal\n",
        "test eax, eax\n\tjz ScriptCompile_280\n",
        "cmp byte [eax+0x2], 0x0\n\tjnz ScriptCompile_290\n"),
    "ScriptCompile_290": (
        "cmp byte [esi+0x2], 0x0\n\tjz ScriptCompile_360\n",
        "movzx eax, word [esi]\n", "cmp edi, eax\n\tjz ScriptCompile_370\n"),
    "ScriptCompile_380": (
        "cmp byte [edx+0x2], 0x0\n\tjz ScriptCompile_360\n",
        "movzx eax, word [edx]\n", "cmp eax, edi\n\tjnz ScriptCompile_380\n"),
    "ScriptCompile_370": ("_cstring_duplicate_includ\n", "call CompileError\n"),
    "ScriptCompile_360": (
        "mov byte [ebx+0x2], 0x0\n", "mov eax, [ebp-0x60]\n", "call FindFirstSibling\n"),
    "ScriptCompile_390": (
        "call GetValueType\n", "sub eax, 0x1\n\tjnz ScriptCompile_410\n", "jmp ScriptCompile_430\n"),
    "ScriptCompile_430": (
        "call FindObject\n", "mov dword [esp+0x4], 0x1\n",
        "call FindVariable\n\tmov [ebp-0x5c], eax\n\ttest eax, eax\n\tjz ScriptCompile_410\n",
        "call Scr_EvalVariableExtern\n",
        "cmp dword [ebp-0x24], 0xd\n\tjz ScriptCompile_410\n",
        "call GetVariableName\n", "call GetVariable\n", "call SGetObjectA\n",
        "mov dword [esp], 0xd\n", "call SpecifyThreadPosition\n",
        "mov eax, [eax]\n\tmov [ebx], eax\n", "xor ecx, ecx\n", "call LinkThread\n"),
    "SpecifyThreadPosition": (
        "call GetVariable\n", "call Scr_EvalVariableExtern\n",
        "test eax, eax\n\tjz SpecifyThreadPosition_10\n",
        "test eax, eax\n\tjz SpecifyThreadPosition_20\n",
        "_cstring_function_s_alrea\n", "call CompileError\n"),
    "SpecifyThreadPosition_20": ("_cstring_function_s_alrea1\n", "call CompileError\n"),
    "EmitFunction_80": ("cmp edx, 0xd\n\tjz EmitFunction_100\n",),
    "ScriptCompile_280": ("_cstring_could_not_find_s\n", "call CompileError\n"),
}
IDENTITIES = (("VAR_CODEPOS", 7), ("VAR_DEVELOPER_CODEPOS", 12), ("VAR_INCLUDE_CODEPOS", 13))


def verify(compiler: str, header: str) -> None:
    """Check selected ordered blocks; this does not execute native dependency linking."""
    for label, needles in ROLES.items():
        body, position = block(compiler, label), 0
        for needle in needles:
            found = body.find(needle, position)
            if found < 0:
                raise ValueError("Selected include lookup/link roles changed")
            position = found + len(needle)
    for name, value in IDENTITIES:
        if not re.search(rf"\b{name}\s*=\s*(?i:0x{value:x})\b", header):
            raise ValueError("Include code-position type identity changed")


def verify_mutations(compiler: str, header: str) -> int:
    """Reject scoped role removal/reordering and type mutations without file changes."""
    cases = []
    for label, needles in ROLES.items():
        body = block(compiler, label)
        original = f"{label}:\n{body}"
        for needle in needles:
            if body.count(needle) != 1:
                raise ValueError("Include fault injection requires one scoped role")
            changed = body.replace(needle, "mutated_role\n", 1)
            cases.append((compiler.replace(original, f"{label}:\n{changed}", 1), header))
        if len(needles) > 1:
            changed = body.replace(needles[0], "", 1) + needles[0]
            cases.append((compiler.replace(original, f"{label}:\n{changed}", 1), header))
    for name, value in IDENTITIES:
        match = re.search(rf"\b{name}\s*=\s*(?i:0x{value:x})\b", header)
        if not match:
            raise ValueError("Missing include type identity")
        cases.append((compiler, header[:match.start()] + f"{name} = 0xff" + header[match.end():]))
    for source, types in cases:
        try:
            verify(source, types)
        except ValueError:
            continue
        raise ValueError("Include source guard accepted an injected role change")
    return len(cases)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    compiler, header = reference.files["src/asmsource/_scr_compiler.asm"], reference.files["src/scr_vm.h"]
    verify(compiler, header)
    print(f"Verified selected include roles; rejected {verify_mutations(compiler, header)} mutations")


if __name__ == "__main__":
    main()
