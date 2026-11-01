# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard array write/delete setup and Undefined dispatch; no native compiler/VM execution."""

import argparse
from pathlib import Path
import re

from reference_source import ReferenceSource
import script_update_reference as assembly

ROLES = {
    "SetVariableValue": ("RemoveRefToValue(VAR_TYPE(entryValue), entryValue->u.u);",
                         "entryValue->w.type &= ~VAR_MASK;", "entryValue->w.type |= value->type;",
                         "entryValue->u.u = value->u;"),
    "RemoveVariableValue": ("MakeVariableExternal(index, &gScrVarGlob.variableList[parentId + VARIABLELIST_PARENT_BEGIN]);",
                            "FreeChildValue(parentId, id);"),
    "SafeRemoveVariable": ("index = FindVariableIndexInternal(parentId, unsignedValue);", "if ( index )",
                            "MakeVariableExternal(index, &gScrVarGlob.variableList[parentId + VARIABLELIST_PARENT_BEGIN]);",
                            "FreeChildValue(parentId, id);"),
    "MakeVariableExternal": ("if ( VAR_TYPE(parentValue) == VAR_ARRAY )", "--parentValue->u.o.u.size;",
                             "var = Scr_GetArrayIndexValue(entryValue->w.classnum >> VAR_NAME_BITS);",
                             "RemoveRefToValue(var.type, var.u);"),
    "FreeChildValue": ("RemoveRefToValue(VAR_TYPE(entryValue), entryValue->u.u);",
                       "--gScrVarPub.totalObjectRefCount;", "--gScrVarPub.numScriptValues;",
                       "nextSiblingIndex = entryValue->nextSibling;", "prevSiblingIndex = entry->u.prev;",
                       "entryValue->w.status = 0;",
                       "entryValue->u.next = gScrVarGlob.variableList[VARIABLELIST_CHILD_BEGIN].u.next;",
                       "entry->u.prev = 0;",
                       "gScrVarGlob.variableList[gScrVarGlob.variableList[VARIABLELIST_CHILD_BEGIN].u.next + VARIABLELIST_CHILD_BEGIN].hash.u.prev = index;",
                       "gScrVarGlob.variableList[VARIABLELIST_CHILD_BEGIN].u.next = index;"),
    "Scr_EvalVariable": ("value.type = VAR_TYPE(entryValue);", "value.u.intValue = entryValue->u.u.intValue;",
                         "AddRefToValue(value.type, value.u);", "return value;"),
    "grammar_expression": ("mov dword [esp], 0x6", "call node2"),
    "grammar_group": ("mov dword [esp], 0x2e", "call node2"),
    "grammar_undefined": ("mov dword [esp], 0x1f", "call node1"),
    "EmitStatement_680": ("mov ecx, [ebx+0x8]", "mov edi, [ebx+0x4]",
                          "cmp dword [ecx], 0x6", "jz EmitStatement_80", "mov eax, ecx"),
    "EmitStatement_620": ("call EmitOrEvalExpression", "test al, al", "jnz EmitStatement_90"),
    "EmitStatement_90": ("call EmitValue", "jmp EmitStatement_590"),
    "EmitStatement_590": ("call EmitVariableExpressionRef", "mov eax, 0x39", "call EmitOpcode"),
    "EmitStatement_80": ("cmp dword [eax], 0x1f", "jnz EmitStatement_20", "cmp eax, 0xd",
                         "mov esi, [edi+0x8]", "mov ebx, [edi+0x4]", "call EmitOrEvalExpression"),
    "EmitStatement_20": ("mov eax, [ebp-0x3c]", "jmp EmitStatement_620"),
    "EmitStatement_50": ("cmp eax, 0x11", "jz EmitStatement_60", "call CompileError"),
    "EmitStatement_60": ("call EmitVariableExpressionRef", "jmp EmitStatement_670"),
    "EmitStatement_670": ("mov eax, 0x24", "call EmitOpcode"),
    "EmitVariableExpressionRef_20": ("cmp eax, 0xd", "mov edi, [ecx+0x8]", "mov ebx, [ecx+0x4]",
                                    "call EmitOrEvalExpression", "jnz EmitVariableExpressionRef_60"),
    "EmitVariableExpressionRef_60": ("call EmitValue", "jmp EmitVariableExpressionRef_150"),
    "EmitVariableExpressionRef_150": ("cmp eax, 0x11", "jz EmitVariableExpressionRef_70", "call CompileError"),
    "EmitVariableExpressionRef_70": ("call EmitVariableExpressionRef_160", "jmp EmitVariableExpressionRef_130"),
    "EmitVariableExpressionRef_130": ("mov eax, 0x23", "call EmitOpcode"),
    "VM_ExecuteInternal_1390": ("call Scr_EvalArrayRef", "mov [ebp-0x34], eax", "cmp edx, 0x6"),
    "VM_ExecuteInternal_1040": ("cmp edx, 0x2", "jz VM_ExecuteInternal_930"),
    "VM_ExecuteInternal_920": ("call IsValidArrayIndex", "jnz VM_ExecuteInternal_1050"),
    "VM_ExecuteInternal_1050": ("call GetArrayVariableIndex", "mov [ebp-0x3c], eax"),
    "VM_ExecuteInternal_930": ("call GetVariableIndexInternal", "mov [ebp-0x3c], eax",
                               "call SL_RemoveRefToString", "jmp VM_ExecuteInternal_1670"),
    "VM_ExecuteInternal_3620": ("test edi, edi", "jz VM_ExecuteInternal_820", "mov esi, [eax+0x4]",
                               "test esi, esi", "jz VM_ExecuteInternal_830", "call SetVariableValue"),
    "VM_ExecuteInternal_820": ("call SetVariableFieldValue",),
    "VM_ExecuteInternal_830": ("mov eax, [ebp-0x3c]", "mov eax, [ebp-0x34]", "call RemoveVariableValue"),
    "VM_ExecuteInternal_3430": ("mov eax, [gFs+0xc]", "mov eax, [ebp-0x38]", "call ClearArray"),
}
OPCODES = (("EvalArrayRef", 0x23, "VM_ExecuteInternal_3420"),
           ("ClearArray", 0x24, "VM_ExecuteInternal_3430"),
           ("SetVariableField", 0x39, "VM_ExecuteInternal_3620"))


def compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def inputs(reference: ReferenceSource) -> tuple[dict[str, str], str, str, str]:
    compiler = reference.files["src/asmsource/_scr_compiler.asm"]
    vm = reference.files["src/asmsource/_scr_vm.asm"]
    grammar = reference.files["src/asmsource/_scr_yacc.asm"]
    section = grammar.split("yyparse_jumptab_0:\n", 1)[1].split("yyparse_jumptab_1:", 1)[0]
    labels = re.findall(r"\bdd (yyparse_\d+)\s*", section)
    if len(labels) != 134:
        raise ValueError("Array write grammar inventory changed")
    productions = {"grammar_expression": 5, "grammar_group": 40, "grammar_undefined": 49}
    profiles = {}
    for name in ROLES:
        if name in productions:
            profiles[name] = assembly.block(grammar, labels[productions[name]])
        elif name.startswith("Emit"):
            profiles[name] = assembly.block(compiler, name)
        elif name.startswith("VM_"):
            profiles[name] = assembly.block(vm, name)
        else:
            profiles[name] = reference.function("src/cscr_variable.cpp", name)
    return profiles, compiler, vm, reference.files["src/scr_vm.h"]


def verify(profiles: dict[str, str], compiler: str, vm: str, header: str) -> None:
    if assembly.dispatch(compiler, "EmitStatement", 79)[2] != "EmitStatement_680":
        raise ValueError("Array assignment compiler dispatch changed")
    dispatch = assembly.dispatch(vm, "VM_ExecuteInternal", 135)
    for name, opcode, label in OPCODES:
        if dispatch[opcode] != label or not re.search(rf"\bOP_{name}\s*=\s*0x{opcode:x}\b", header):
            raise ValueError("Array write opcode/header dispatch changed")
    if not re.search(r"(?m)^VM_ExecuteInternal_3420:\n\s*mov eax, \[ebp-0x38\]\nVM_ExecuteInternal_1390:", vm):
        raise ValueError("Array reference helper fallthrough changed")
    for name, needles in ROLES.items():
        body, position = compact(profiles[name]), 0
        for needle in needles:
            needle = compact(needle)
            found = body.find(needle, position)
            if found < 0:
                raise ValueError(f"Selected array write role changed: {name}")
            position = found + len(needle)


def verify_mutations(profiles: dict[str, str], compiler: str, vm: str, header: str) -> int:
    cases = []
    for name, needles in ROLES.items():
        body = compact(profiles[name])
        for needle in needles:
            needle = compact(needle)
            if body.count(needle) != 1:
                raise ValueError(f"Array write injection needs a unique role: {name}")
            changed = dict(profiles)
            changed[name] = body.replace(needle, "mutated_array_write_role", 1)
            cases.append((changed, compiler, vm, header))
        if len(needles) > 1:
            first = compact(needles[0])
            changed = dict(profiles)
            changed[name] = body.replace(first, "", 1) + first
            cases.append((changed, compiler, vm, header))
    for name, opcode, label in OPCODES:
        cases.append((profiles, compiler, vm.replace(f"\tdd {label}\n", "\tdd VM_ExecuteInternal_40\n", 1), header))
        cases.append((profiles, compiler, vm, header.replace(f"OP_{name} = 0x{opcode:x}", f"OP_{name} = 0xff", 1)))
    cases.append((profiles, compiler.replace("\tdd EmitStatement_680\n", "\tdd EmitStatement_10\n", 1), vm, header))
    cases.append((profiles, compiler, vm.replace("VM_ExecuteInternal_1390:\n", "mutated_array_fallthrough:\n", 1), header))
    for changed in cases:
        try:
            verify(*changed)
        except ValueError:
            continue
        raise ValueError("Array write guard accepted a scoped mutation")
    return len(cases)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    source = inputs(ReferenceSource(args.reference))
    verify(*source)
    print(f"Verified selected array write roles; rejected {verify_mutations(*source)} mutations")


if __name__ == "__main__":
    main()
