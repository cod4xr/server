# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected pure predicate signatures/registrations and compiler lookup roles."""

import argparse
from pathlib import Path
import re

from reference_source import ReferenceSource
import script_update_reference as assembly

PREDICATES = (("isint", "GScr_IsInt", "VAR_INTEGER", 6, "Scr_AddBool"),
              ("isfloat", "GScr_IsFloat", "VAR_FLOAT", 5, "Scr_AddBool"),
              ("isvector", "GScr_IsVector", "VAR_VECTOR", 4, "Scr_AddBool"),
              ("isstring", "GScr_IsString", "VAR_STRING", 2, "Scr_AddInt"))
ROLES = {
    "EmitCall_10": ("mov ebx, [eax+0x4]\n\tcmp dword [ebx], 0x12\n\tjz EmitCall_110\n",),
    "EmitCall_110": (
        "cmp dword [ebx], 0x14\n\tjnz EmitCall_130\n",
        "mov eax, [scrCompileGlob+0x8]\n\tmov [esp], eax\n\tcall FindVariable\n",
        "test eax, eax\n\tjnz EmitCall_130\n",
        "jz EmitCall_140\n",
    ),
    "EmitCall_140": ("call Scr_GetFunction\n",),
    "EmitCall_170": ("call EmitOrEvalExpression\n", "jz EmitCall_190\n", "call EmitValue\n"),
    "EmitCall_190": ("add dword [ebp-0x44], 0x1\n", "mov ebx, [ebx+0x4]\n", "jz EmitCall_180\n"),
    "ScriptCompile_260": (
        "mov eax, gScrVarPub\n\tcmp byte [eax+0x7], 0x0\n\tjz ScriptCompile_250\n",
        "mov dword [ebp-0x54], 0xc\n\tjmp ScriptCompile_570\n",
    ),
}


def verify(compiler: str, functions: tuple[str, ...], registrations: str, header: str) -> None:
    """Check complete small C bodies plus selected ordered assembly roles, not execution."""
    for (_, name, kind, tag, writer), function in zip(PREDICATES, functions, strict=True):
        pattern = (rf'void\s+{name}\(\)\s*\{{\s*if\s*\(Scr_GetNumParam\(\)\s*!=\s*1\)\s*'
                   rf'\{{\s*Scr_Error\("[^"\n]+"\);\s*return;\s*\}}\s*'
                   rf'{writer}\(Scr_GetType\(0\)\s*==\s*{tag}\);\s*\}}')
        if not re.fullmatch(pattern, function):
            raise ValueError("Predicate arity-before-type/result roles changed")
        if not re.search(rf"\b{kind}\s*=\s*0x{tag:x}\b", header):
            raise ValueError("Predicate native type identities changed")
    for spelling, name, _, _, _ in PREDICATES:
        if not re.search(rf'(?m)^\s*Scr_AddFunction\("{spelling}",\s*{name},\s*0\s*\);', registrations):
            raise ValueError("Pure builtin nondeveloper registration changed")
    for label, needles in ROLES.items():
        body = assembly.block(compiler, label)
        position = 0
        for needle in needles:
            found = body.find(needle, position)
            if found < 0:
                raise ValueError("Predicate lookup/developer/argument roles changed")
            position = found + len(needle)


def verify_mutations(inputs: tuple[str, tuple[str, ...], str, str]) -> int:
    """Reject scoped signature/tag/registration/role and preserved-needle ordering changes."""
    cases = []
    compiler, functions, registrations, header = inputs
    for label, needles in ROLES.items():
        body = assembly.block(compiler, label)
        original = f"{label}:\n{body}"
        for needle in needles:
            if body.count(needle) != 1:
                raise ValueError("Predicate fault injection requires unique scoped roles")
            changed = body.replace(needle, "mutated_role\n", 1)
            cases.append((compiler.replace(original, f"{label}:\n{changed}", 1), functions, registrations, header))
        if len(needles) > 1:
            changed = body.replace(needles[0], "", 1) + needles[0]
            cases.append((compiler.replace(original, f"{label}:\n{changed}", 1), functions, registrations, header))
    for index, (spelling, name, kind, tag, writer) in enumerate(PREDICATES):
        other_writer = "Scr_AddInt" if writer == "Scr_AddBool" else "Scr_AddBool"
        for old, new in (("!= 1", "!= 2"), (f"== {tag}", f"== {tag+1}"), ("return;", ""), (writer, other_writer)):
            changed = list(functions)
            changed[index] = changed[index].replace(old, new, 1)
            cases.append((compiler, tuple(changed), registrations, header))
        changed = list(functions)
        # Every original call/token remains, but result access moves ahead of arity.
        expression = f"    {writer}(Scr_GetType(0) == {tag});"
        changed[index] = changed[index].replace(expression, "", 1).replace("{", "{\n" + expression, 1)
        cases.append((compiler, tuple(changed), registrations, header))
        changed = registrations.replace(f'Scr_AddFunction("{spelling}", {name}, 0',
                                        f'Scr_AddFunction("{spelling}", {name}, 1', 1)
        cases.append((compiler, functions, changed, header))
        match = re.search(rf"\b{kind}\s*=\s*0x{tag:x}\b", header)
        if not match:
            raise ValueError("Missing recorded predicate type")
        cases.append((compiler, functions, registrations,
                      header[:match.start()] + f"{kind} = 0xff" + header[match.end():]))
    for changed in cases:
        try:
            verify(*changed)
        except ValueError:
            continue
        raise ValueError("Predicate source guard accepted an injected role change")
    return len(cases)


def inputs(reference: ReferenceSource) -> tuple[str, tuple[str, ...], str, str]:
    """Extract locked small C definitions and selected compiler/header sources."""
    return (reference.files["src/asmsource/_scr_compiler.asm"],
            tuple(reference.function("src/scr_vm_functions.c", name) for _, name, _, _, _ in PREDICATES),
            reference.function("src/scr_vm_main.c", "Scr_AddStockFunctions"), reference.files["src/scr_vm.h"])


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    source = inputs(ReferenceSource(args.reference))
    verify(*source)
    print(f"Verified pure predicate source roles; rejected {verify_mutations(source)} mutations")


if __name__ == "__main__":
    main()
