# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected native script root roles; does not execute compiler/VM/lifetimes."""
import argparse
from pathlib import Path
import re

from reference_source import ReferenceSource
from reference_source import declaration
from script_update_reference import block


C_ROLES = {
    "Scr_VM_Init": ("src/cscr_vm.cpp", (
        "gScrVarPub.levelId = 0;", "gScrVarPub.gameId = 0;", "gScrVarPub.animId = 0;")),
    "Scr_InitSystem": ("src/cscr_vm.cpp", (
        "gScrVarPub.timeArrayId = AllocObject();", "gScrVarPub.pauseArrayId = Scr_AllocArray();",
        "gScrVarPub.levelId = AllocObject();", "gScrVarPub.animId = AllocObject();",
        "gScrVarPub.time = 0;", "g_script_error_level = -1;")),
    "Scr_AllocGameVariable": ("src/cscr_variable.cpp", (
        "if ( !gScrVarPub.gameId )", "gScrVarPub.gameId = AllocValue();",
        "SetEmptyArray(gScrVarPub.gameId);", "gScrVarPub.varUsagePos = 0;", "Scr_UpdateDebugger();")),
    "Scr_GetSelf": ("src/cscr_variable.cpp", (
        "assert((gScrVarGlob.variableList[VARIABLELIST_PARENT_BEGIN + threadId].w.status & VAR_STAT_MASK) != VAR_STAT_FREE);",
        ">= VAR_THREAD", "<= VAR_CHILD_THREAD",
        "return gScrVarGlob.variableList[threadId + VARIABLELIST_PARENT_BEGIN].u.o.u.self;")),
    "AllocThread": ("src/cscr_variable.cpp", (
        "id = AllocVariable();", "id + VARIABLELIST_PARENT_BEGIN", "VAR_STAT_EXTERNAL;",
        "entryValue->w.type |= VAR_THREAD;", "entryValue->u.o.refCount = 0;",
        "entryValue->u.o.u.self = self;")),
    "AllocChildThread": ("src/cscr_variable.cpp", (
        "id = AllocVariable();", "id + VARIABLELIST_PARENT_BEGIN", "VAR_STAT_EXTERNAL;",
        "entryValue->w.type |= VAR_CHILD_THREAD;", "parentLocalId << VAR_NAME_BITS",
        "entryValue->u.o.u.self = self;")),
    "Scr_ClearThread": ("src/cscr_variable.cpp", (
        "parentId + VARIABLELIST_PARENT_BEGIN", ">= VAR_THREAD", "<= VAR_CHILD_THREAD",
        "if ( parentValue->nextSibling )", "ClearObjectInternal(parentId);",
        "RemoveRefToObject(parentValue->u.o.u.self);")),
    "Scr_StopThread": ("src/cscr_variable.cpp", (
        "Scr_ClearThread(threadId);", "u.o.u.self = gScrVarPub.levelId;",
        "AddRefToObject(gScrVarPub.levelId);")),
    "Scr_EvalVariable": ("src/cscr_variable.cpp", (
        "id + VARIABLELIST_CHILD_BEGIN", "value.type = VAR_TYPE(entryValue);",
        "value.u.intValue = entryValue->u.u.intValue;", "assert(!IsObject( entryValue ));",
        "AddRefToValue(value.type, value.u);", "return value;")),
    "Scr_EvalVariableExtern": ("src/cscr_variable.cpp", ("return Scr_EvalVariable(id);",)),
}
VM_ROLES = {
    "VM_ExecuteInternal_3200": ("mov eax, [eax+0x20]\n", "mov [ebp-0x34], eax\n"),
    "VM_ExecuteInternal_3210": ("mov eax, [eax+0x28]\n", "mov [ebp-0x34], eax\n"),
    "VM_ExecuteInternal_3220": (
        "mov dword [eax+0x4], 0x1\n", "mov eax, [gFs+0x4]\n", "call Scr_GetSelf\n", "mov [ebx], eax\n"),
    "VM_ExecuteInternal_1430": ("mov eax, [eax]\n", "call AddRefToObject\n"),
    "VM_ExecuteInternal_3230": (
        "mov dword [eax+0x4], 0x1\n", "mov edx, [ecx+0x20]\n", "mov [eax], edx\n",
        "mov eax, [ecx+0x20]\n", "call AddRefToObject\n"),
    "VM_ExecuteInternal_3240": ("lea eax, [ebx+0x8]\n", "mov eax, [eax+0x24]\n"),
    "VM_ExecuteInternal_1140": (
        "call Scr_EvalVariableExtern\n", "mov [ebx+0x8], eax\n", "mov [ebx+0xc], edx\n"),
    "VM_ExecuteInternal_3250": (
        "mov dword [eax+0x4], 0x1\n", "mov edx, [ecx+0x28]\n", "mov [eax], edx\n",
        "mov eax, [ecx+0x28]\n", "call AddRefToObject\n"),
    "VM_ExecuteInternal_3270": (
        "mov eax, [eax+0x24]\n", "mov [ebp-0x38], eax\n", "jmp VM_ExecuteInternal_1120\n"),
    "VM_ExecuteInternal_3450": (
        "call Scr_GetSelf\n", "mov [ebp-0x34], eax\n", "call IsFieldObject\n"),
}
COMPILER_ROLES = {
    "EmitOrEvalPrimitiveExpression_330": ("mov edx, 0x1\n", "mov eax, 0xf\n"),
    "EmitOrEvalPrimitiveExpression_340": ("mov edx, 0x1\n", "mov eax, 0x10\n", "jmp EmitOrEvalPrimitiveExpression_60\n"),
    "EmitOrEvalPrimitiveExpression_350": ("mov edx, 0x1\n", "mov eax, 0x11\n", "jmp EmitOrEvalPrimitiveExpression_60\n"),
    "EmitOrEvalPrimitiveExpression_360": ("mov edx, 0x1\n", "mov eax, 0x12\n", "jmp EmitOrEvalPrimitiveExpression_60\n"),
    "EmitOrEvalPrimitiveExpression_60": ("call EmitOpcode\n", "call AddOpcodePos\n"),
}
OPCODES = {
    "OP_GetLevelObject": (0xD, "VM_ExecuteInternal_3200"),
    "OP_GetAnimObject": (0xE, "VM_ExecuteInternal_3210"),
    "OP_GetSelf": (0xF, "VM_ExecuteInternal_3220"),
    "OP_GetLevel": (0x10, "VM_ExecuteInternal_3230"),
    "OP_GetGame": (0x11, "VM_ExecuteInternal_3240"),
    "OP_GetAnim": (0x12, "VM_ExecuteInternal_3250"),
    "OP_GetGameRef": (0x14, "VM_ExecuteInternal_3270"),
    "OP_GetSelfObject": (0x26, "VM_ExecuteInternal_3450"),
}
PRIMARY_TAGS = ((0x20, "330"), (0x22, "340"), (0x23, "350"), (0x24, "360"))
TYPES = (("VAR_POINTER", 1), ("VAR_THREAD", 14), ("VAR_CHILD_THREAD", 17),
         ("VAR_OBJECT", 18), ("VAR_ARRAY", 21))
ABI_ROLES = ('"call Scr_EvalVariable\\n"', '"mov -0x14(%ebp), %eax\\n"',
             '"mov -0x10(%ebp), %edx\\n"')
FALLTHROUGHS = (
    ("vm", "VM_ExecuteInternal_3220", "VM_ExecuteInternal_1430"),
    ("vm", "VM_ExecuteInternal_3240", "VM_ExecuteInternal_1140"),
    ("compiler", "EmitOrEvalPrimitiveExpression_330", "EmitOrEvalPrimitiveExpression_60"),
)


def ordered(body: str, needles: tuple[str, ...]) -> None:
    position = 0
    for needle in needles:
        found = body.find(needle, position)
        if found < 0:
            raise ValueError("Selected native root role changed")
        position = found + len(needle)


def verify(sources: dict[str, str]) -> None:
    for name, (_, needles) in C_ROLES.items():
        ordered(sources[name], needles)
    ordered(sources["variable_abi"], ABI_ROLES)
    for name, value in TYPES:
        if not re.search(rf"\b{name}\s*=\s*(?i:0x{value:x})\b", sources["header"]):
            raise ValueError("Native root type identity changed")
    for key, roles in (("vm", VM_ROLES), ("compiler", COMPILER_ROLES)):
        for label, needles in roles.items():
            ordered(block(sources[key], label), needles)
    vm = re.findall(r"(?m)^\s*dd (\w+)\s*$", block(sources["vm"], "VM_ExecuteInternal_jumptab_0"))
    for name, (value, label) in OPCODES.items():
        if not re.search(rf"\b{name}\s*=\s*(?i:0x{value:x})\b", sources["header"]):
            raise ValueError("Native root opcode identity changed")
        if len(vm) <= value or vm[value] != label:
            raise ValueError("Native root opcode dispatch changed")
    compiler = re.findall(r"(?m)^\s*dd (\w+)\s*$", block(sources["compiler"], "EmitOrEvalPrimitiveExpression_jumptab_0"))
    for tag, suffix in PRIMARY_TAGS:
        index = tag - 0x11
        if len(compiler) <= index or compiler[index] != f"EmitOrEvalPrimitiveExpression_{suffix}":
            raise ValueError("Native root primary dispatch changed")
    for key, label, following in FALLTHROUGHS:
        expected = f"{label}:\n{block(sources[key], label)}{following}:\n"
        if expected not in sources[key]:
            raise ValueError("Native root shared-tail fallthrough changed")


def verify_mutations(sources: dict[str, str]) -> int:
    cases = []
    roles = [(name, None, needles) for name, (_, needles) in C_ROLES.items()]
    roles.append(("variable_abi", None, ABI_ROLES))
    roles += [(key, label, needles) for key, table in (("vm", VM_ROLES), ("compiler", COMPILER_ROLES))
              for label, needles in table.items()]
    for key, label, needles in roles:
        original = sources[key] if label is None else block(sources[key], label)
        prefix = "" if label is None else f"{label}:\n"
        for needle in needles:
            if original.count(needle) != 1:
                raise ValueError("Root fault injection requires unique scoped roles")
            changed = original.replace(needle, "mutated_root_role\n", 1)
            cases.append((key, sources[key].replace(prefix + original, prefix + changed, 1)))
        if len(needles) > 1:
            changed = original.replace(needles[0], "", 1) + needles[0]
            cases.append((key, sources[key].replace(prefix + original, prefix + changed, 1)))
    for name, (value, label) in OPCODES.items():
        pattern = rf"\b{name}\s*=\s*(?i:0x{value:x})\b"
        cases.append(("header", re.sub(pattern, f"{name} = 0xff", sources["header"], count=1)))
        table = block(sources["vm"], "VM_ExecuteInternal_jumptab_0")
        changed = table.replace(f"dd {label}\n", "dd mutated_root_dispatch\n", 1)
        cases.append(("vm", sources["vm"].replace(table, changed, 1)))
    for name, value in TYPES:
        pattern = rf"\b{name}\s*=\s*(?i:0x{value:x})\b"
        cases.append(("header", re.sub(pattern, f"{name} = 0xff", sources["header"], count=1)))
    for key, label, following in FALLTHROUGHS:
        original = f"{label}:\n{block(sources[key], label)}{following}:\n"
        changed = original.replace(f"{following}:\n", f"mutated_fallthrough:\n{following}:\n")
        cases.append((key, sources[key].replace(original, changed, 1)))
    for _, suffix in PRIMARY_TAGS:
        table = block(sources["compiler"], "EmitOrEvalPrimitiveExpression_jumptab_0")
        changed = table.replace(f"dd EmitOrEvalPrimitiveExpression_{suffix}\n", "dd mutated_root_primary\n", 1)
        cases.append(("compiler", sources["compiler"].replace(table, changed, 1)))
    for key, changed in cases:
        mutated = dict(sources, **{key: changed})
        try:
            verify(mutated)
        except ValueError:
            continue
        raise ValueError("Root guard accepted an injected role change")
    return len(cases)


def inputs(reference: ReferenceSource) -> dict[str, str]:
    """Extract selected roles only after the complete pinned-source hash check."""
    sources = {name: reference.function(filename, name) for name, (filename, _) in C_ROLES.items()}
    sources.update(vm=reference.files["src/asmsource/_scr_vm.asm"],
                   compiler=reference.files["src/asmsource/_scr_compiler.asm"],
                   header=reference.files["src/scr_vm.h"])
    sources["variable_abi"] = declaration(reference.files["src/cscr_variable.cpp"],
        r'__asm__\(/\* Scr_EvalVariable ABI Wrapper \*/.*?"ret\\n"\);')
    return sources


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    sources = inputs(ReferenceSource(args.reference))
    verify(sources)
    print(f"Verified selected native root roles; rejected {verify_mutations(sources)} mutations")


if __name__ == "__main__":
    main()
