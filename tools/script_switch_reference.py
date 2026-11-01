# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected legacy switch roles; full-word/count expansion is separate evidence."""

import argparse
from pathlib import Path
import re

from reference_source import ReferenceSource
import script_update_reference as assembly

COMPILER = {
    "EmitSwitchStatement": (
        "mov edx, [scrCompileGlob+0x28]\n\tmov [ebp-0x74], edx\n",
        "movzx ecx, byte [scrCompileGlob+0x2c]\n\tmov [ebp-0x6d], cl\n",
        "mov ebx, [scrCompileGlob+0x30]\n\tmov [ebp-0x6c], ebx\n",
        "mov byte [scrCompileGlob+0x2c], 0x0\n", "call EmitOrEvalExpression\n",
    ),
    "EmitSwitchStatement_590": ("mov eax, 0x7c\n\tcall EmitOpcode\n",),
    "EmitSwitchStatement_130": (
        "cmp eax, 0x7\n\tjz EmitSwitchStatement_460\n",
        "cmp eax, 0xb\n\tjz EmitSwitchStatement_470\n",
        "call CompileError\n",
    ),
    "EmitSwitchStatement_460": ("call IsValidArrayIndex\n", "jnz EmitSwitchStatement_630\n", "call CompileError\n"),
    "EmitSwitchStatement_630": ("call GetInternalVariableIndex\n",),
    "EmitSwitchStatement_470": (
        "mov esi, [esi+0x4]\n", "cmp dword [gScrCompilePub+0x20024], 0x2\n",
        "jz EmitSwitchStatement_610\n", "cmp byte [scrCompileGlob+0x1c], 0x0\n",
        "jnz EmitSwitchStatement_620\n",
    ),
    "EmitSwitchStatement_650": (
        "mov dword [esp+0x4], 0x1\n", "mov [esp], esi\n", "call SL_TransferRefToUser\n",
    ),
    "CompareCaseInfo": ("cmp [eax], edx\n", "ja CompareCaseInfo_10\n", "setb al\n"),
    "CompareCaseInfo_10": ("mov eax, 0xffffffff\n",),
    "EmitSwitchStatement_580": ("mov [ebx], ax\n", "CompareCaseInfo", "call qsort\n"),
    "EmitSwitchStatement_380": ("mov edx, [edi]\n\tadd edi, 0x8\n\tcmp edx, [edi]\n\tjz EmitSwitchStatement_370\n",),
    "EmitSwitchStatement_390": (
        "mov esi, [ebp-0x74]\n\tmov [scrCompileGlob+0x28], esi\n",
        "movzx eax, byte [ebp-0x6d]\n\tmov [scrCompileGlob+0x2c], al\n",
        "mov edx, [ebp-0x6c]\n\tmov [scrCompileGlob+0x30], edx\n",
    ),
}
VM = {
    "VM_ExecuteInternal_4060": (
        "movzx edx, word [eax]\n", "cmp edx, 0x2\n\tjz VM_ExecuteInternal_1160\n",
        "cmp edx, 0x6\n\tjz VM_ExecuteInternal_1170\n",
        "_cstring_cannot_switch_on",
    ),
    "VM_ExecuteInternal_1170": ("call IsValidArrayIndex\n", "jz VM_ExecuteInternal_2350\n", "call GetInternalVariableIndex\n"),
    "VM_ExecuteInternal_1160": (
        "mov eax, [eax]\n", "mov [ebp-0x2c], eax\n", "call SL_RemoveRefToString\n",
        "jmp VM_ExecuteInternal_1580\n",
    ),
}


def verify(compiler: str, vm: str, header: str, variables: str, valid: str, index: str) -> None:
    """Check ordered selected scopes/tags/case roles, never original routine execution."""
    for source, profiles in ((compiler, COMPILER), (vm, VM)):
        for label, needles in profiles.items():
            body = assembly.block(source, label)
            position = 0
            for needle in needles:
                found = body.find(needle, position)
                if found < 0:
                    raise ValueError("Selected ordered switch source roles changed")
                position = found + len(needle)
    if assembly.dispatch(vm, "VM_ExecuteInternal", 135)[0x7c] != "VM_ExecuteInternal_4060":
        raise ValueError("Switch VM dispatch identity changed")
    if not re.search(r"\bOP_switch\s*=\s*0x7C\b", header):
        raise ValueError("Switch opcode identity changed")
    if not re.search(r"(?m)^#define MAX_ARRAYINDEX 0x800000$", variables):
        raise ValueError("Native encoded integer-key domain changed")
    if not re.fullmatch(r"bool __cdecl IsValidArrayIndex\(unsigned int unsignedValue\)\s*\{\s*return unsignedValue < MAX_ARRAYINDEX;\s*\}", valid):
        raise ValueError("Native integer-key validity helper changed")
    if "assert(IsValidArrayIndex( unsignedValue ));" not in index or "return (unsignedValue + MAX_ARRAYINDEX) & VAR_NAME_LOW_MASK;" not in index:
        raise ValueError("Native encoded key helper changed")
    # Switch owns break slots, while an enclosing loop keeps its continue slots.
    for label in ("EmitSwitchStatement", "EmitSwitchStatement_390"):
        if re.search(r"mov(?: byte| dword)? \[scrCompileGlob\+0x(?:38|3c|40)\]", assembly.block(compiler, label)):
            raise ValueError("Switch entry/exit unexpectedly replaces continue context")


def verify_mutations(inputs: tuple[str, ...]) -> int:
    """Reject role removal/reordering plus opcode/domain/helper/context changes."""
    cases = []
    for index, profiles in ((0, COMPILER), (1, VM)):
        for label, needles in profiles.items():
            body = assembly.block(inputs[index], label)
            original = f"{label}:\n{body}"
            for needle in needles:
                if body.count(needle) != 1:
                    raise ValueError("Switch fault injection requires one scoped role")
                changed = body.replace(needle, "mutated_role\n", 1)
                cases.append((index, inputs[index].replace(original, f"{label}:\n{changed}", 1)))
            if len(needles) > 1:
                changed = body.replace(needles[0], "", 1) + needles[0]
                cases.append((index, inputs[index].replace(original, f"{label}:\n{changed}", 1)))
    cases += [(2, inputs[2].replace("OP_switch = 0x7C", "OP_switch = 0x7B", 1)),
              (3, inputs[3].replace("#define MAX_ARRAYINDEX 0x800000", "#define MAX_ARRAYINDEX 0x800001", 1)),
              (4, inputs[4].replace("< MAX_ARRAYINDEX", "<= MAX_ARRAYINDEX", 1)),
              (5, inputs[5].replace("+ MAX_ARRAYINDEX", "- MAX_ARRAYINDEX", 1))]
    body = assembly.block(inputs[0], "EmitSwitchStatement")
    changed = body + "\tmov byte [scrCompileGlob+0x3c], 0x0\n"
    cases.append((0, inputs[0].replace(f"EmitSwitchStatement:\n{body}", f"EmitSwitchStatement:\n{changed}", 1)))
    for index, changed in cases:
        mutated = list(inputs)
        mutated[index] = changed
        try:
            verify(*mutated)
        except ValueError:
            continue
        raise ValueError("Switch source guard accepted an injected role change")
    return len(cases)


def inputs(reference: ReferenceSource) -> tuple[str, ...]:
    """Extract only pinned compiler/VM/header and encoded-key helper sources."""
    return (reference.files["src/asmsource/_scr_compiler.asm"], reference.files["src/asmsource/_scr_vm.asm"],
            reference.files["src/scr_vm.h"], reference.files["src/cscr_variable.cpp"],
            reference.function("src/cscr_variable.cpp", "IsValidArrayIndex"),
            reference.function("src/cscr_variable.cpp", "GetInternalVariableIndex"))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    source = inputs(ReferenceSource(args.reference))
    verify(*source)
    print(f"Verified selected switch source roles; rejected {verify_mutations(source)} mutations")


if __name__ == "__main__":
    main()
