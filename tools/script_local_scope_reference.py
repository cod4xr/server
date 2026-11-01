# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected scope-removal emission/dispatch roles; no native VM execution."""
import argparse
from pathlib import Path
import re

from reference_source import ReferenceSource
from script_update_reference import block, dispatch

EMISSION = {
    "EmitIfStatement_30": ("sub ebx, [esi+0x8]\n", "mov [eax], bl\n", "esi"),
    "EmitIfElseStatement_350": ("sub eax, [ebx+0x8]\n", "mov [eax], dl\n", "ebx"),
    "EmitIfElseStatement_170": (None, "mov [eax], dl\n", "ebx"),
    "EmitBreakStatement_40": ("sub edi, [edx+0x8]\n", "mov [eax], dl\n", "ebx"),
    "EmitContinueStatement_40": ("sub edi, [ebx+0x8]\n", "mov [eax], dl\n", "ebx"),
    "EmitSwitchStatement_120": ("sub edi, [ebx+0x8]\n", "mov [eax], dl\n", "ebx"),
    "EmitSwitchStatement_320": ("sub esi, [ebx+0x8]\n", "mov [eax], dl\n", "ebx"),
    "EmitDeveloperStatementList_220": (None, "mov [eax], bl\n", "esi"),
}
ROLES = {
    label: tuple(role for role in (
        difference, "mov eax, 0x17\n", "call EmitOpcode\n",
        "mov dword [esp], 0x1\n", "call TempMalloc\n",
        "mov [scrCompileGlob], eax\n", byte_store,
        f"mov eax, [{register}+0x8]\n", f"mov [{register}+0x4], eax\n",
    ) if role is not None)
    for label, (difference, byte_store, register) in EMISSION.items()
}
ROLES.update({
    "VM_ExecuteInternal_3300": (
        "movzx esi, byte [edx]\n", "lea eax, [edx*4]\n",
        "sub [gScrVmPub], eax\n", "sub [gFs+0x8], edx\n",
        "test al, al\n", "jz VM_ExecuteInternal_810\n", "xor ebx, ebx\n",
    ),
    "VM_ExecuteInternal_1130": (
        "mov eax, [gFs+0x4]\n", "mov [esp], eax\n", "call RemoveNextVariable\n",
        "sub bl, 0x1\n", "mov eax, esi\n", "add al, bl\n",
        "jnz VM_ExecuteInternal_1130\n", "jmp VM_ExecuteInternal_810\n",
    ),
})


def inputs(reference: ReferenceSource) -> dict[str, str]:
    compiler = reference.files["src/asmsource/_scr_compiler.asm"]
    vm = reference.files["src/asmsource/_scr_vm.asm"]
    return {
        **{label: block(compiler if label in EMISSION else vm, label) for label in ROLES},
        "header": reference.files["src/scr_vm.h"],
        "dispatch": block(vm, "VM_ExecuteInternal_jumptab_0"),
        "inventory": "\n".join(dispatch(vm, "VM_ExecuteInternal", 135)),
    }


def verify(sources: dict[str, str]) -> None:
    for label, roles in ROLES.items():
        position = 0
        for role in roles:
            found = sources[label].find(role, position)
            if found < 0:
                raise ValueError(f"Native local-removal role changed: {label}")
            position = found + len(role)
    if not re.search(r"\bOP_RemoveLocalVariables\s*=\s*0x17\b", sources["header"]):
        raise ValueError("Native local-removal opcode changed")
    entries = re.findall(r"(?m)^\s*dd (\w+)\s*$", sources["dispatch"])
    if len(entries) != 135 or entries[0x17] != "VM_ExecuteInternal_3300":
        raise ValueError("Native local-removal dispatch changed")
    if sources["inventory"].splitlines() != entries:
        raise ValueError("Native local-removal dispatch inventory changed")


def mutations(sources: dict[str, str]) -> int:
    cases = []
    for label, roles in ROLES.items():
        for role in roles:
            if sources[label].count(role) != 1:
                raise ValueError("Scope fault injection requires unique scoped roles")
            cases.append((label, sources[label].replace(role, "mutated_scope_role\n", 1)))
        cases.append((label, sources[label].replace(roles[0], "", 1) + roles[0]))
    cases.extend((
        ("header", sources["header"].replace("OP_RemoveLocalVariables = 0x17", "OP_RemoveLocalVariables = 0xff")),
        ("dispatch", sources["dispatch"].replace("dd VM_ExecuteInternal_3300\n", "dd wrong_scope_dispatch\n")),
        ("inventory", ""),
    ))
    for label, changed in cases:
        try:
            verify(dict(sources, **{label: changed}))
        except ValueError:
            continue
        raise ValueError("Native scope guard accepted an injected role change")
    return len(cases)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    sources = inputs(ReferenceSource(parser.parse_args().reference))
    verify(sources)
    print(f"Verified selected local-removal roles; rejected {mutations(sources)} mutations")


if __name__ == "__main__":
    main()
