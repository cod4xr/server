# SPDX-License-Identifier: AGPL-3.0-or-later
"""Structural grammar replay from pinned tables; semantic actions are not executed."""

import re
import struct

import script_lexer_reference as lexical


LAST = 0x543
FINAL = 0x105
NONTERMINAL_START = 0x5b
UNSET = -32768
MODES = {"script": 0x118, "fragment": 0x11a}


def recover(source: str) -> dict[str, list[int]]:
    shapes = {
        "TRANSLATE": ("yytranslate", "b", 345), "PACT": ("yypact", "h", 262),
        "DEFAULT_REDUCE": ("yydefact", "h", 262), "ACTION": ("yytable", "h", LAST + 1),
        "CHECK": ("yycheck", "h", LAST + 1), "LHS": ("yyr1", "h", 134),
        "LENGTH": ("yyr2", "h", 134), "GOTO_OFFSET": ("yypgoto", "h", 28),
        "DEFAULT_GOTO": ("yydefgoto", "h", 28),
    }
    tables = {}
    for name, (label, kind, size) in shapes.items():
        match = re.search(rf"(?m)^{label}: dd ([^\n]+)", source)
        if not match:
            raise ValueError("Missing parser table")
        words = [int(value.strip(), 16) for value in match[1].split(",")]
        raw = b"".join(struct.pack("<I", word) for word in words)
        values = list(struct.unpack(f"<{len(raw) // struct.calcsize(kind)}{kind}", raw))
        if len(values) < size or any(values[size:]):
            raise ValueError("Unexpected parser data/padding")
        tables[name] = values[:size]
    if min(tables["LENGTH"]) < 0 or min(tables["TRANSLATE"]) < 0:
        raise ValueError("Negative production/terminal data")
    if [index for index, lhs in enumerate(tables["LHS"]) if lhs == NONTERMINAL_START] != [1, 2, 3, 4]:
        raise ValueError("Unexpected entry productions")
    if any(not NONTERMINAL_START <= lhs < NONTERMINAL_START + 28 for lhs in tables["LHS"][1:]):
        raise ValueError("Invalid production nonterminal")
    if any(not 0 <= state < 262 for state in tables["DEFAULT_GOTO"]):
        raise ValueError("Invalid goto state")
    if any(not 0 <= rule <= 133 for rule in tables["DEFAULT_REDUCE"]):
        raise ValueError("Invalid default reduction")
    if any(value > FINAL or value < -133 for value in tables["ACTION"]):
        raise ValueError("Invalid action target")
    for needle in ("cmp eax, 0x158", "cmp edx, 0x543", "cmp ax, 0x105", "lea ebx, [eax-0x5b]"):
        if needle not in source:
            raise ValueError("Parser constants differ from source")
    return tables


def action(tables: dict[str, list[int]], state: int, code: int) -> int:
    symbol = tables["TRANSLATE"][code]
    base = tables["PACT"][state]
    index = base + symbol
    if base != UNSET and 0 <= index <= LAST and tables["CHECK"][index] == symbol:
        return tables["ACTION"][index]
    return -tables["DEFAULT_REDUCE"][state]


def goto(tables: dict[str, list[int]], state: int, rule: int) -> int:
    lhs = tables["LHS"][rule] - NONTERMINAL_START
    index = tables["GOTO_OFFSET"][lhs] + state
    if 0 <= index <= LAST and tables["CHECK"][index] == state:
        return tables["ACTION"][index]
    return tables["DEFAULT_GOTO"][lhs]


def parse(tables: dict[str, list[int]], tokens: list[int], mode: str,
          lexical_status: str = "ok") -> tuple[str, list[int]]:
    values = [MODES[mode]] + tokens
    states = [0]
    position = 0
    code = None
    reductions = []
    for _ in range(100000):
        if tables["PACT"][states[-1]] == UNSET:
            target = -tables["DEFAULT_REDUCE"][states[-1]]
        else:
            if code is None:
                if position >= len(values) and lexical_status != "ok":
                    return lexical_status, reductions
                code = values[position] if position < len(values) else 0
            target = action(tables, states[-1], code)
        if target == FINAL:
            return "ok", reductions
        if target > 0:
            states.append(target)
            if code != 0:  # Original parser retains EOF after an EOF shift.
                position += 1
                code = None
        elif target == 0 or target == UNSET:
            return "syntax", reductions
        else:
            rule = -target
            count = tables["LENGTH"][rule]
            if count >= len(states):
                raise ValueError("Parser stack underflow")
            if count:
                del states[-count:]
            states.append(goto(tables, states[-1], rule))
            reductions.append(rule)
    raise ValueError("Unbounded structural replay")


def source_tokens(tables: dict[str, list[int]], data: bytes) -> tuple[list[int], str]:
    lexemes, status = lexical.lex(tables, data)
    tokens = [tables["TOKEN_CODES"][int(row.split(":")[0])] for row in lexemes]
    return [code for code in tokens if code], status
