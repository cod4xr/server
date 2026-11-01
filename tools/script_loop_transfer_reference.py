# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected while transfer contexts/fixups without executing native code."""

import re

import script_update_reference as assembly


PROFILES = {
    "EmitBreakStatement": ("cmp byte [scrCompileGlob+0x2c], 0x0", "jz EmitBreakStatement_10\n"),
    "EmitContinueStatement": ("cmp byte [scrCompileGlob+0x34], 0x0", "jz EmitContinueStatement_10\n"),
    "EmitBreakStatement_60": ("mov dword [ebx], 0x2", "mov eax, 0x62", "call EmitOpcode\n",
                              "mov eax, [scrCompileGlob+0x30]", "mov [ebx+0x8], eax",
                              "mov [scrCompileGlob+0x30], ebx"),
    "EmitContinueStatement_60": ("mov dword [ebx], 0x1", "mov eax, 0x62", "call EmitOpcode\n",
                                 "mov eax, [scrCompileGlob+0x38]", "mov [ebx+0x8], eax",
                                 "mov [scrCompileGlob+0x38], ebx"),
    "EmitBreakStatement_10": ("_cstring_illegal_break_st", "call CompileError\n"),
    "EmitContinueStatement_10": ("_cstring_illegal_continue", "call CompileError\n"),
    "EmitWhileStatement": ("movzx eax, byte [scrCompileGlob+0x2c]",
                            "mov edx, [scrCompileGlob+0x30]",
                            "movzx ecx, byte [scrCompileGlob+0x34]",
                            "mov ebx, [scrCompileGlob+0x38]"),
    "EmitWhileStatement_340": ("mov byte [scrCompileGlob+0x2c], 0x1",
                                "mov dword [scrCompileGlob+0x30], 0x0",
                                "mov byte [scrCompileGlob+0x34], 0x1",
                                "mov dword [scrCompileGlob+0x38], 0x0"),
    "EmitWhileStatement_200": ("mov ecx, [scrCompileGlob+0x38]",),
    "EmitWhileStatement_220": ("mov [edx], eax", "mov ecx, [ecx+0x8]"),
    "EmitWhileStatement_240": ("mov ecx, [scrCompileGlob+0x30]",),
    "EmitWhileStatement_260": ("mov [edx], eax", "mov ecx, [ecx+0x8]"),
    "EmitWhileStatement_250": ("mov [scrCompileGlob+0x2c], al", "mov [scrCompileGlob+0x30], edx",
                                "mov [scrCompileGlob+0x34], cl", "mov [scrCompileGlob+0x38], ebx"),
}


def verify(compiler: str, vm: str, header: str) -> None:
    """Check selected legality, linked fixups, nearest-loop restore and jump roles."""
    labels = assembly.dispatch(compiler, "EmitStatement", 79)
    for tag, name in ((0x3f, "Break"), (0x40, "Continue")):
        if f"call Emit{name}Statement\n" not in assembly.block(compiler, labels[tag]):
            raise ValueError("Loop transfer statement dispatch changed")
    for label, needles in PROFILES.items():
        if any(needle not in assembly.block(compiler, label) for needle in needles):
            raise ValueError("While transfer context/legality/fixup profile changed")
    if not re.search(r"\bOP_jump\s*=\s*0x62\b", header):
        raise ValueError("Loop transfer header jump identity changed")
    labels = assembly.dispatch(vm, "VM_ExecuteInternal", 135)
    body = assembly.block(vm, labels[0x62])
    if any(needle not in body for needle in ("lea eax, [ecx+0x5]", "add eax, [edx]", "mov [gFs], eax")):
        raise ValueError("Loop transfer VM jump profile changed")
