# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected compound/postfix field lowering order; not executable compiler/VM parity."""
import argparse
from pathlib import Path
import re

from reference_source import ReferenceSource
import script_array_write_reference as writes
import script_field_reference as fields
import script_update_reference as assembly

ROLES = {
    "EmitStatement_790": (
        "mov byte [scrCompileGlob+0x1c], 0x1", "call EmitVariableExpression",
        "mov byte [scrCompileGlob+0x1c], 0x0", "call EmitOrEvalExpression",
        "test al, al", "jnz EmitStatement_160"),
    "EmitStatement_160": ("mov eax, ebx", "call EmitValue", "jmp EmitStatement_500"),
    "EmitStatement_500": (
        "movsx eax, byte [ebp-0x40]", "call EmitOpcode", "call AddOpcodePos",
        "call EmitVariableExpressionRef", "mov eax, 0x39", "call EmitOpcode",
        "jmp EmitStatement_10"),
    "EmitStatement_770": (
        "mov byte [scrCompileGlob+0x50], 0x1", "call EmitVariableExpressionRef",
        "mov byte [scrCompileGlob+0x50], 0x0", "mov eax, 0x64", "call EmitOpcode",
        "mov eax, 0x39", "call EmitOpcode", "jmp EmitStatement_10"),
    "EmitStatement_780": (
        "mov byte [scrCompileGlob+0x50], 0x1", "call EmitVariableExpressionRef",
        "mov byte [scrCompileGlob+0x50], 0x0", "mov eax, 0x65", "call EmitOpcode",
        "mov eax, 0x39", "jmp EmitStatement_150"),
}


def verify(compiler: str) -> None:
    labels = assembly.dispatch(compiler, "EmitStatement", 79)
    if [labels[tag] for tag in (0x29, 0x2a, 0x2b)] != [
            "EmitStatement_770", "EmitStatement_780", "EmitStatement_790"]:
        raise ValueError("Field update statement dispatch changed")
    for name, needles in ROLES.items():
        raw = assembly.block(compiler, name)
        body, position = writes.compact(raw), 0
        for needle in needles:
            needle = writes.compact(needle)
            found = body.find(needle, position)
            if found < 0:
                raise ValueError(f"Field update role/order changed: {name}")
            position = found + len(needle)
        if body.count("callEmitVariableExpressionRef") != int(name in (
                "EmitStatement_500", "EmitStatement_770", "EmitStatement_780")):
            raise ValueError("Field update writable target evaluation count changed")
        reads = re.findall(r"(?m)^[ \t]*call[ \t]+EmitVariableExpression[ \t]*$", raw)
        if len(reads) != int(name == "EmitStatement_790"):
            raise ValueError("Field update read target evaluation count changed")


def mutations(compiler: str) -> int:
    changed = []
    for name, needles in ROLES.items():
        body = assembly.block(compiler, name)
        for needle in dict.fromkeys(needles):
            # Delete every occurrence when an opcode emitter appears twice.
            changed.append(compiler.replace(body, body.replace(needle, "mutated_update_role"), 1))
        first = needles[0]
        changed.append(compiler.replace(body, body.replace(first, "", 1) + first, 1))
    for name in ("EmitStatement_770", "EmitStatement_780", "EmitStatement_790"):
        changed.append(compiler.replace(f"\tdd {name}\n", "\tdd EmitStatement_10\n", 1))
    for name in ("EmitStatement_500", "EmitStatement_770", "EmitStatement_780", "EmitStatement_790"):
        body = assembly.block(compiler, name)
        call = "call EmitVariableExpression" if name == "EmitStatement_790" else "call EmitVariableExpressionRef"
        needle = call + "\n"
        changed.append(compiler.replace(body, body.replace(needle, needle + "\t" + needle, 1), 1))
    for source in changed:
        try:
            verify(source)
        except ValueError:
            continue
        raise ValueError("Field update guard accepted a scoped mutation")
    return len(changed)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    fields.verify(*fields.inputs(reference))
    compiler = reference.files["src/asmsource/_scr_compiler.asm"]
    assembly.verify(compiler, reference.files["src/asmsource/_scr_vm.asm"], reference.files["src/scr_vm.h"])
    verify(compiler)
    print(f"Verified selected field update order; rejected {mutations(compiler)} mutations")


if __name__ == "__main__":
    main()
