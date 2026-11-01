# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected vector list/traversal/slot roles without native parser/VM execution."""

import re

import script_update_reference as assembly


TREE = ("mov ebx, [ebp+0xc]", "mov edx, [ebp+0x8]", "mov [eax], edx",
        "mov edx, [ebx]", "mov [eax+0x4], edx", "mov [ebx], eax")
COMPILER = {
    "EmitOrEvalPrimitiveExpression": ("sub edx, 0x11", "cmp edx, 0x3a"),
    "EmitOrEvalPrimitiveExpression_370": ("mov edi, [eax+0x4]", "mov eax, [edi]"),
    "EmitOrEvalPrimitiveExpression_80": ("mov edx, [edx+0x4]", "cmp ecx, 0x1", "cmp ecx, 0x3"),
    "EmitOrEvalPrimitiveExpression_140": ("mov edx, [ebp-0x3c]", "mov eax, [edx]",
                                         "mov eax, [eax]", "call EmitOrEvalExpression\n"),
    "EmitOrEvalPrimitiveExpression_110": ("mov eax, [eax+0x4]", "mov [ebp-0x3c], eax",
                                         "call EmitOrEvalExpression\n", "call EmitValue\n"),
    "EmitOrEvalPrimitiveExpression_220": ("mov eax, 0x7e", "call EmitOpcode\n"),
}
GRAMMAR = {
    40: ("mov edx, [edi-0x8]", "mov [esp+0x4], edx", "mov dword [esp], 0x2e", "call node2\n"),
    111: ("mov ecx, [ecx-0x10]", "mov [esp+0x4], edi\n\tmov [esp], eax\n\tcall prepend_node\n"),
    112: ("mov dword [esp], 0x0", "call node0\n", "mov [esp+0x4], ebx", "call prepend_node\n"),
}


def verify(source: str, tree: str, compiler: str, vm: str, header: str, constructor: str) -> None:
    """Tie prepended list traversal to three components and reversed native value slots."""
    if any(needle not in assembly.block(tree, "prepend_node") for needle in TREE):
        raise ValueError("Expression prepend value/old-head/new-head profile changed")
    section = source.split("yyparse_jumptab_0:\n", 1)[1].split("yyparse_jumptab_1:", 1)[0]
    labels = re.findall(r"\bdd (yyparse_\d+)\s*", section)
    if len(labels) != 134:
        raise ValueError("Vector source grammar inventory changed")
    for rule, needles in GRAMMAR.items():
        if any(needle not in assembly.block(source, labels[rule]) for needle in needles):
            raise ValueError("Vector group/list operand/action roles changed")
    labels = assembly.dispatch(compiler, "EmitOrEvalPrimitiveExpression", 59)
    if labels[0x2e - 0x11] != "EmitOrEvalPrimitiveExpression_370":
        raise ValueError("Parenthesized primitive emitter dispatch changed")
    for label, needles in COMPILER.items():
        # The public entry immediately aliases _150 before its first instructions.
        label = "EmitOrEvalPrimitiveExpression_150" if label == "EmitOrEvalPrimitiveExpression" else label
        if any(needle not in assembly.block(compiler, label) for needle in needles):
            raise ValueError("Vector arity/head/traversal/emission profile changed")
    for name, value in (("OP_vector", 0x7e), ("VAR_VECTOR", 4)):
        if not re.search(rf"\b{name}\s*=\s*(?i:0x{value:x})\b", header):
            raise ValueError("Vector constructor type/opcode identity changed")
    labels = assembly.dispatch(vm, "VM_ExecuteInternal", 135)
    body = assembly.block(vm, labels[0x7e])
    if any(needle not in body for needle in ("sub eax, 0x10", "mov [gFs+0xc], eax", "call Scr_CastVector\n")):
        raise ValueError("Vector VM value-slot collapse/helper profile changed")
    if any(needle not in constructor for needle in ("for ( i = 2; i >= 0; --i )", "vec[2 - i] = value[i].u.floatValue", "vec[2 - i] = (float)value[i].u.intValue")):
        raise ValueError("Vector numeric coordinate/slot conversion profile changed")
