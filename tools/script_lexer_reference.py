# SPDX-License-Identifier: AGPL-3.0-or-later
"""Replay recovered lexer tables/actions; this is not original x86 execution."""

import re
import struct


WORD_TABLES = {
    "ACCEPT": "yy_accept", "BASE": "yy_base", "CHECK": "yy_chk",
    "DEFAULT": "yy_def", "NEXT": "yy_nxt",
}
JAM_BASE = 0x1b3
NORMAL_STATE = 3
COMMENT_STATE = 5


def recover(source: str) -> dict[str, list[int]]:
    tables = {}
    for output, label in {**WORD_TABLES, "EQUIVALENCE": "yy_ec", "META": "yy_meta"}.items():
        match = re.search(rf"(?m)^{label}: dd ([^\n]+)", source)
        if not match:
            raise ValueError(f"Missing source table: {label}")
        words = [int(value.strip(), 16) for value in match[1].split(",")]
        if output in WORD_TABLES:
            raw = b"".join(struct.pack("<I", word) for word in words)
            tables[output] = list(struct.unpack(f"<{len(raw) // 2}H", raw))
        else:
            count = 256 if output == "EQUIVALENCE" else 64
            # yy_ec's label includes following unlabeled original data. Never
            # export those addresses/data as part of the character map.
            values = words[:count]
            if len(values) != count or any(value > 255 for value in values):
                raise ValueError("Unexpected byte-map shape")
            tables[output] = values
    jump_table = re.search(r"(?ms)^yyparse_jumptab_1:\n(.*?)(?=\n\n)", source)
    labels = re.findall(r"dd (yyparse_[0-9]+)", jump_table[1])
    if len(labels) != 99:
        raise ValueError("Unexpected lexical action count")
    codes = [0] * 94
    modes = [0] * 94
    for rule in range(1, 94):
        label = labels[rule]
        action = re.search(rf"(?ms)^{label}:\n(.*?)(?=^yyparse_[0-9]+:)", source)[1]
        changes = re.findall(r"mov dword \[yy_start\], (0x[0-9a-f]+)", action)
        if changes:
            modes[rule] = int(changes[-1], 16)
        if rule >= 9:
            values = re.findall(r"mov eax, (0x[0-9a-f]+)", action)
            codes[rule] = int(values[-1], 16)
    # Literal decoding branches join shared parser exits. These constants are
    # separately asserted in the pinned source; no decoding/interning is ported.
    for rule, code in ((7, 0x103), (8, 0x104)):
        if len(re.findall(rf"mov eax, {hex(code)}\b", source)) != 1:
            raise ValueError("Unexpected literal action exit")
        codes[rule] = code
    tables["TOKEN_CODES"] = codes
    if modes[2] != NORMAL_STATE or modes[6] != COMMENT_STATE or any(mode not in (0, NORMAL_STATE, COMMENT_STATE) for mode in modes):
        raise ValueError("Unexpected scanner mode actions")
    tables["RULE_MODES"] = modes
    validate(tables)
    return tables


def transition(tables: dict[str, list[int]], state: int, symbol: int) -> int:
    for _ in range(len(tables["BASE"])):
        index = tables["BASE"][state] + symbol
        if tables["CHECK"][index] == state:
            return tables["NEXT"][index]
        state = tables["DEFAULT"][state]
        if state > 0x101:
            symbol = tables["META"][symbol]
    raise ValueError("Cyclic fallback chain")


def validate(tables: dict[str, list[int]]) -> None:
    if [len(tables[key]) for key in ("ACCEPT", "BASE", "DEFAULT", "CHECK", "NEXT")] != [272, 272, 272, 496, 496]:
        raise ValueError("Unexpected lexer table dimensions")
    pending = [NORMAL_STATE, COMMENT_STATE]
    seen = set()
    while pending:
        state = pending.pop()
        if state in seen or tables["BASE"][state] == JAM_BASE:
            continue
        seen.add(state)
        for byte in range(1, 256):
            target = transition(tables, state, tables["EQUIVALENCE"][byte])
            if not 0 <= target < len(tables["BASE"]):
                raise ValueError("Invalid reachable transition")
            pending.append(target)


def first(tables: dict[str, list[int]], data: bytes, mode: int) -> tuple[int, int] | None:
    state = mode
    accepted = None
    for position, byte in enumerate(data):
        if byte == 0:
            raise ValueError("nul")
        if tables["ACCEPT"][state]:
            accepted = (tables["ACCEPT"][state], position)
        state = transition(tables, state, tables["EQUIVALENCE"][byte])
        if tables["BASE"][state] == JAM_BASE:
            return (tables["ACCEPT"][state], position + 1) if tables["ACCEPT"][state] else accepted
    # Real source EOF is distinct from the original scanner's refill/NUL action.
    return (tables["ACCEPT"][state], len(data)) if tables["ACCEPT"][state] else accepted


def lex(tables: dict[str, list[int]], data: bytes) -> tuple[list[str], str]:
    position = 0
    mode = NORMAL_STATE
    output = []
    while position < len(data):
        try:
            match = first(tables, data[position:], mode)
        except ValueError:
            return output, "nul"
        if not match or match[0] >= 93 or match[1] == 0:
            return output, "invalid"
        rule, length = match
        output.append(f"{rule}:{position}:{length}")
        position += length
        mode = tables["RULE_MODES"][rule] or mode
    return output, "comment" if mode == COMMENT_STATE else "ok"
