# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected script-parameter roles without executing native compiler/VM code."""

import argparse
from pathlib import Path
import re

from reference_source import ReferenceSource
import script_update_reference as assembly


def verify(compiler: str, vm: str, header: str, variables: str, factory: str) -> None:
    """Check default Undefined, safe setter skips/consumption and excess-argument errors."""
    for name, value in (("VAR_UNDEFINED", 0), ("VAR_CODEPOS", 7), ("VAR_PRECODEPOS", 8),
                        ("OP_CreateLocalVariable", 0x16), ("OP_SafeSetVariableFieldCached0", 0x31),
                        ("OP_SafeSetVariableFieldCached", 0x32), ("OP_checkclearparams", 0x35)):
        if not re.search(rf"\b{name}\s*=\s*0x{value:x}\b", header):
            raise ValueError("Script argument type/opcode identity changed")
    profiles = {
        "EmitThreadInternal_30": ("mov ecx, 0x1", "call Scr_FindLocalVarIndex\n",
                                  "cmp eax, 0x1", "add eax, 0x32", "call EmitOpcode\n"),
        "EmitThreadInternal_40": ("mov eax, 0x35", "call EmitOpcode\n"),
        "EmitThreadInternal_140": ("jnz EmitThreadInternal_30\n", "jmp EmitThreadInternal_40\n"),
        "EmitThreadInternal_50": ("mov esi, [esi+0x4]", "jz EmitThreadInternal_40\n"),
        "EmitCall_120": ("mov ebx, [eax]", "jz EmitCall_30\n", "jmp EmitCall_40\n"),
        "EmitCall_60": ("add esi, 0x1", "mov ebx, [ebx+0x4]", "jz EmitCall_50\n"),
        "EmitCall_40": ("call EmitOrEvalExpression\n", "jz EmitCall_60\n",
                        "call EmitValue\n", "mov ebx, [ebx+0x4]", "jnz EmitCall_40\n",
                        "call EmitOrEvalExpression\n\ttest al, al\n\tjz EmitCall_60\n"
                        "\tmov eax, edi\n\tcall EmitValue\n"),
        "EmitCall_50": ("mov edx, esi", "call EmitPostFunctionCall\n"),
        "EmitThreadInternal_70": ("call EmitStatement\n",),
        "EmitThreadInternal_80": ("call EmitStatement\n",),
    }
    for label, needles in profiles.items():
        if any(text not in assembly.block(compiler, label) for text in needles):
            raise ValueError("Compiler parameter initialization/check profile changed")
    check = assembly.block(compiler, "EmitThreadInternal_40")
    ordered = ("mov eax, 0x35\n\tcall EmitOpcode\n", "call AddOpcodePos\n",
               "mov edx, [ebp-0x1c]\n\tmov eax, [edx+0xc]",
               "mov edx, [eax+0x4]", "jz EmitThreadInternal_60\n")
    cursor = 0
    for needle in ordered:
        position = check.find(needle, cursor)
        if position < 0:
            raise ValueError("Parameter check must precede selected body traversal")
        cursor = position + len(needle)
    labels = assembly.dispatch(vm, "VM_ExecuteInternal", 135)
    profiles = {
        labels[0x16]: ("add dword [gFs+0x8], 0x1", "call GetNewVariable\n"),
        labels[0x31]: ("cmp dword [eax+0x4], 0x8", "jz VM_ExecuteInternal_810\n"),
        labels[0x32]: ("cmp dword [eax+0x4], 0x8", "jz VM_ExecuteInternal_1510\n"),
        "VM_ExecuteInternal_980": ("call SetVariableValue\n", "jmp VM_ExecuteInternal_800\n"),
        "VM_ExecuteInternal_1560": ("call SetVariableValue\n", "add dword [gFs], 0x1"),
        "VM_ExecuteInternal_800": ("sub dword [gFs+0xc], 0x8",),
        "VM_ExecuteInternal_810": ("mov ecx, [gFs]",),
        "VM_ExecuteInternal_1510": ("lea eax, [ecx+0x2]", "mov [gFs], eax"),
        labels[0x35]: ("cmp dword [eax+0x4], 0x8", "jz VM_ExecuteInternal_840\n", "jz VM_ExecuteInternal_850\n"),
        "VM_ExecuteInternal_840": ("mov dword [eax+0x4], 0x7",),
        "VM_ExecuteInternal_850": ("_cstring_function_called_", "jmp VM_ExecuteInternal_2600\n"),
    }
    for label, needles in profiles.items():
        if any(text not in assembly.block(vm, label) for text in needles):
            raise ValueError("VM script argument marker/setter/check profile changed")
    if 'db "function called with too many parameters",0' not in vm:
        raise ValueError("Extra script argument diagnostic profile changed")
    for name, value in (("VAR_MASK", "0x1F"), ("VAR_STAT_MOVABLE", "32"), ("VAR_STAT_HEAD", "64")):
        if not re.search(rf"(?m)^#define {name} {value}\s*$", variables):
            raise ValueError("Undefined child variable status/type profile changed")
    needles = ("assert(VAR_TYPE(newEntryValue) == VAR_UNDEFINED)",
               "newEntryValue->w.status = VAR_STAT_MOVABLE", "newEntryValue->w.status = VAR_STAT_HEAD")
    if any(text not in factory for text in needles):
        raise ValueError("New child variable Undefined creation evidence changed")


def verify_mutations(inputs: tuple[str, ...]) -> int:
    """Reject isolated meaningful in-memory role changes; no native code is executed."""
    compiler_roles = {
        "EmitThreadInternal_30": ("mov ecx, 0x1", "add eax, 0x32"),
        "EmitThreadInternal_40": ("mov eax, 0x35", "call EmitOpcode\n",
                                  "call AddOpcodePos\n", "mov eax, [edx+0xc]",
                                  "mov edx, [eax+0x4]", "jz EmitThreadInternal_60\n"),
        "EmitThreadInternal_140": ("jnz EmitThreadInternal_30\n", "jmp EmitThreadInternal_40\n"),
        "EmitThreadInternal_50": ("mov esi, [esi+0x4]", "jz EmitThreadInternal_40\n"),
        "EmitThreadInternal_70": ("call EmitStatement\n",),
        "EmitThreadInternal_80": ("call EmitStatement\n",),
        "EmitCall_120": ("mov ebx, [eax]", "jz EmitCall_30\n", "jmp EmitCall_40\n"),
        "EmitCall_60": ("add esi, 0x1", "mov ebx, [ebx+0x4]", "jz EmitCall_50\n"),
        "EmitCall_40": ("call EmitOrEvalExpression\n", "jz EmitCall_60\n",
                        "call EmitValue\n", "mov ebx, [ebx+0x4]", "jnz EmitCall_40\n"),
        "EmitCall_50": ("mov edx, esi", "call EmitPostFunctionCall\n"),
    }
    labels = assembly.dispatch(inputs[1], "VM_ExecuteInternal", 135)
    vm_roles = {
        labels[0x31]: ("jz VM_ExecuteInternal_810\n",),
        labels[0x32]: ("jz VM_ExecuteInternal_1510\n",),
        labels[0x35]: ("cmp dword [eax+0x4], 0x8", "jz VM_ExecuteInternal_840\n",
                       "jz VM_ExecuteInternal_850\n"),
        "VM_ExecuteInternal_840": ("mov dword [eax+0x4], 0x7",),
        "VM_ExecuteInternal_850": ("jmp VM_ExecuteInternal_2600\n",),
    }
    cases = []
    for index, profiles in ((0, compiler_roles), (1, vm_roles)):
        for label, needles in profiles.items():
            section = assembly.block(inputs[index], label)
            for needle in needles:
                if section.count(needle) != 1:
                    raise ValueError("Argument fault injection requires a unique scoped role")
                replacement = section.replace(needle, "mutated_role\n", 1)
                original = f"{label}:\n{section}"
                cases.append((index, inputs[index].replace(original, f"{label}:\n{replacement}", 1)))
    # Preserve opcode identity and Undefined factory assertions as separate inputs.
    match = re.search(r"\bOP_checkclearparams\s*=\s*0x35\b", inputs[2])
    if not match:
        raise ValueError("Missing recorded parameter-check opcode")
    cases.append((2, inputs[2][:match.start()] + "OP_checkclearparams = 0x36" + inputs[2][match.end():]))
    cases.append((4, inputs[4].replace("assert(VAR_TYPE(newEntryValue) == VAR_UNDEFINED)",
                                      "assert(VAR_TYPE(newEntryValue) == VAR_INTEGER)", 1)))
    section = assembly.block(inputs[0], "EmitThreadInternal_40")
    opcode = "\tmov eax, 0x35\n\tcall EmitOpcode\n"
    moved = section.replace(opcode, "", 1) + opcode
    cases.append((0, inputs[0].replace(f"EmitThreadInternal_40:\n{section}",
                                      f"EmitThreadInternal_40:\n{moved}", 1)))
    section = assembly.block(inputs[0], "EmitCall_40")
    swapped = section.replace("call EmitOrEvalExpression\n", "call swap_placeholder\n", 1)
    swapped = swapped.replace("call EmitValue\n", "call EmitOrEvalExpression\n", 1)
    swapped = swapped.replace("call swap_placeholder\n", "call EmitValue\n", 1)
    cases.append((0, inputs[0].replace(f"EmitCall_40:\n{section}", f"EmitCall_40:\n{swapped}", 1)))
    for index, mutated in cases:
        changed = list(inputs)
        changed[index] = mutated
        try:
            verify(*changed)
        except ValueError:
            continue
        raise ValueError("Argument source guard accepted an injected role change")
    return len(cases)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    inputs = (reference.files["src/asmsource/_scr_compiler.asm"],
              reference.files["src/asmsource/_scr_vm.asm"], reference.files["src/scr_vm.h"],
              reference.files["src/cscr_variable.cpp"],
              reference.function("src/cscr_variable.cpp", "GetNewVariableIndexInternal3"))
    verify(*inputs)
    print(f"Verified argument ordering/check roles; rejected {verify_mutations(inputs)} in-memory mutations")


if __name__ == "__main__":
    main()
