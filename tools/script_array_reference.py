# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected array copy/key/order/lifetime roles; not native VM execution."""

import argparse
from pathlib import Path
import re

from reference_source import ReferenceSource

ROLES = {
    "IsValidArrayIndex": ("return unsignedValue < MAX_ARRAYINDEX;",),
    "GetInternalVariableIndex": (
        "assert(IsValidArrayIndex(unsignedValue));",
        "return (unsignedValue + MAX_ARRAYINDEX) & VAR_NAME_LOW_MASK;",
    ),
    "Scr_GetArrayIndexValue": (
        "assert(name != 0);", "if (name >= SL_MAX_STRING_INDEX)",
        "if (name >= 0x17FFE)", "value.type = VAR_INTEGER;",
        "value.u.intValue = name - 0x800000;", "value.type = VAR_POINTER;",
        "value.u.intValue = name - SL_MAX_STRING_INDEX;", "value.type = VAR_STRING;",
        "value.u.intValue = (uint16_t)name;", "return value;",
    ),
    "Scr_EvalArrayRef": (
        "if (entryValue->u.o.refCount) { id.pointerValue = varValue.u.pointerValue; "
        "RemoveRefToObject(varValue.u.pointerValue); varValue.u.pointerValue = Scr_AllocArray(); "
        "CopyArray(id.pointerValue, varValue.u.pointerValue); assert(parentValue != NULL); "
        "parentValue->u.u.pointerValue = varValue.u.pointerValue; }",
        'Scr_Error("read-only array cannot be changed");',
        "assert(VAR_TYPE(parentValue) == VAR_UNDEFINED);",
        "parentValue->w.type |= VAR_POINTER;",
        "parentValue->u.u.pointerValue = Scr_AllocArray();",
    ),
    "ClearArray": (
        'Scr_Error("read-only array cannot be changed");',
        "if (entryValue->u.o.refCount) { id = varValue.u.pointerValue; "
        "RemoveRefToObject(varValue.u.pointerValue); varValue.u.pointerValue = Scr_AllocArray(); "
        "CopyArray(id, varValue.u.pointerValue); assert(parentValue != 0); "
        "parentValue->u.u.pointerValue = varValue.u.pointerValue; }",
        "if (value->type == VAR_INTEGER)", "if (IsValidArrayIndex(value->u.intValue))",
        "SafeRemoveArrayVariable(varValue.u.stringValue, value->u.intValue);",
        "else if (value->type == VAR_STRING)", "SL_RemoveRefToString(value->u.intValue);",
        "SafeRemoveVariable(varValue.u.pointerValue, value->u.intValue);",
    ),
    "CopyArray": (
        "id = gScrVarGlob.variableList[parentId + VARIABLELIST_PARENT_BEGIN].nextSibling;",
        "GetVariableIndexInternal(newParentId, entryValue->w.classnum >> VAR_NAME_BITS)",
        "newEntryValue->w.status |= type;", "if (type == VAR_POINTER)",
        "if (VAR_TYPE((&gScrVarGlob.variableList[entryValue->u.u.pointerValue + VARIABLELIST_PARENT_BEGIN])) == VAR_ARRAY)",
        "newEntryValue->u.u.pointerValue = Scr_AllocArray();",
        "CopyArray(entryValue->u.u.pointerValue, newEntryValue->u.u.pointerValue);",
        "newEntryValue->u.u.pointerValue = entryValue->u.u.pointerValue; "
        "AddRefToObject(entryValue->u.u.pointerValue);", "assert(type != VAR_STACK);",
        "AddRefToValue(type, entryValue->u.u);",
        "nextSibling = gScrVarGlob.variableList[id + VARIABLELIST_CHILD_BEGIN].nextSibling;",
        "id = gScrVarGlob.variableList[nextSibling + VARIABLELIST_CHILD_BEGIN].hash.id;",
    ),
    "GetNewVariableIndexInternal2": (
        "index = GetNewVariableIndexInternal3(parentId, name, index);",
        "siblingId = parentValue->nextSibling;", "parentValue->nextSibling = id;",
        "entry->hash.u.prev = 0;",
        "gScrVarGlob.variableList[id + VARIABLELIST_CHILD_BEGIN].nextSibling = siblingIndex;",
    ),
    "FindFirstSibling": ("return entryValue->nextSibling;",),
    "GetArraySize": ("assert(VAR_TYPE(entryValue) == VAR_ARRAY);", "return entryValue->u.o.u.size;"),
    "FindNextSibling": ("nextSibling = entryValue->nextSibling;",
                        "childId = list[nextSibling].hash.id;", "return childId;"),
    "Scr_AddArrayKeys": (
        "Scr_MakeArray();", "for (id = FindFirstSibling(parentId); id; id = FindNextSibling(id))",
        "value = Scr_GetArrayIndexValue(entryValue->w.name >> VAR_NAME_BITS);",
        "if (value.type == VAR_STRING)", "Scr_AddConstString(value.u.stringValue);",
        "else if (value.type == VAR_INTEGER)", "Scr_AddInt(value.u.intValue);", "Scr_AddArray();",
    ),
}
CONSTANTS = {"MAX_ARRAYINDEX": "0x800000", "SL_MAX_STRING_INDEX": "0x10000", "VAR_NAME_BITS": "8"}


def compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def inputs(reference: ReferenceSource) -> tuple[dict[str, str], str]:
    return ({name: reference.function("src/cscr_variable.cpp", name) for name in ROLES},
            reference.files["src/cscr_variable.cpp"])


def verify(functions: dict[str, str], source: str) -> None:
    for name, value in CONSTANTS.items():
        if not re.search(rf"(?m)^#define {name} {value}\s*$", source):
            raise ValueError("Native array name/index profile changed")
    for name, needles in ROLES.items():
        body = compact(functions[name])
        position = 0
        for needle in needles:
            found = body.find(compact(needle), position)
            if found < 0:
                raise ValueError(f"Selected array roles changed: {name}")
            position = found + len(compact(needle))


def verify_mutations(functions: dict[str, str], source: str) -> int:
    cases = []
    for name, needles in ROLES.items():
        body = compact(functions[name])
        for needle in needles:
            needle = compact(needle)
            if body.count(needle) != 1:
                raise ValueError("Array fault injection needs a unique scoped role")
            changed = dict(functions)
            changed[name] = body.replace(needle, "mutated_array_role", 1)
            cases.append((changed, source))
        if len(needles) > 1:
            needle = compact(needles[0])
            changed = dict(functions)
            changed[name] = body.replace(needle, "", 1) + needle
            cases.append((changed, source))
    for name, value in CONSTANTS.items():
        cases.append((functions, source.replace(f"#define {name} {value}", f"#define {name} 0", 1)))
    for changed in cases:
        try:
            verify(*changed)
        except ValueError:
            continue
        raise ValueError("Array role guard accepted a scoped mutation")
    return len(cases)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    source = inputs(ReferenceSource(args.reference))
    verify(*source)
    print(f"Verified selected array roles; rejected {verify_mutations(*source)} mutations")


if __name__ == "__main__":
    main()
