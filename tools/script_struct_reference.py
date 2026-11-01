# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected native struct constructor roles, not whole VM allocation/cleanup."""
import argparse
from pathlib import Path
import re

from reference_source import ReferenceSource

ROLES = {
    "AllocObject": ("id = AllocVariable();", "id + VARIABLELIST_PARENT_BEGIN",
                    "entryValue->w.status = VAR_STAT_EXTERNAL;", "entryValue->w.type |= VAR_OBJECT;",
                    "entryValue->u.o.refCount = 0;", "return id;"),
    "Scr_AddObject": ("assert(id != 0);", "IncInParam();", "gScrVmPub.top->type = VAR_POINTER;",
                      "gScrVmPub.top->u.intValue = id;", "AddRefToObject(id);"),
    "Scr_AddStruct": ("id = AllocObject();", "Scr_AddObject(id );", "RemoveRefToObject(id);"),
}


def compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def inputs(reference: ReferenceSource) -> tuple[dict[str, str], str, str]:
    return ({name: reference.function("src/cscr_variable.cpp" if name == "AllocObject" else
                                     "src/cscr_vm.cpp", name) for name in ROLES},
            reference.files["src/scr_vm_main.c"], reference.files["src/scr_vm.h"])


def verify(functions: dict[str, str], registration: str, header: str) -> None:
    for name, needles in ROLES.items():
        body, position = compact(functions[name]), 0
        for needle in needles:
            needle = compact(needle)
            found = body.find(needle, position)
            if found < 0:
                raise ValueError(f"Selected native struct constructor role changed: {name}")
            position = found + len(needle)
        if "Scr_GetNumParam" in body or "Scr_GetType" in body:
            raise ValueError("Struct constructor gained argument inspection")
    if not re.search(r'(?m)^\s*Scr_AddFunction\("spawnstruct",\s*Scr_AddStruct,\s*0\s*\);', registration):
        raise ValueError("Struct constructor registration changed")
    for name, value in (("VAR_POINTER", 1), ("VAR_OBJECT", 18)):
        if not re.search(rf"\b{name}\s*=\s*0x{value:x}\b", header):
            raise ValueError("Struct constructor value tags changed")


def mutations(functions: dict[str, str], registration: str, header: str) -> int:
    cases = []
    for name, needles in ROLES.items():
        body = compact(functions[name])
        for needle in needles:
            needle = compact(needle)
            if body.count(needle) != 1:
                raise ValueError("Constructor role mutation is not unique")
            changed = dict(functions)
            changed[name] = body.replace(needle, "mutated_constructor_role", 1)
            cases.append((changed, registration, header))
        first = compact(needles[0])
        changed = dict(functions)
        changed[name] = body.replace(first, "", 1) + first
        cases.append((changed, registration, header))
    for call in ("Scr_GetNumParam", "Scr_GetType"):
        changed = dict(functions)
        changed["Scr_AddStruct"] += f" {call}();"
        cases.append((changed, registration, header))
    cases.append((functions, registration.replace('"spawnstruct", Scr_AddStruct, 0', '"spawnstruct", Scr_AddStruct, 1', 1), header))
    for name, value in (("VAR_POINTER", 1), ("VAR_OBJECT", 18)):
        cases.append((functions, registration, header.replace(f"{name} = 0x{value:x}", f"{name} = 0xff", 1)))
    for changed in cases:
        try:
            verify(*changed)
        except ValueError:
            continue
        raise ValueError("Struct constructor guard accepted a scoped mutation")
    return len(cases)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    source = inputs(ReferenceSource(args.reference))
    verify(*source)
    print(f"Verified native struct constructor roles; rejected {mutations(*source)} mutations")


if __name__ == "__main__":
    main()
