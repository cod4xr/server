# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard borrowed operator roles; this does not execute native semantic actions."""

import re


def verify(source: str, tables: dict[str, list[int]], header: str) -> None:
    """Check the expression wrapper/operator/primary production profile."""
    section = source.split("yyparse_jumptab_0:\n", 1)[1].split("yyparse_jumptab_1:", 1)[0]
    labels = re.findall(r"\bdd (yyparse_\d+)\s*", section)
    if len(labels) != 134:
        raise ValueError("Unexpected semantic dispatch shape")
    roles = {
        2: (2, 0x41, "node1"), 5: (1, 6, "node2"),
        6: (3, 0x2f, "node5"), 7: (3, 0x30, "node5"),
        24: (2, 0x32, "node2"), 25: (2, 0x33, "node2"),
    }
    for rule, (length, tag, constructor) in roles.items():
        body = source.split(labels[rule] + ":\n", 1)[1].split("\nyyparse_", 1)[0]
        if tables["LENGTH"][rule] != length or f"mov dword [esp], 0x{tag:x}" not in body:
            raise ValueError("Expression wrapper/logical/prefix profile changed")
        if f"call {constructor}\n" not in body:
            raise ValueError("Expression semantic constructor changed")
    names = ("bit_or", "bit_ex_or", "bit_and", "equality", "inequality", "less", "greater",
             "less_equal", "greater_equal", "shift_left", "shift_right", "plus", "minus",
             "multiply", "divide", "mod")
    for rule, name in enumerate(names, 8):
        opcode = 0x66 + rule - 8
        body = source.split(labels[rule] + ":\n", 1)[1].split("\nyyparse_", 1)[0]
        instructions = (f"mov dword [esp], 0x{opcode:x}", "call node1_\n",
                        "mov dword [esp], 0x31", "call node4\n")
        if tables["LENGTH"][rule] != 3 or any(text not in body for text in instructions):
            raise ValueError("Eager infix production/action profile changed")
        if not re.search(rf"\bOP_{name}\s*=\s*0x{opcode:x}\b", header, re.IGNORECASE):
            raise ValueError("Operator opcode/header identity changed")
    if any(tables["LHS"][rule] != 92 for rule in range(5, 26)):
        raise ValueError("Operator expression nonterminal changed")
    if any(tables["LHS"][rule] != 100 for rule in range(40, 62)):
        raise ValueError("Opaque primitive nonterminal profile changed")
    verify_primaries(source, tables, labels)
    verify_lists(source, tables, labels)
    verify_calls(source, tables, labels)


def verify_primaries(source: str, tables: dict[str, list[int]], labels: list[str]) -> None:
    """Guard local primary roles without executing or converting their values."""
    profiles = (
        (3, 0x2e, "node2"), (1, 7, "node2"), (1, 8, "node2"),
        (2, 9, "node2"), (2, 10, "node2"), (1, 11, "node2"), (1, 12, "node2"),
        (1, 0x13, "node1"), (1, 0x11, "node2"), (1, 0x1f, "node1"),
        (1, 0x20, "node1"), (1, 0x22, "node1"), (1, 0x23, "node1"), (1, 0x24, "node1"),
        (2, 0x34, "node2"), (1, 0x12, "node2"), (2, 0x42, "node1"), (2, 0x43, "node2"),
        (1, 0x48, "node1"), (1, 0x49, "node1"), (1, 0x4a, "node1"), (5, 0x4b, "node3"),
    )
    for rule, (length, tag, constructor) in enumerate(profiles, 40):
        body = source.split(labels[rule] + ":\n", 1)[1].split("\nyyparse_", 1)[0]
        instructions = (f"mov dword [esp], 0x{tag:x}", f"call {constructor}\n")
        if tables["LENGTH"][rule] != length or any(text not in body for text in instructions):
            raise ValueError(f"Primary production/action profile changed: {rule}")
    watch = source.split(labels[61] + ":\n", 1)[1].split("\nyyparse_", 1)[0]
    if "mov edi, [ebx-0x8]" not in watch or "mov edx, [ebx-0x20]" not in watch:
        raise ValueError("Breakon operand role changed")


def verify_lists(source: str, tables: dict[str, list[int]], labels: list[str]) -> None:
    """Check spine/optional roles; prepended native nodes do not assert evaluation order."""
    profiles = {
        111: (108, 3, ("call node2_\n", "call prepend_node\n", "mov ecx, [ecx-0x10]")),
        112: (108, 1, ("call node0\n", "call node2_\n", "call prepend_node\n")),
        113: (109, 1, ("mov ebx, [ebp-0x280c]",)),
        114: (109, 0, ("mov dword [esp], 0x0", "call node0\n")),
    }
    for rule, (lhs, length, instructions) in profiles.items():
        body = source.split(labels[rule] + ":\n", 1)[1].split("\nyyparse_", 1)[0]
        if tables["LHS"][rule] != lhs or tables["LENGTH"][rule] != length:
            raise ValueError("Expression list production profile changed")
        if any(text not in body for text in instructions):
            raise ValueError("Expression list action role changed")


def verify_calls(source: str, tables: dict[str, list[int]], labels: list[str]) -> None:
    """Guard exact callee/name/invocation roles, not native calls or scheduling."""
    profiles = {
        28: (94, 1, None, None), 29: (94, 1, None, None),
        30: (95, 3, 0x15, "node3"), 31: (95, 1, 0x14, "node2"),
        32: (96, 3, 0x15, "node3"), 33: (96, 2, 0x14, "node2"),
        34: (97, 1, 0x12, "node2"), 35: (97, 5, 0x16, "node2"),
        36: (98, 1, 0x1a, "node2"), 37: (98, 2, 0x1e, "node3"),
        38: (99, 4, 0x17, "node3"), 39: (99, 5, 0x18, "node5"),
    }
    for rule, (lhs, length, tag, constructor) in profiles.items():
        body = source.split(labels[rule] + ":\n", 1)[1].split("\nyyparse_", 1)[0]
        if tables["LHS"][rule] != lhs or tables["LENGTH"][rule] != length:
            raise ValueError("Call/name production shape changed")
        if rule <= 33 and "call SL_ConvertToLowercase" not in body:
            raise ValueError("Native name action role changed")
        if tag is not None:
            instructions = (f"mov dword [esp], 0x{tag:x}", f"call {constructor}\n")
            if any(text not in body for text in instructions):
                raise ValueError("Call/name semantic constructor profile changed")
