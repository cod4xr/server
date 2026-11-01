# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard native empty-array allocation/emission roles, not whole-VM execution."""
import argparse
from pathlib import Path
import re

from reference_source import ReferenceSource
from script_update_reference import block


ROLES = {
    "allocation": (
        "varUsagePos = gScrVarPub.varUsagePos;",
        "if ( !gScrVarPub.varUsagePos )",
        'gScrVarPub.varUsagePos = "<script array variable>";',
        "id = AllocVariable();",
        "entryValue = &gScrVarGlob.variableList[id + VARIABLELIST_PARENT_BEGIN];",
        "entryValue->w.status = VAR_STAT_EXTERNAL;",
        "assert(VAR_TYPE(entryValue) == VAR_UNDEFINED);",
        "entryValue->w.type |= VAR_ARRAY;",
        "entryValue->u.o.refCount = 0;",
        "entryValue->u.o.u.entnum = 0;",
        "gScrVarPub.varUsagePos = varUsagePos;",
        "return id;",
    ),
    "vm": (
        "mov eax, [gFs+0xc]\n", "add eax, 0x8\n", "mov [gFs+0xc], eax\n",
        "mov dword [eax+0x4], 0x1\n", "mov ebx, [gFs+0xc]\n",
        "call Scr_AllocArray\n", "mov [ebx], eax\n", "mov ecx, [gFs]\n",
        "jmp VM_ExecuteInternal_40\n",
    ),
    "compiler": (
        "mov edi, [eax+0x4]\n", "xor ecx, ecx\n", "mov edx, 0x1\n",
        "mov eax, 0x25\n", "call EmitOpcode\n", "mov dword [esp+0x4], 0x1\n",
        "mov [esp], edi\n", "call AddOpcodePos\n", "xor eax, eax\n",
        "jmp EmitOrEvalPrimitiveExpression_20\n",
    ),
}
IDENTITIES = (("OP_EmptyArray", 0x25), ("VAR_POINTER", 1), ("VAR_ARRAY", 21))
DISPATCH = {
    "vm_table": (0x25, "VM_ExecuteInternal_3440"),
    "compiler_table": (0x42 - 0x11, "EmitOrEvalPrimitiveExpression_390"),
}


def verify(sources: dict[str, str]) -> None:
    for key, roles in ROLES.items():
        position = 0
        for role in roles:
            found = sources[key].find(role, position)
            if found < 0:
                raise ValueError("Native empty-array allocation/emission role changed")
            position = found + len(role)
    for name, value in IDENTITIES:
        if not re.search(rf"\b{name}\s*=\s*(?i:0x{value:x})\b", sources["header"]):
            raise ValueError("Native empty-array type/opcode changed")
    for key, (index, label) in DISPATCH.items():
        entries = re.findall(r"(?m)^\s*dd (\w+)\s*$", sources[key])
        if len(entries) <= index or entries[index] != label:
            raise ValueError("Native empty-array primary/opcode dispatch changed")


def verify_mutations(sources: dict[str, str]) -> int:
    cases = []
    for key, roles in ROLES.items():
        for role in roles:
            if sources[key].count(role) != 1:
                raise ValueError("Empty-array fault injection requires unique scoped roles")
            cases.append((key, sources[key].replace(role, "mutated_empty_array_role\n", 1)))
        cases.append((key, sources[key].replace(roles[0], "", 1) + roles[0]))
    for name, value in IDENTITIES:
        pattern = rf"\b{name}\s*=\s*(?i:0x{value:x})\b"
        cases.append(("header", re.sub(pattern, f"{name} = 0xff", sources["header"], count=1)))
    for key, (_, label) in DISPATCH.items():
        cases.append((key, sources[key].replace(f"dd {label}\n", "dd mutated_empty_array_dispatch\n", 1)))
    for key, changed in cases:
        try:
            verify(dict(sources, **{key: changed}))
        except ValueError:
            continue
        raise ValueError("Empty-array guard accepted an injected role change")
    return len(cases)


def inputs(reference: ReferenceSource) -> dict[str, str]:
    """Read selected bodies only after all pinned revision/hash checks succeed."""
    vm = reference.files["src/asmsource/_scr_vm.asm"]
    compiler = reference.files["src/asmsource/_scr_compiler.asm"]
    return {
        "allocation": reference.function("src/cscr_variable.cpp", "Scr_AllocArray"),
        "vm": block(vm, "VM_ExecuteInternal_3440"),
        "compiler": block(compiler, "EmitOrEvalPrimitiveExpression_390"),
        "header": reference.files["src/scr_vm.h"],
        "vm_table": block(vm, "VM_ExecuteInternal_jumptab_0"),
        "compiler_table": block(compiler, "EmitOrEvalPrimitiveExpression_jumptab_0"),
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    sources = inputs(ReferenceSource(parser.parse_args().reference))
    verify(sources)
    print(f"Verified selected native empty-array roles; rejected {verify_mutations(sources)} mutations")


if __name__ == "__main__":
    main()
