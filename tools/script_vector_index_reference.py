# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected vector read roles and immutable lvalue policy, without native VM execution."""

import re

import script_update_reference as assembly

TREE = ("mov edx, [ebp+0xc]\n\tmov [eax+0x4], edx",
        "mov edx, [ebp+0x10]\n\tmov [eax+0x8], edx")
GRAMMAR = ("mov edi, [edi-0x8]", "mov edx, [ecx-0x18]",
           "mov [esp+0x8], edi\n\tmov [esp+0x4], edx\n\tmov dword [esp], 0xd\n\tcall node4\n")
COMPILER = {
    "EmitVariableExpression_20": ("cmp eax, 0xd", "mov ebx, [ecx+0x4]",
                                   "mov eax, [ecx+0x8]", "call EmitOrEvalExpression\n"),
    "EmitVariableExpression_210": ("mov eax, ebx", "call EmitOrEvalPrimitiveExpression\n"),
    "EmitVariableExpression_200": ("mov eax, 0x20", "call EmitOpcode\n"),
    "EmitVariableExpression_70": ("call EmitValue\n", "jmp EmitVariableExpression_210\n"),
    "EmitVariableExpression_80": ("call EmitValue\n", "jmp EmitVariableExpression_200\n"),
}
READ = ("if ( index->type == VAR_INTEGER )", "(unsigned int)index->u.intValue >= 3u",
        "index->type = VAR_FLOAT", "index->u.floatValue = value->u.vectorValue[index->u.intValue]",
        "RemoveRefToVector(value->u.vectorValue)")


def verify(source: str, tree: str, compiler: str, vm: str, header: str, read: str, write: str) -> None:
    """Check index/object fields, index-first emission, helper arguments and strict vector reads."""
    if any(needle not in assembly.block(tree, "node4") for needle in TREE):
        raise ValueError("Index node object/index storage roles changed")
    section = source.split("yyparse_jumptab_0:\n", 1)[1].split("yyparse_jumptab_1:", 1)[0]
    labels = re.findall(r"\bdd (yyparse_\d+)\s*", section)
    if len(labels) != 134 or any(needle not in assembly.block(source, labels[63]) for needle in GRAMMAR):
        raise ValueError("Index reference grammar operand roles changed")
    for label, needles in COMPILER.items():
        if any(needle not in assembly.block(compiler, label) for needle in needles):
            raise ValueError("Index-first/object-second emission profile changed")
    if not re.search(r"\bOP_EvalArray\s*=\s*0x20\b", header):
        raise ValueError("Index opcode identity changed")
    label = assembly.dispatch(vm, "VM_ExecuteInternal", 135)[0x20]
    body = assembly.block(vm, label)
    needles = ("lea eax, [edx-0x8]", "mov [esp+0x4], eax\n\tmov [esp], edx\n\tcall Scr_EvalArray\n",
               "jmp VM_ExecuteInternal_800\n")
    if any(needle not in body for needle in needles):
        raise ValueError("Index VM object/index argument/collapse profile changed")
    match = re.search(r"(?ms)^    case VAR_VECTOR:\n(.*?)(?=^    default:)", read)
    if not match or any(needle not in match[1] for needle in READ):
        raise ValueError("Vector strict integer/range/copy read profile changed")
    if not re.search(r'if\s*\(\s*varValue.type\s*==\s*VAR_VECTOR\s*\)\s*{\s*Scr_Error\("vector components cannot be individually changed"\);', write):
        raise ValueError("Legacy vector component-write rejection changed")
