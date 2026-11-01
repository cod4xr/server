# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected dynamic while roles; no original compiler/VM code executes."""

import re

import script_update_reference as assembly


COMPILER_PROFILES = {
    "EmitWhileStatement_150": ("call EmitOrEvalExpression\n",),
    "EmitWhileStatement_420": ("mov eax, 0x5e", "call EmitOpcode\n"),
    "EmitWhileStatement_340": ("call EmitStatement\n",),
    "EmitWhileStatement_210": ("mov eax, 0x63", "call EmitOpcode\n"),
    "EmitWhileStatement_430": ("sub eax, [ebp-0x7c]", "mov [edx], ax"),
}
VM_PROFILES = {
    "VM_ExecuteInternal_3810": ("call Scr_CastBool\n", "movzx edx, word [eax]",
                                "lea ecx, [eax+0x2]", "jnz VM_ExecuteInternal_800\n"),
    "VM_ExecuteInternal_1350": ("lea eax, [ecx+edx]", "jmp VM_ExecuteInternal_800\n"),
    "VM_ExecuteInternal_800": ("sub dword [gFs+0xc], 0x8",),
    "VM_ExecuteInternal_3860": ("call Scr_ScriptRuntimecheckInfiniteLoop\n",
                                "movzx eax, word [eax]", "sub edx, eax", "mov [gFs], edx"),
}


def verify(compiler: str, vm: str, header: str) -> None:
    """Tie while tag/emission to consuming numeric branches and backward offsets."""
    labels = assembly.dispatch(compiler, "EmitStatement", 79)
    if "call EmitWhileStatement\n" not in assembly.block(compiler, labels[0x27]):
        raise ValueError("While statement emitter dispatch changed")
    labels = assembly.dispatch(vm, "VM_ExecuteInternal", 135)
    if labels[0x5e] != "VM_ExecuteInternal_3810" or labels[0x63] != "VM_ExecuteInternal_3860":
        raise ValueError("While branch/backward VM dispatch changed")
    for name, value in (("OP_JumpOnFalse", 0x5e), ("OP_jumpback", 0x63)):
        if not re.search(rf"\b{name}\s*=\s*(?i:0x{value:x})\b", header):
            raise ValueError("While branch/backward header identity changed")
    for source, profiles in ((compiler, COMPILER_PROFILES), (vm, VM_PROFILES)):
        for label, needles in profiles.items():
            if any(needle not in assembly.block(source, label) for needle in needles):
                raise ValueError("While expression/body/branch/backward source profile changed")
    if "jnz VM_ExecuteInternal_800\nVM_ExecuteInternal_1350:\n" not in vm:
        raise ValueError("False condition no longer falls through to offset application")
