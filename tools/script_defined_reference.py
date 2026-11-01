# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard isdefined's selected archival branches and first-parameter access rules."""

import argparse
from pathlib import Path
import re

from reference_source import ReferenceSource
import script_update_reference as assembly

ROLES = {
    "GScr_IsDefined": (
        "mov dword [esp], 0x0", "call Scr_GetType", "cmp eax, 0x1",
        "jz GScr_IsDefined_10", "test eax, eax", "setnz al", "movzx eax, al",
        "mov [esp], eax", "call Scr_AddInt", "ret",
    ),
    "GScr_IsDefined_10": (
        "mov dword [esp], 0x0", "call Scr_GetPointerType", "cmp eax, 0x15",
        "jg GScr_IsDefined_20", "cmp eax, 0x13", "jz GScr_IsDefined_20",
        "mov eax, 0x1", "mov [esp], eax", "call Scr_AddInt", "ret",
    ),
    "GScr_IsDefined_20": ("xor eax, eax", "mov [esp], eax", "call Scr_AddInt", "ret"),
    "Scr_GetType": (
        "mov eax, [ebp+0x8]", "cmp [gScrVmPub+0x1c], eax", "jbe Scr_GetType_10",
        "lea edx, [eax*8]", "mov eax, [gScrVmPub+0x10]", "sub eax, edx",
        "mov eax, [eax+0x4]", "ret",
    ),
    "Scr_GetType_10": ("add eax, 0x1", "_cstring_parameter_d_does", "call va"),
}


def compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def inputs(reference: ReferenceSource) -> tuple[dict[str, str], str, str]:
    game = reference.files["src/asmsource/_g_scr_main_mp.asm"]
    vm = reference.files["src/asmsource/_scr_vm.asm"]
    return ({name: assembly.block(vm if name.startswith("Scr_GetType") else game, name)
             for name in ROLES}, reference.files["src/scr_vm_main.c"], reference.files["src/scr_vm.h"])


def verify(profiles: dict[str, str], registration: str, header: str) -> None:
    for name, needles in ROLES.items():
        body, position = compact(profiles[name]), 0
        for needle in needles:
            needle = compact(needle)
            found = body.find(needle, position)
            if found < 0:
                raise ValueError(f"Selected definedness role changed: {name}")
            position = found + len(needle)
        if "Scr_GetNumParam" in body:
            raise ValueError("Definedness gained an explicit argument-count check")
    if not re.search(r'(?m)^\s*Scr_AddFunction\("isdefined",\s*GScr_IsDefined,\s*0\s*\);', registration):
        raise ValueError("Definedness nondeveloper registration changed")
    for kind, tag in (("VAR_UNDEFINED", 0), ("VAR_POINTER", 1), ("VAR_DEAD_ENTITY", 19), ("VAR_ARRAY", 21)):
        if not re.search(rf"\b{kind}\s*=\s*0x{tag:x}\b", header):
            raise ValueError("Definedness branch tag changed")


def verify_mutations(profiles: dict[str, str], registration: str, header: str) -> int:
    cases = []
    for name, needles in ROLES.items():
        body = compact(profiles[name])
        for needle in needles:
            needle = compact(needle)
            if body.count(needle) != 1:
                raise ValueError("Definedness fault injection requires unique roles")
            changed = dict(profiles)
            changed[name] = body.replace(needle, "mutated_definedness_role", 1)
            cases.append((changed, registration, header))
        first = compact(needles[0])
        changed = dict(profiles)
        changed[name] = body.replace(first, "", 1) + first
        cases.append((changed, registration, header))
    changed = dict(profiles)
    changed["GScr_IsDefined"] += " call Scr_GetNumParam"
    cases.append((changed, registration, header))
    cases.append((profiles, registration.replace('"isdefined", GScr_IsDefined, 0', '"isdefined", GScr_IsDefined, 1', 1), header))
    for kind, tag in (("VAR_UNDEFINED", 0), ("VAR_POINTER", 1), ("VAR_DEAD_ENTITY", 19), ("VAR_ARRAY", 21)):
        cases.append((profiles, registration, header.replace(f"{kind} = 0x{tag:x}", f"{kind} = 0xff", 1)))
    for changed in cases:
        try:
            verify(*changed)
        except ValueError:
            continue
        raise ValueError("Definedness guard accepted a scoped mutation")
    return len(cases)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    source = inputs(ReferenceSource(args.reference))
    verify(*source)
    print(f"Verified selected definedness roles; rejected {verify_mutations(*source)} mutations")


if __name__ == "__main__":
    main()
