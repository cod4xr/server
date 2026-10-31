# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected postfix compiler/VM paths; no original interpreter is executed."""

import re


def block(source: str, label: str) -> str:
    """Read one exact assembly basic block, without inferring a whole function."""
    match = re.search(rf"(?ms)^{re.escape(label)}:\n(.*?)(?=^[A-Za-z_][A-Za-z_0-9]*:|\Z)", source)
    if not match:
        raise ValueError("Missing recorded update basic block")
    return match[1]


def dispatch(source: str, prefix: str, count: int) -> list[str]:
    """Read a pinned direct dispatch shape; this is not executable VM conformance."""
    match = re.search(rf"(?m)^{prefix}_jumptab_0:\n((?:\s*dd {prefix}_\d+\n)+)", source)
    labels = re.findall(rf"\bdd ({prefix}_\d+)", match[1]) if match else []
    if len(labels) != count:
        raise ValueError("Recorded update dispatch inventory changed")
    return labels


def verify(compiler: str, vm: str, header: str) -> None:
    """Tie update tags/opcodes/type checks/32-bit mutations to the pinned profile."""
    compiler_labels = dispatch(compiler, "EmitStatement", 79)
    vm_labels = dispatch(vm, "VM_ExecuteInternal", 135)
    if "cmp eax, 0x86" not in vm or not re.search(r"\bVAR_INTEGER\s*=\s*0x6\b", header):
        raise ValueError("VM update opcode/type inventory changed")
    for tag, opcode, name, success, operation in (
        (0x29, 0x64, "inc", "VM_ExecuteInternal_690", "add"),
        (0x2a, 0x65, "dec", "VM_ExecuteInternal_740", "sub"),
    ):
        body = block(compiler, compiler_labels[tag])
        needles = ("call EmitVariableExpressionRef\n", f"mov eax, 0x{opcode:x}",
                   "call EmitOpcode\n", "mov eax, 0x39", "mov byte [scrCompileGlob+0x50], 0x1")
        if any(text not in body for text in needles):
            raise ValueError("Compiler postfix target/update/store profile changed")
        if not re.search(rf"\bOP_{name}\s*=\s*0x{opcode:x}\b", header):
            raise ValueError("Postfix header opcode identity changed")
        body = block(vm, vm_labels[opcode])
        needles = ("call Scr_EvalVariableFieldExtern\n", "cmp edx, 0x6", f"jz {success}\n")
        if any(text not in body for text in needles):
            raise ValueError("VM postfix integer-only branch changed")
        if f"{operation} dword [eax], 0x1" not in block(vm, success):
            raise ValueError("VM postfix word update changed")
