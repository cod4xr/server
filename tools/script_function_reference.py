# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected function-reference emission/type/pop roles, not native execution."""

import argparse
from pathlib import Path
import re

from reference_source import ReferenceSource
import script_update_reference as assembly

COMPILER = {
    "EmitOrEvalPrimitiveExpression_150": (
        "mov edx, [eax]\n\tsub edx, 0x11\n\tcmp edx, 0x3a\n",
        "jmp dword [edx*4+EmitOrEvalPrimitiveExpression_jumptab_0]\n",
    ),
    "EmitOrEvalPrimitiveExpression_310": (
        "mov edi, [eax+0x4]\n", "mov eax, 0x15\n\tcall EmitOpcode\n",
        "mov eax, edi\n\tcall EmitFunction\n",
    ),
    "EmitPostFunctionCall_10": (
        "cmp edx, 0x12\n\tjz EmitPostFunctionCall_30\n",
        "cmp edx, 0x16\n\tjnz EmitPostFunctionCall_40\n",
        "mov esi, [eax+0x4]\n", "mov eax, esi\n\tcall EmitOrEvalExpression\n",
        "jnz EmitPostFunctionCall_50\n",
    ),
    "EmitPostFunctionCall_50": ("call EmitValue\n\tjmp EmitPostFunctionCall_150\n",),
    "EmitPostFunctionCall_150": (
        "cmp byte [ebp-0x31], 0x0\n\tjnz EmitPostFunctionCall_60\n",
        "not ebx\n", "mov edx, ebx\n\tmov eax, 0x51\n\tcall EmitOpcode\n",
    ),
}
VM = {
    "VM_ExecuteInternal_3280": (
        "mov eax, [gFs+0xc]\n\tadd eax, 0x8\n\tmov [gFs+0xc], eax\n",
        "mov dword [eax+0x4], 0x9\n\tjmp VM_ExecuteInternal_1110\n",
    ),
    "VM_ExecuteInternal_1110": (
        "mov eax, [gFs]\n\tmov edx, [eax]\n\tadd eax, 0x4\n\tmov [gFs], eax\n",
        "mov eax, [gFs+0xc]\n\tmov [eax], edx\n",
    ),
    "VM_ExecuteInternal_90": (
        "mov eax, [gFs+0xc]\n\tcmp dword [eax+0x4], 0x9\n\tjnz VM_ExecuteInternal_100\n",
    ),
    "VM_ExecuteInternal_110": (
        "mov eax, [gFs+0xc]\n\tmov edx, [eax]\n\tmov [gFs], edx\n",
        "sub eax, 0x8\n\tmov [gFs+0xc], eax\n\tjmp VM_ExecuteInternal_530\n",
    ),
}
IDENTITIES = (("OP_GetFunction", 0x15), ("OP_ScriptFunctionCallPointer", 0x51), ("VAR_FUNCTION", 9))


def verify(compiler: str, vm: str, header: str) -> None:
    """Check ordered selected roles without asserting frame/self/cleanup equivalence."""
    for source, profiles in ((compiler, COMPILER), (vm, VM)):
        for label, needles in profiles.items():
            body = assembly.block(source, label)
            position = 0
            for needle in needles:
                found = body.find(needle, position)
                if found < 0:
                    raise ValueError("Selected function-reference ordered roles changed")
                position = found + len(needle)
    labels = assembly.dispatch(vm, "VM_ExecuteInternal", 135)
    if labels[0x15] != "VM_ExecuteInternal_3280" or labels[0x51] != "VM_ExecuteInternal_90":
        raise ValueError("Function-reference VM dispatch identities changed")
    labels = assembly.dispatch(compiler, "EmitOrEvalPrimitiveExpression", 59)
    if labels[0x12 - 0x11] != "EmitOrEvalPrimitiveExpression_310":
        raise ValueError("Function-reference primitive compiler dispatch changed")
    for name, value in IDENTITIES:
        if not re.search(rf"\b{name}\s*=\s*(?i:0x{value:x})\b", header):
            raise ValueError("Function-reference opcode/type identity changed")


def verify_mutations(inputs: tuple[str, str, str]) -> int:
    """Reject scoped role removal and preserved-needle ordering mutations in memory."""
    cases = []
    for index, profiles in ((0, COMPILER), (1, VM)):
        for label, needles in profiles.items():
            body = assembly.block(inputs[index], label)
            original = f"{label}:\n{body}"
            for needle in needles:
                if body.count(needle) != 1:
                    raise ValueError("Reference fault injection requires one scoped role")
                changed = body.replace(needle, "mutated_role\n", 1)
                cases.append((index, inputs[index].replace(original, f"{label}:\n{changed}", 1)))
            if len(needles) > 1:
                # Every original needle remains present, but the first role moves last.
                changed = body.replace(needles[0], "", 1) + needles[0]
                cases.append((index, inputs[index].replace(original, f"{label}:\n{changed}", 1)))
    for name, value in IDENTITIES:
        match = re.search(rf"\b{name}\s*=\s*(?i:0x{value:x})\b", inputs[2])
        if not match:
            raise ValueError("Missing recorded function-reference identity")
        cases.append((2, inputs[2][:match.start()] + f"{name} = 0xff" + inputs[2][match.end():]))
    for index, changed in cases:
        mutated = list(inputs)
        mutated[index] = changed
        try:
            verify(*mutated)
        except ValueError:
            continue
        raise ValueError("Reference source guard accepted an injected role change")
    return len(cases)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    inputs = (reference.files["src/asmsource/_scr_compiler.asm"],
              reference.files["src/asmsource/_scr_vm.asm"], reference.files["src/scr_vm.h"])
    verify(*inputs)
    print(f"Verified selected function-reference roles; rejected {verify_mutations(inputs)} mutations")


if __name__ == "__main__":
    main()
