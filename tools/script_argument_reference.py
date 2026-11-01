# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected script-parameter roles without executing native compiler/VM code."""

import re

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
    }
    for label, needles in profiles.items():
        if any(text not in assembly.block(compiler, label) for text in needles):
            raise ValueError("Compiler parameter initialization/check profile changed")
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
