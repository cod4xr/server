# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard struct field names/read/write/delete roles; not native compiler/VM execution."""
import argparse
from pathlib import Path
import re

from reference_source import ReferenceSource
import script_array_write_reference as writes
import script_update_reference as assembly

ROLES = {
    "Scr_FindVariableField": ("parentId + VARIABLELIST_PARENT_BEGIN", "id = FindVariable(parentId, name);",
                              "if ( id )", "return Scr_EvalVariable(id);", "VAR_TYPE(entryValue) == VAR_ENTITY",
                              "return Scr_EvalVariableEntityField(parentId, name);", "nret.u.intValue = 0;",
                              "nret.type = VAR_UNDEFINED;", "return nret;"),
    "Scr_GetVariableFieldIndex": ("if ( type <= VAR_OBJECT )", "return GetVariableIndexInternal(parentId, name);",
                                  "if ( type == VAR_ENTITY )", "index = FindVariableIndexInternal(parentId, name);",
                                  "gScrVarPub.entId = parentId;", "gScrVarPub.entFieldName = name;",
                                  'Scr_Error(va("cannot set field of %s", var_typename[type]));'),
    "ClearVariableField": ("if ( FindVariableIndexInternal(parentId, name) )", "RemoveVariable(parentId, name);",
                            "VAR_TYPE(parentValue) == VAR_ENTITY", "++value;", "value->type = VAR_UNDEFINED;",
                            "SetEntityFieldValue("),
    "Scr_EvalFieldObject": ("type = value->type;", "if(type == VAR_POINTER)",
                             "value->u.intValue + VARIABLELIST_PARENT_BEGIN", "if(type < VAR_ARRAY)",
                             "SetVariableValue(tempVariable, &tempValue);", "return tempValue.u.intValue;",
                             "RemoveRefToValue(value->type, value->u);",
                             'Scr_Error(va("%s is not a field object", var_typename[type]));'),
    "Scr_EvalVariableObject": ("id + VARIABLELIST_CHILD_BEGIN", "if(type == VAR_POINTER)",
                                "entryValue->u.u.intValue + VARIABLELIST_PARENT_BEGIN", "if(type < VAR_ARRAY)",
                                "return entryValue->u.u.intValue;",
                                'Scr_Error(va("%s is not a field object", var_typename[type]));'),
    "grammar_field": ("call SL_ConvertToLowercase", "mov [ebx], eax", "mov ecx, [ebx]",
                       "mov edi, [ebx-0x10]", "mov dword [esp], 0xf", "call node3"),
    "EmitVariableExpression_10": ("call EmitPrimitiveExpressionFieldObject", "mov eax, 0x2a", "call EmitOpcode"),
    "EmitVariableExpressionRef_10": ("call EmitPrimitiveExpressionFieldObject", "mov eax, 0x2e", "call EmitOpcode"),
    "EmitStatement_80": ("cmp dword [eax], 0x1f", "jnz EmitStatement_20", "cmp eax, 0xf", "jz EmitStatement_30"),
    "EmitStatement_30": ("call EmitPrimitiveExpressionFieldObject", "mov eax, 0x2f", "call EmitOpcode"),
    "EmitStatement_660": ("call SL_TransferToCanonicalString", "mov [ebx], ax"),
    "VM_ExecuteInternal_3490": ("movzx edx, word [eax]", "mov eax, [ebp-0x34]", "call Scr_FindVariableFieldExtern",
                                "mov [ebx+0x8], eax", "mov [ebx+0xc], edx"),
    "VM_ExecuteInternal_1150": ("call Scr_GetVariableFieldIndex", "mov [ebp-0x3c], eax", "call Scr_GetVarId", "mov [ebp-0x38], eax"),
    "VM_ExecuteInternal_3530": ("mov eax, [ebp-0x34]", "call ClearVariableField"),
    "VM_ExecuteInternal_3760": ("call Scr_EvalFieldObject", "mov [ebp-0x34], eax", "jmp VM_ExecuteInternal_800"),
    "VM_ExecuteInternal_3770": ("call Scr_EvalVariableObject", "mov [ebp-0x34], eax"),
}
OPCODES = (("EvalFieldVariable", 0x2a, "VM_ExecuteInternal_3490"),
           ("EvalFieldVariableRef", 0x2e, "VM_ExecuteInternal_1150"),
           ("ClearFieldVariable", 0x2f, "VM_ExecuteInternal_3530"),
           ("CastFieldObject", 0x59, "VM_ExecuteInternal_3760"),
           ("EvalLocalVariableObjectCached", 0x5a, "VM_ExecuteInternal_3770"))


def inputs(reference: ReferenceSource) -> tuple[dict[str, str], str, str]:
    compiler = reference.files["src/asmsource/_scr_compiler.asm"]
    vm = reference.files["src/asmsource/_scr_vm.asm"]
    grammar = reference.files["src/asmsource/_scr_yacc.asm"]
    section = grammar.split("yyparse_jumptab_0:\n", 1)[1].split("yyparse_jumptab_1:", 1)[0]
    labels = re.findall(r"\bdd (yyparse_\d+)\s*", section)
    if len(labels) != 134:
        raise ValueError("Field grammar inventory changed")
    profiles = {}
    for name in ROLES:
        if name == "grammar_field":
            profiles[name] = assembly.block(grammar, labels[62])
        elif name.startswith("Emit"):
            profiles[name] = assembly.block(compiler, name)
        elif name.startswith("VM_"):
            profiles[name] = assembly.block(vm, name)
        else:
            profiles[name] = reference.function("src/cscr_variable.cpp", name)
    return profiles, vm, reference.files["src/scr_vm.h"]


def verify(profiles: dict[str, str], vm: str, header: str) -> None:
    for name, needles in ROLES.items():
        body, position = writes.compact(profiles[name]), 0
        for needle in needles:
            needle = writes.compact(needle)
            found = body.find(needle, position)
            if found < 0:
                raise ValueError(f"Selected field role changed: {name}")
            position = found + len(needle)
    dispatch = assembly.dispatch(vm, "VM_ExecuteInternal", 135)
    for name, opcode, label in OPCODES:
        if dispatch[opcode] != label or not re.search(rf"\bOP_{name}\s*=\s*0x{opcode:x}\b", header, re.IGNORECASE):
            raise ValueError("Field opcode/header dispatch changed")


def mutations(profiles: dict[str, str], vm: str, header: str) -> int:
    cases = []
    for name, needles in ROLES.items():
        body = writes.compact(profiles[name])
        for needle in needles:
            needle = writes.compact(needle)
            if body.count(needle) != 1:
                raise ValueError("Field mutation requires a unique role")
            changed = dict(profiles)
            changed[name] = body.replace(needle, "mutated_field_role", 1)
            cases.append((changed, vm, header))
        first = writes.compact(needles[0])
        changed = dict(profiles)
        changed[name] = body.replace(first, "", 1) + first
        cases.append((changed, vm, header))
    for name, opcode, label in OPCODES:
        cases.append((profiles, vm.replace(f"\tdd {label}\n", "\tdd VM_ExecuteInternal_40\n", 1), header))
        changed = re.sub(rf"(\bOP_{name}\s*=\s*)0x{opcode:x}\b", r"\g<1>0xff", header,
                         count=1, flags=re.IGNORECASE)
        cases.append((profiles, vm, changed))
    for changed in cases:
        try:
            verify(*changed)
        except ValueError:
            continue
        raise ValueError("Field guard accepted a scoped mutation")
    return len(cases)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    writes.verify(*writes.inputs(reference))
    source = inputs(reference)
    verify(*source)
    print(f"Verified selected field roles; rejected {mutations(*source)} mutations")


if __name__ == "__main__":
    main()
