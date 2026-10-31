#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Recover source-grounded lexer data and replay fixtures from pinned assembly."""

import argparse
from pathlib import Path
import subprocess

from reference_source import ReferenceSource
import script_lexer_reference as reference_lexer


ROOT = Path(__file__).resolve().parents[1]
RUST = ROOT / "crates/script/src/lexical/legacy_tables.rs"
FIXTURES = ROOT / "crates/script/tests/fixtures/lexical_vectors.txt"


def samples() -> list[bytes]:
    keywords = ("if else for while foreach switch case default break continue return thread wait "
                "waittill waittillmatch waittillframeend notify endon undefined true false "
                "level self game anim animation animtree breakpoint prof_begin prof_end").split()
    samples = [b"", b"main() { level thread work(); wait 0.125; return; }",
               b"/* first\r\nsecond * / \xff */ foo", b"// comment\nfoo", b"/* unfinished",
               b"/* nested /* closes */ */", b'/# developer #/', b'"unterminated',
               b'"escaped\\\"quote" &"localized"', b"0x12 1.5 .5 1. 1e3 -12",
               b"#include maps\\mp\\common;", b"#using_animtree(\"mp\");", b"a\0tail"]
    samples += [word.encode("ascii") for word in keywords]
    samples += [bytes([byte]) for byte in range(1, 256)]
    samples += [b"/*" + bytes([byte]) + b"*/" for byte in range(1, 256)]
    samples += [operator.encode("ascii") for operator in
                "( ) [ ] { } , ; : :: ? . + - * / % == != < <= > >= && || & | ^ ! ~ ++ -- += -= *= /= %= <<= >>= << >> = \\".split()]
    state = 0x1b5fa76d
    alphabet = b"abcXYZ012 _\\\"/*#;:(){}[]\r\n+=?!"
    for length in range(128):
        data = bytearray()
        for _ in range(length):
            state ^= (state << 13) & 0xffffffff
            state ^= state >> 17
            state ^= (state << 5) & 0xffffffff
            data.append(alphabet[state % len(alphabet)])
        samples.append(bytes(data))
    return samples


def render(tables: dict[str, list[int]], revision: str) -> str:
    lines = ["// SPDX-License-Identifier: AGPL-3.0-or-later",
             "// Generated from pinned _scr_yacc.asm; do not edit by hand.",
             f"// Source revision: {revision}.",
             "pub(super) const JAM_BASE: u16 = 435;",
             "pub(super) const NORMAL_STATE: usize = 3;",
             "pub(super) const COMMENT_STATE: usize = 5;"]
    for name, values in tables.items():
        kind = "u8" if name in ("EQUIVALENCE", "META") else "u16"
        lines.append(f"pub(super) const {name}: [{kind}; {len(values)}] = [")
        for index in range(0, len(values), 16):
            lines.append("    " + ", ".join(map(str, values[index:index + 16])) + ",")
        lines.append("];\n")
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    tables = reference_lexer.recover(reference.files["src/asmsource/_scr_yacc.asm"])
    rows = ["# Replayed pinned lexer tables/actions, NOT original x86 execution.",
            "# source_hex rule:start:length,... terminal_status"]
    for data in samples():
        lexemes, status = reference_lexer.lex(tables, data)
        rows.append(f"{data.hex() or '~'} {','.join(lexemes) or '~'} {status}")
    expected = "\n".join(rows) + "\n"
    generated = render(tables, reference.revision)
    # Keep checked-in generated Rust in the same stable formatter style as code.
    formatted = subprocess.run(["rustfmt", "--edition", "2024", "--emit", "stdout"],
                               input=generated, text=True, capture_output=True, check=True).stdout
    if args.check:
        if RUST.read_text(encoding="utf-8") != formatted or FIXTURES.read_text(encoding="utf-8") != expected:
            raise SystemExit("Lexer tables/replay fixtures differ from pinned source")
    else:
        RUST.parent.mkdir(parents=True, exist_ok=True)
        RUST.write_text(formatted, encoding="utf-8", newline="\n")
        FIXTURES.write_text(expected, encoding="utf-8", newline="\n")
    print(f"Verified bounded reachable tables and {len(rows) - 2} source-derived lexical replays")


if __name__ == "__main__":
    main()
