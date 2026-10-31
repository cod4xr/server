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
