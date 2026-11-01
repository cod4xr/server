# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard ordinary pointer identity roles without claiming array comparison or native VM parity."""
import argparse
from pathlib import Path

from reference_source import ReferenceSource
import script_array_write_reference as writes
import script_update_reference as assembly

ROLES = {
    "Scr_EvalEquality": (
        "Scr_CastWeakerPair(value1, value2);", "case VAR_BEGIN_REF:",
        "value1->u.intValue + VARIABLELIST_CHILD_BEGIN", "== VAR_ARRAY",
        "value2->u.intValue + VARIABLELIST_CHILD_BEGIN", "== VAR_ARRAY",
        "&& !gScrVarPub.evaluate", "value1->type = VAR_INTEGER;",
        "equal = value1->u.intValue == value2->u.intValue;",
        "RemoveRefToObject(value1->u.intValue);", "RemoveRefToObject(value2->u.intValue);",
        "value1->u.intValue = equal;", "Scr_UnmatchingTypesError(value1, value2);"),
    "Scr_EvalInequality": ("Scr_EvalEquality(value1, value2);", "value1->u.intValue = value1->u.intValue == 0;"),
    "_Z16Scr_EvalEqualityP13VariableValueS0__140": (
        "mov edx, [esi]", "shl eax, 0x4", "mov eax, [eax+scrVarGlob+0x80028]",
        "and eax, 0x1f", "cmp eax, 0x15", "jnz _Z16Scr_EvalEqualityP13VariableValueS0__20"),
    "_Z16Scr_EvalEqualityP13VariableValueS0__20": (
        "mov eax, [edi]", "shl eax, 0x4", "mov eax, [eax+scrVarGlob+0x80028]",
        "cmp eax, 0x15", "jz _Z16Scr_EvalEqualityP13VariableValueS0__100"),
    "_Z16Scr_EvalEqualityP13VariableValueS0__30": (
        "mov dword [esi+0x4], 0x6", "cmp edx, [edi]", "setz bl",
        "call _Z17RemoveRefToObjectj", "mov eax, [edi]", "call _Z17RemoveRefToObjectj", "mov [esi], ebx"),
}


def inputs(reference: ReferenceSource) -> dict[str, str]:
    archive = reference.files["tools/cod4_dasm/src/_scr_variable.asm"]
    return {name: assembly.block(archive, name) if name.startswith("_Z") else
            reference.function("src/cscr_variable.cpp", name) for name in ROLES}


def verify(profiles: dict[str, str]) -> None:
    for name, needles in ROLES.items():
        body, position = writes.compact(profiles[name]), 0
        for needle in needles:
            needle = writes.compact(needle)
            found = body.find(needle, position)
            if found < 0:
                raise ValueError(f"Selected object comparison role changed: {name}")
            position = found + len(needle)


def mutations(profiles: dict[str, str]) -> int:
    cases = []
    for name, needles in ROLES.items():
        body = writes.compact(profiles[name])
        for needle in dict.fromkeys(needles):
            needle = writes.compact(needle)
            changed = dict(profiles)
            changed[name] = body.replace(needle, "mutated_object_role")
            cases.append(changed)
        changed = dict(profiles)
        first = writes.compact(needles[0])
        changed[name] = body.replace(first, "", 1) + first
        cases.append(changed)
    for changed in cases:
        try:
            verify(changed)
        except ValueError:
            continue
        raise ValueError("Object comparison guard accepted a scoped mutation")
    return len(cases)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    profiles = inputs(ReferenceSource(args.reference))
    verify(profiles)
    print(f"Verified selected pointer identity roles; rejected {mutations(profiles)} mutations")


if __name__ == "__main__":
    main()
