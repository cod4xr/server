# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected native array reads and size; no compiler/VM/unwind execution."""
import argparse
from pathlib import Path

from reference_source import ReferenceSource
from script_array_reference import compact


ROLES = {
    "FindArrayVariableIndex": (
        "assert(IsValidArrayIndex( unsignedValue ));",
        "return FindVariableIndexInternal(parentId, (unsignedValue + VAR_ARRAYINDEXSTART) & VAR_NAME_LOW_MASK);"),
    "FindArrayVariable": (
        "FindArrayVariableIndex(parentId, intValue) + VARIABLELIST_CHILD_BEGIN", ".hash.id;"),
    "FindVariable": (
        "FindVariableIndexInternal(parentId, unsignedValue) + VARIABLELIST_CHILD_BEGIN", ".hash.id;"),
    "Scr_FindArrayIndex": (
        "if ( index->type == VAR_INTEGER )", "if ( IsValidArrayIndex(index->u.intValue) )",
        "return FindArrayVariable(parentId, index->u.intValue);",
        'Scr_Error(va("array index %d out of range", index->u.intValue)); AddRefToObject(parentId); return 0;',
        "if ( index->type == VAR_STRING )", "id = FindVariable(parentId, index->u.intValue);",
        "SL_RemoveRefToString(index->u.intValue);", "return id;",
        'Scr_Error(va("%s is not an array index", var_typename[index->type])); AddRefToObject(parentId); return 0;'),
    "Scr_EvalVariable": (
        "id + VARIABLELIST_CHILD_BEGIN", "VAR_STAT_FREE) || !id",
        "value.type = VAR_TYPE(entryValue);", "value.u.intValue = entryValue->u.u.intValue;",
        "assert(!IsObject( entryValue ));", "AddRefToValue(value.type, value.u);", "return value;"),
    "Scr_EvalArray": (
        "assert( value != index);", "case VAR_POINTER:",
        "value->u.intValue + VARIABLELIST_PARENT_BEGIN", "assert(IsObject( entryValue ));",
        "if ( (VAR_TYPE(entryValue)) == VAR_ARRAY )",
        "arridx = Scr_FindArrayIndex(value->u.intValue, index);",
        "*index = Scr_EvalVariable(arridx);", "RemoveRefToObject(value->u.intValue);",
        'gScrVarPub.error_index = 1; Scr_Error(va("%s is not an array", var_typename[VAR_TYPE(entryValue)]));'),
    "Scr_EvalSizeValue": (
        "if ( value->type == VAR_POINTER )", "id = value->u.intValue;",
        "value->u.intValue + VARIABLELIST_PARENT_BEGIN",
        "value->type = VAR_INTEGER; if ( VAR_TYPE(entryValue) == VAR_ARRAY )",
        "value->u.intValue = entryValue->u.o.u.size;",
        "value->u.intValue = 1;", "RemoveRefToObject(id);"),
}
NO_MUTATION = ("CopyArray(", "Scr_EvalArrayRef(", "GetNewVariable(", "SetVariableValue(")


def inputs(reference: ReferenceSource) -> dict[str, str]:
    return {name: reference.function("src/cscr_variable.cpp", name) for name in ROLES}


def verify(functions: dict[str, str]) -> None:
    for name, needles in ROLES.items():
        body, position = compact(functions[name]), 0
        for needle in needles:
            expected = compact(needle)
            found = body.find(expected, position)
            if found < 0:
                raise ValueError("Selected native array read role changed")
            position = found + len(expected)
    for name in ("Scr_EvalArray", "Scr_EvalSizeValue"):
        if any(compact(call) in compact(functions[name]) for call in NO_MUTATION):
            raise ValueError("Array read unexpectedly mutates or detaches data")


def verify_mutations(functions: dict[str, str]) -> int:
    cases = []
    for name, needles in ROLES.items():
        body = compact(functions[name])
        for needle in needles:
            needle = compact(needle)
            if body.count(needle) != 1:
                raise ValueError("Read fault injection requires unique scoped roles")
            changed = dict(functions)
            changed[name] = body.replace(needle, "mutated_read_role", 1)
            cases.append(changed)
        if len(needles) > 1:
            first = compact(needles[0])
            changed = dict(functions)
            changed[name] = body.replace(first, "", 1) + first
            cases.append(changed)
    for name in ("Scr_EvalArray", "Scr_EvalSizeValue"):
        for call in NO_MUTATION:
            changed = dict(functions)
            changed[name] += "\n" + call
            cases.append(changed)
    for changed in cases:
        try:
            verify(changed)
        except ValueError:
            continue
        raise ValueError("Array read guard accepted an injected role change")
    return len(cases)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    functions = inputs(ReferenceSource(args.reference))
    verify(functions)
    print(f"Verified selected array read roles; rejected {verify_mutations(functions)} mutations")


if __name__ == "__main__":
    main()
