#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Generate static Huffman data and differential vectors from original C."""

import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

from reference_source import ReferenceSource


ROOT = Path(__file__).resolve().parents[1]
FUNCTIONS = (
    "add_bit", "get_bit", "Huff_offsetReceive", "Huff_send",
    "Huff_offsetTransmit", "Huff_Init", "MSG_ReadBitsCompress",
    "MSG_WriteBitsCompress", "Huff_BuildFromData", "Huffman_InitMain"
)


def run_oracle(reference: ReferenceSource, compiler: str) -> bytes:
    header = reference.files["src/huffman.h"].replace('#include "msg.h"', "")
    parts = [
        "#include <stdint.h>\n#include <stdio.h>\n#include <string.h>\n"
        "typedef unsigned char byte; typedef int qboolean;\n"
        "enum { qfalse = 0, qtrue = 1 };\n#define Com_Memset memset\n",
        header, "static int bloc = 0; static huff_t msgHuff;"
    ]
    parts.extend(reference.function("src/huffman.c", name) for name in FUNCTIONS)
    parts.append(
        (ROOT / "tools/oracles/huffman_driver.c").read_text(encoding="utf-8")
    )
    with tempfile.TemporaryDirectory(prefix="cod4x-huffman-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / (
            "oracle.exe" if sys.platform == "win32" else "oracle"
        )
        subprocess.run(
            [compiler, "-std=c11", "-O0", str(source), "-o", str(executable)],
            check=True
        )
        return subprocess.check_output([str(executable)]).replace(b"\r\n", b"\n")


def generate_table(rows: bytes) -> str:
    codes = []
    widths = []
    for row in rows.decode("ascii").splitlines():
        if not row.startswith("code "):
            continue
        _, symbol, width, encoded = row.split()
        if int(symbol) != len(codes):
            raise ValueError("Symbols must be in byte order, followed by NYT")
        bits = int.from_bytes(bytes.fromhex(encoded), "little")
        width = int(width)
        if width > 16:
            raise ValueError("Code exceeds the reviewed u16 representation")
        codes.append(bits)
        widths.append(width)
    if len(codes) != 257:
        raise ValueError("Expected all byte codes and the NYT sentinel")
    edges = [[None, None]]
    for symbol, (code, width) in enumerate(zip(codes, widths)):
        node = 0
        for depth in range(width):
            bit = (code >> depth) & 1
            if depth == width - 1:
                if edges[node][bit] is not None:
                    raise ValueError("Non-prefix-free Huffman table")
                edges[node][bit] = 256 + symbol
            else:
                child = edges[node][bit]
                if child is None:
                    child = len(edges)
                    edges[node][bit] = child
                    edges.append([None, None])
                if child >= 256:
                    raise ValueError("Code overlaps a leaf")
                node = child
    if len(edges) != 256 or any(None in edge for edge in edges):
        raise ValueError("Expected a complete 256-node internal tree")
    output = [
        "// SPDX-License-Identifier: AGPL-3.0-or-later",
        "// Generated from pinned huffman.c; see NOTICE/upstream.lock.json.",
        "// Regenerate with tools/generate_huffman_vectors.py, then cargo fmt."
    ]
    for name, kind, values in [("CODES", "u16", codes), ("WIDTHS", "u8", widths)]:
        output.append(f"pub(super) const {name}: [{kind}; 257] = [")
        for start in range(0, len(values), 12):
            output.append("    " + ", ".join(str(v) for v in values[start:start + 12]) + ",")
        output.append("];")
    output.append("pub(super) const TREE: [[u16; 2]; 256] = [")
    output.extend(f"    [{left}, {right}]," for left, right in edges)
    output.append("];")
    return "\n".join(output) + "\n"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--cc", default="gcc")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    rows = run_oracle(reference, args.cc)
    fixture = (
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Generated from upstream {reference.revision}\n"
    ).encode("utf-8") + rows
    table = generate_table(rows).encode("utf-8")
    formatted = subprocess.check_output(
        ["rustfmt", "--edition", "2024", "--emit", "stdout"], input=table
    ).replace(b"\r\n", b"\n")
    outputs = {
        ROOT / "crates/protocol/tests/fixtures/huffman_vectors.txt": fixture,
        ROOT / "crates/protocol/src/compression/codebook.rs": formatted,
    }
    for path, expected in outputs.items():
        if args.check:
            if path.read_bytes() != expected:
                raise SystemExit(f"Huffman artifact differs: {path.name}")
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(expected)
    print("Verified 257 Huffman symbols and 32 original compression samples")


if __name__ == "__main__":
    main()
