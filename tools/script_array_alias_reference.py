# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected native excess-reference/COW roles; no collector or VM conformance claim."""
import argparse
from pathlib import Path

from reference_source import ReferenceSource
import script_array_reference as arrays
import script_array_write_reference as writes

ROLES = {
    "AddRefToObject": (
        "++gScrVarPub.totalObjectRefCount;", "entryValue = &gScrVarGlob.variableList[id + VARIABLELIST_PARENT_BEGIN];",
        "++entryValue->u.o.refCount;", "assert(entryValue->u.o.refCount > 0);"),
    "RemoveRefToObject": (
        "entryValue = &gScrVarGlob.variableList[VARIABLELIST_PARENT_BEGIN+ id];",
        "if ( entryValue->u.o.refCount )", "--gScrVarPub.totalObjectRefCount;",
        "!--entryValue->u.o.refCount && VAR_TYPE(entryValue) == VAR_ENTITY && !entryValue->nextSibling",
        "else", "if ( entryValue->nextSibling )", "ClearObject(id);", "FreeVariable(id);"),
}


def inputs(reference: ReferenceSource) -> dict[str, str]:
    return {name: reference.function("src/cscr_variable.cpp", name) for name in ROLES}


def verify(profiles: dict[str, str]) -> None:
    for name, needles in ROLES.items():
        body, position = writes.compact(profiles[name]), 0
        for needle in needles:
            needle = writes.compact(needle)
            found = body.find(needle, position)
            if found < 0:
                raise ValueError(f"Selected array alias role changed: {name}")
            position = found + len(needle)


def mutations(profiles: dict[str, str]) -> int:
    changed = []
    for name, needles in ROLES.items():
        body = writes.compact(profiles[name])
        for needle in needles:
            candidate = dict(profiles)
            candidate[name] = body.replace(writes.compact(needle), "mutated_alias_role")
            changed.append(candidate)
        first = writes.compact(needles[0])
        candidate = dict(profiles)
        candidate[name] = body.replace(first, "", 1) + first
        changed.append(candidate)
    for candidate in changed:
        try:
            verify(candidate)
        except ValueError:
            continue
        raise ValueError("Array alias guard accepted a scoped mutation")
    return len(changed)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    arrays.verify(*arrays.inputs(reference))
    profiles = inputs(reference)
    verify(profiles)
    print(f"Verified selected alias/ref roles; rejected {mutations(profiles)} mutations")


if __name__ == "__main__":
    main()
