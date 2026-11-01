# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected for phases/optionals; original compiler routines do not execute."""

import re

import script_update_reference as assembly


PROFILES = {
    "EmitForStatement": ("call EmitStatement\n",),
    "EmitForStatement_670": ("mov eax, 0x5e", "call EmitOpcode\n"),
    "EmitForStatement_570": ("mov byte [scrCompileGlob+0x2c], 0x1",
                             "mov byte [scrCompileGlob+0x34], 0x1", "call EmitStatement\n"),
    "EmitForStatement_330": ("mov ecx, [scrCompileGlob+0x38]",),
    "EmitForStatement_360": ("mov [edx], eax", "mov ecx, [ecx+0x8]"),
    "EmitForStatement_450": ("call EmitStatement\n", "mov eax, 0x63", "call EmitOpcode\n"),
    "EmitForStatement_470": ("mov ecx, [scrCompileGlob+0x30]",),
    "EmitForStatement_490": ("mov [edx], eax", "mov ecx, [ecx+0x8]"),
    "EmitForStatement_480": ("mov [scrCompileGlob+0x2c], bl", "mov [scrCompileGlob+0x30], esi",
                             "mov [scrCompileGlob+0x34], al", "mov [scrCompileGlob+0x38], edx"),
}


def verify(source: str, tables: dict[str, list[int]], compiler: str) -> None:
    """Check optional productions and initializer/body/continue-update/backward roles."""
    section = source.split("yyparse_jumptab_0:\n", 1)[1].split("yyparse_jumptab_1:", 1)[0]
    labels = re.findall(r"\bdd (yyparse_\d+)\s*", section)
    if len(labels) != 134:
        raise ValueError("For optional grammar inventory changed")
    for rule, lhs, length, needles in (
        (26, 93, 1, ("mov dword [esp], 0x41", "call node1\n")),
        (27, 93, 0, ("mov dword [esp], 0x0", "call node0\n")),
        (95, 104, 0, ("mov dword [esp], 0x0", "call node0\n")),
        (96, 104, 1, ("mov ebx, [ebp-0x280c]",)),
    ):
        body = assembly.block(source, labels[rule])
        if tables["LHS"][rule] != lhs or tables["LENGTH"][rule] != length:
            raise ValueError("For optional production shape changed")
        if any(needle not in body for needle in needles):
            raise ValueError("For optional construction profile changed")
    labels = assembly.dispatch(compiler, "EmitStatement", 79)
    if "call EmitForStatement\n" not in assembly.block(compiler, labels[0x28]):
        raise ValueError("For statement emitter dispatch changed")
    for label, needles in PROFILES.items():
        if any(needle not in assembly.block(compiler, label) for needle in needles):
            raise ValueError("For phase/fixup/restore profile changed")
