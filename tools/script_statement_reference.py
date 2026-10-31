# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard local statement/body roles; this does not execute native semantic actions."""

import re


def verify(source: str, tables: dict[str, list[int]], header: str) -> None:
    """Check statement production/constructor and compound-operator identities."""
    section = source.split("yyparse_jumptab_0:\n", 1)[1].split("yyparse_jumptab_1:", 1)[0]
    labels = re.findall(r"\bdd (yyparse_\d+)\s*", section)
    if len(labels) != 134:
        raise ValueError("Unexpected semantic dispatch shape")
    profiles = {
        67: (102, 3, 2, "node4"), 68: (102, 2, 0x1b, "node2"),
        69: (102, 1, 0x1c, "node1"), 70: (102, 2, 0x1d, "node3"),
        71: (102, 2, 0x29, "node2"), 72: (102, 2, 0x2a, "node2"),
        83: (102, 5, 0x37, "node4"), 84: (102, 5, 0x38, "node4"),
        85: (102, 1, 0x39, "node1"), 86: (102, 5, 0x3a, "node4"),
        87: (102, 5, 0x3b, "node4"), 88: (102, 1, 0x3f, "node1"),
        89: (102, 1, 0x40, "node1"), 90: (102, 1, 0x4c, "node1"),
        91: (102, 4, 0x4d, "node2"), 92: (102, 4, 0x4e, "node2"),
        93: (103, 1, 0x19, "node1"), 94: (103, 1, None, None),
        95: (104, 0, 0, "node0"), 96: (104, 1, None, None),
        97: (105, 2, None, None), 98: (105, 3, 0x2c, "node3"),
        99: (105, 5, 0x25, "node4"), 100: (105, 7, 0x26, "node7"),
        101: (105, 5, 0x27, "node5"), 102: (105, 8, 0x28, "node8"),
        103: (105, 7, 0x3c, "node3"), 104: (105, 3, 0x2d, "node3"),
        105: (106, 1, 0, "node0"), 106: (106, 3, 0x3d, "node3"),
        107: (106, 2, 0x3e, "node2"), 108: (106, 1, None, None),
        109: (107, 2, None, "append_node"), 110: (107, 0, 0, "node0"),
    }
    profiles.update({rule: (102, 3, 0x2b, "node4") for rule in range(73, 83)})
    for rule, (lhs, length, tag, constructor) in profiles.items():
        body = source.split(labels[rule] + ":\n", 1)[1].split("\nyyparse_", 1)[0]
        if tables["LHS"][rule] != lhs or tables["LENGTH"][rule] != length:
            raise ValueError(f"Statement/body production profile changed: {rule}")
        if tag is not None and f"mov dword [esp], 0x{tag:x}" not in body:
            raise ValueError(f"Statement semantic tag changed: {rule}")
        if constructor is not None and f"call {constructor}\n" not in body:
            raise ValueError(f"Statement/body constructor changed: {rule}")
        if rule in (94, 96, 97, 108) and "mov ebx, [ebp-0x280c]" not in body:
            raise ValueError("Statement wrapper alias changed")
    for rule, name, opcode in (
        (73, "bit_or", 0x66), (74, "bit_ex_or", 0x67), (75, "bit_and", 0x68),
        (76, "shift_left", 0x6f), (77, "shift_right", 0x70), (78, "plus", 0x71),
        (79, "minus", 0x72), (80, "multiply", 0x73), (81, "divide", 0x74),
        (82, "mod", 0x75),
    ):
        body = source.split(labels[rule] + ":\n", 1)[1].split("\nyyparse_", 1)[0]
        if f"mov dword [esp], 0x{opcode:x}" not in body or "call node1_\n" not in body:
            raise ValueError("Compound assignment operator action changed")
        if not re.search(rf"\bOP_{name}\s*=\s*0x{opcode:x}\b", header, re.IGNORECASE):
            raise ValueError("Compound assignment opcode identity changed")
    body = source.split(labels[109] + ":\n", 1)[1].split("\nyyparse_", 1)[0]
    if "mov ecx, [eax]" not in body or "mov edx, [eax-0x8]" not in body:
        raise ValueError("Body-list append operand roles changed")
    body = source.split(labels[110] + ":\n", 1)[1].split("\nyyparse_", 1)[0]
    if "call linked_list_end\n" not in body:
        raise ValueError("Empty body-list action changed")
