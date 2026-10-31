#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Replay pinned string actions and C-locale CRT numeric calls, not the original parser."""

import argparse
from decimal import Decimal
from pathlib import Path
import re
import subprocess
import sys
import tempfile

from reference_source import ReferenceSource
import script_lexer_reference as lexer


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/literal_vectors.txt"


def decoded(body: bytes) -> bytes:
    output = bytearray()
    position = 0
    while position < len(body):
        value = body[position]
        position += 1
        if value == 92:
            if position == len(body):
                break
            value = {114: 13, 116: 9, 110: 10}.get(body[position], body[position])
            position += 1
        output.append(value)
    return bytes(output)


def strings(tables: dict[str, list[int]]) -> list[str]:
    bodies = [b"", b"plain", b"private-secret", b"\xff\x80", b'\\n\\r\\t\\\\\\"',
              b"\\x41\\u0041\\0\\b", b"prefix\\q suffix"]
    bodies += [b"\\" + bytes([byte]) for byte in range(1, 256)]
    rows = []
    for prefix, rule, kind in ((b'"', 7, "plain"), (b'&"', 8, "localized")):
        for body in bodies:
            encoded = prefix + body + b'"'
            tokens, status = lexer.lex(tables, encoded)
            if status != "ok" or tokens != [f"{rule}:0:{len(encoded)}"]:
                continue
            rows.append(f"S {encoded.hex()} {kind} {decoded(body).hex() or '~'}")
    return rows


def numeric(tables: dict[str, list[int]], compiler: str) -> list[str]:
    integers = ["0", "1", "10", "010", "2147483647", "0000000002147483647", "16777217", "65535"]
    floats = ["0.0", ".0", ".5", "1.", "1.5", "0.1", "16777217.0", "2147483520.0",
              "1.1754943508222875e-38", "1.401298464324817e-45", "1.0e-50",
              "3.4028234663852886e38", "1.000000059604644775390625",
              "1.000000178813934326171875", "1.0e3", "2.5E-2"]
    inputs = []
    for kind, rule, values in (("I", 35, integers), ("F", 36, floats)):
        for text in values:
            if kind == "F" and "e" in text.lower():
                # Exponent notation is not a single float token in this profile.
                # Keep the numerical boundary cases in accepted fixed spelling.
                text = format(Decimal(text), "f")
                if "." not in text:
                    text += ".0"
            data = text.encode("ascii")
            tokens, status = lexer.lex(tables, data)
            if status == "ok" and tokens == [f"{rule}:0:{len(data)}"]:
                inputs.append(f"{kind} {text}")
    with tempfile.TemporaryDirectory(prefix="cod4x-literal-crt-") as scratch:
        directory = Path(scratch)
        source = ROOT / "tools/oracles/literal_crt_driver.c"
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        subprocess.run([compiler, "-std=c11", "-O0", str(source), "-lm", "-o", str(executable)], check=True)
        rows = subprocess.check_output([str(executable)], input=("\n".join(inputs) + "\n").encode("ascii"))
    return rows.decode("ascii").splitlines()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--cc", default="gcc")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    source = reference.files["src/asmsource/_scr_yacc.asm"]
    # Guard the source-specific escape actions and scanf formats being replayed.
    for value, label, byte in ((0x72, "630", 13), (0x74, "640", 9), (0x6e, "650", 10)):
        if not re.search(rf"cmp cl, {hex(value)}\n\s*jz yyparse_{label}", source):
            raise ValueError("Unexpected plain escape dispatch")
        if not re.search(rf"yyparse_{label}:\n\s*mov byte \[eax\], {hex(byte)}", source):
            raise ValueError("Unexpected plain escape replacement")
    for code in ("d", "f"):
        if not re.search(rf'_cstring_{code}:\s*db "%{code}",0', source):
            raise ValueError("Unexpected source numeric conversion format")
    tables = lexer.recover(source)
    rows = ["# Source-action strings and C-locale CRT numeric conversions; NOT original parser execution.",
            "# S source_hex kind decoded_hex; N kind source_ascii result_bits_hex"]
    rows += strings(tables) + numeric(tables, args.cc)
    expected = "\n".join(rows) + "\n"
    if args.check:
        if OUTPUT.read_text(encoding="utf-8") != expected:
            raise SystemExit("Literal action/CRT replay fixtures differ")
    else:
        OUTPUT.write_text(expected, encoding="utf-8", newline="\n")
    print(f"Verified {len(rows) - 2} source-action/CRT literal replays")


if __name__ == "__main__":
    main()
