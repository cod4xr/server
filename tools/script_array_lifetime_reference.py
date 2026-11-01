# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected native zero-owner clear/free paths; no allocator/VM/GC parity claim."""
import argparse
from pathlib import Path

from reference_source import ReferenceSource
import script_array_alias_reference as aliases
import script_array_write_reference as writes

ROLES = {
    **aliases.ROLES,
    "RemoveRefToValue": (
        "value = type - VAR_BEGIN_REF;", "(unsigned int)value < 4",
        "value == VAR_POINTER - VAR_BEGIN_REF", "RemoveRefToObject(u.pointerValue);",
        "RemoveRefToVector(u.vectorValue);", "SL_RemoveRefToString(u.stringValue);"),
    "ClearObject": (
        "AddRefToObject(parentId);", "ClearObjectInternal(parentId);", "RemoveRefToEmptyObject(parentId);"),
    "ClearObjectInternal": (
        "parentValue = &gScrVarGlob.variableList[parentId + VARIABLELIST_PARENT_BEGIN];",
        "if ( parentValue->nextSibling )", "FindVariableIndexInternal(parentId, entryValue->w.classnum >> VAR_NAME_BITS)",
        "id = gScrVarGlob.variableList[nextSibling + VARIABLELIST_CHILD_BEGIN].hash.id;",
        "MakeVariableExternal(nextSibling, parentValue);",
        "nextId = parentValue->nextSibling;", "savedNextId = nextId;",
        "FreeChildValue(parentId, savedNextId);"),
    "RemoveRefToEmptyObject": (
        "assert(!entryValue->nextSibling);", "if ( entryValue->u.o.refCount )",
        "--gScrVarPub.totalObjectRefCount;", "--entryValue->u.o.refCount;", "else", "FreeVariable(id);"),
    "FreeVariable": (
        "--gScrVarPub.totalObjectRefCount;", "--gScrVarPub.numScriptObjects;",
        "index = entryValue->v.next;", "assert(entry->id == id);", "assert(!entryValue->nextSibling);",
        "entryValue->w.status = 0;",
        "entryValue->u.next = gScrVarGlob.variableList[VARIABLELIST_PARENT_BEGIN].u.next;",
        "entry->u.prev = 0;",
        "gScrVarGlob.variableList[VARIABLELIST_PARENT_BEGIN].u.next = index;"),
    "FreeChildValue": (
        "RemoveRefToValue(VAR_TYPE(entryValue), entryValue->u.u);",
        "--gScrVarPub.totalObjectRefCount;", "--gScrVarPub.numScriptValues;",
        "nextSiblingIndex = entryValue->nextSibling;", "prevSiblingIndex = entry->u.prev;",
        "entryValue->w.status = 0;",
        "entryValue->u.next = gScrVarGlob.variableList[VARIABLELIST_CHILD_BEGIN].u.next;",
        "entry->u.prev = 0;", "gScrVarGlob.variableList[VARIABLELIST_CHILD_BEGIN].u.next = index;"),
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
                raise ValueError(f"Selected array lifetime role changed: {name}")
            position = found + len(needle)


def mutations(profiles: dict[str, str]) -> int:
    count = 0
    for name, needles in ROLES.items():
        body = writes.compact(profiles[name])
        candidates = [body.replace(writes.compact(needle), "mutated_lifetime_role") for needle in needles]
        first = writes.compact(needles[0])
        candidates.append(body.replace(first, "", 1) + first)
        for body in candidates:
            changed = dict(profiles)
            changed[name] = body
            try:
                verify(changed)
            except ValueError:
                count += 1
                continue
            raise ValueError("Array lifetime guard accepted a scoped mutation")
    return count


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    profiles = inputs(ReferenceSource(args.reference))
    verify(profiles)
    print(f"Verified selected lifetime/free roles; rejected {mutations(profiles)} mutations")


if __name__ == "__main__":
    main()
