# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected string access and unary size emission roles, not compiler/VM execution."""

import argparse
from pathlib import Path
import re

from reference_source import ReferenceSource
import script_update_reference as assembly

ROLES = {
    "string_read": (
        "if ( index->type == VAR_INTEGER )",
        "index->u.intValue < 0 || (s = SL_ConvertToString(value->u.intValue), index->u.intValue >= (signed int)strlen(s))",
        'Scr_Error(va("string index %d out of range", index->u.intValue));',
        "index->type = VAR_STRING;", "c[0] = s[index->u.intValue];", "c[1] = 0;",
        "index->u.intValue = SL_GetStringOfSize(c, 0, 2u, 15);",
        "SL_RemoveRefToString(value->u.intValue);",
        'Scr_Error(va("%s is not a string index", var_typename[index->type]));',
    ),
    "size": (
        "if ( value->type == VAR_POINTER )",
        "entryValue = &gScrVarGlob.variableList[value->u.intValue + VARIABLELIST_PARENT_BEGIN];",
        "if ( VAR_TYPE(entryValue) == VAR_ARRAY )",
        "value->u.intValue = entryValue->u.o.u.size;", "value->u.intValue = 1;",
        "RemoveRefToObject(id);", "else if ( value->type == VAR_STRING ) { value->type = VAR_INTEGER;",
        "stringValue = value->u.intValue;",
        "value->u.intValue = strlen(SL_ConvertToString(value->u.intValue));",
        "SL_RemoveRefToString(stringValue);", 'va("size cannot be applied to %s", var_typename[value->type])',
        "RemoveRefToValue(value->type, value->u);", "value->type = VAR_UNDEFINED;",
        "Scr_Error(error_message);",
    ),
    "size_operand": (
        "mov edx, [eax+0x8]", "mov edi, [eax+0x4]", "mov eax, edi",
        "call EmitOrEvalPrimitiveExpression_150", "jnz EmitOrEvalPrimitiveExpression_160",
    ),
    "size_emit": ("mov eax, 0x76", "call EmitOpcode", "call AddOpcodePos"),
    "size_constant": ("call EmitValue", "jmp EmitOrEvalPrimitiveExpression_210"),
    "size_vm": ("mov eax, [gFs+0xc]", "mov [esp], eax", "call Scr_EvalSizeValue",
                "jmp VM_ExecuteInternal_40"),
}


def compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def inputs(reference: ReferenceSource) -> tuple[dict[str, str], str, str, str]:
    read = reference.function("src/cscr_variable.cpp", "Scr_EvalArray")
    match = re.search(r"(?ms)^    case VAR_STRING:\n(.*?)(?=^    case VAR_VECTOR:)", read)
    if not match:
        raise ValueError("Missing native string index branch")
    compiler = reference.files["src/asmsource/_scr_compiler.asm"]
    vm = reference.files["src/asmsource/_scr_vm.asm"]
    return ({
        "string_read": match[1], "size": reference.function("src/cscr_variable.cpp", "Scr_EvalSizeValue"),
        "size_operand": assembly.block(compiler, "EmitOrEvalPrimitiveExpression_380"),
        "size_emit": assembly.block(compiler, "EmitOrEvalPrimitiveExpression_210"),
        "size_constant": assembly.block(compiler, "EmitOrEvalPrimitiveExpression_160"),
        "size_vm": assembly.block(vm, "VM_ExecuteInternal_4020"),
    }, compiler, vm, reference.files["src/scr_vm.h"])


def verify(profiles: dict[str, str], compiler: str, vm: str, header: str) -> None:
    if assembly.dispatch(compiler, "EmitOrEvalPrimitiveExpression", 59)[0x34 - 0x11] != "EmitOrEvalPrimitiveExpression_380":
        raise ValueError("Size compiler dispatch changed")
    if assembly.dispatch(vm, "VM_ExecuteInternal", 135)[0x76] != "VM_ExecuteInternal_4020":
        raise ValueError("Size VM dispatch changed")
    if not re.search(r"\bOP_size\s*=\s*0x76\b", header):
        raise ValueError("Size opcode changed")
    for name, needles in ROLES.items():
        body, position = compact(profiles[name]), 0
        for needle in needles:
            found = body.find(compact(needle), position)
            if found < 0:
                raise ValueError(f"Selected string access role changed: {name}")
            position = found + len(compact(needle))


def verify_mutations(profiles: dict[str, str], compiler: str, vm: str, header: str) -> int:
    cases = []
    for name, needles in ROLES.items():
        body = compact(profiles[name])
        for needle in needles:
            needle = compact(needle)
            if body.count(needle) != 1:
                raise ValueError(f"String access fault injection needs a unique role: {name}")
            changed = dict(profiles)
            changed[name] = body.replace(needle, "mutated_string_access_role", 1)
            cases.append((changed, compiler, vm, header))
        first = compact(needles[0])
        changed = dict(profiles)
        changed[name] = body.replace(first, "", 1) + first
        cases.append((changed, compiler, vm, header))
    cases.extend((
        (profiles, compiler.replace("\tdd EmitOrEvalPrimitiveExpression_380\n", "\tdd EmitOrEvalPrimitiveExpression_10\n", 1), vm, header),
        (profiles, compiler, vm.replace("\tdd VM_ExecuteInternal_4020\n", "\tdd VM_ExecuteInternal_40\n", 1), header),
        (profiles, compiler, vm, header.replace("OP_size = 0x76", "OP_size = 0x75", 1)),
    ))
    for changed in cases:
        try:
            verify(*changed)
        except ValueError:
            continue
        raise ValueError("String access guard accepted a scoped mutation")
    return len(cases)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    source = inputs(ReferenceSource(args.reference))
    verify(*source)
    print(f"Verified selected string access roles; rejected {verify_mutations(*source)} mutations")


if __name__ == "__main__":
    main()
