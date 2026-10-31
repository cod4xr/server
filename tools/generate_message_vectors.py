#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Generate wire fixtures by compiling selected, unmodified upstream functions.

The C program is a disposable test oracle, never linked into the Rust product.
Only the pinned, hashed msg.c is accepted. No upstream build scripts run.
"""

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[1]
FUNCTIONS = (
    "MSG_WriteByte",
    "MSG_WriteShort",
    "MSG_WriteLong",
    "MSG_WriteInt64",
    "MSG_WriteFloat",
    "MSG_WriteBits",
)
PREAMBLE = r"""
#include <stdint.h>
#include <stdio.h>
#include <string.h>
enum { qtrue = 1 };
typedef struct {
    unsigned char *data;
    int maxsize, cursize, bit, overflowed;
} msg_t;
"""
DRIVER = r"""
static uint32_t next_value(uint32_t *state) {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    return *state;
}

int main(void) {
    for (uint32_t seed = 1; seed <= 64; ++seed) {
        unsigned char storage[4096];
        memset(storage, 0xa5, sizeof(storage));
        msg_t msg = { storage, sizeof(storage), 0, 0, 0 };
        uint32_t state = seed;
        for (int operation = 0; operation < 96; ++operation) {
            uint32_t kind = next_value(&state) % 6;
            uint32_t raw = next_value(&state);
            switch (kind) {
                case 0: {
                    int bits;
                    memcpy(&bits, &raw, sizeof(bits));
                    MSG_WriteBits(&msg, bits, next_value(&state) % 33);
                    break;
                }
                case 1: MSG_WriteByte(&msg, raw & 0xff); break;
                case 2: {
                    uint16_t low = raw & 0xffff;
                    int16_t value;
                    memcpy(&value, &low, sizeof(value));
                    MSG_WriteShort(&msg, value);
                    break;
                }
                case 3: {
                    int32_t value;
                    memcpy(&value, &raw, sizeof(value));
                    MSG_WriteLong(&msg, value);
                    break;
                }
                case 4: {
                    uint64_t wide = ((uint64_t)next_value(&state) << 32) | raw;
                    int64_t value;
                    memcpy(&value, &wide, sizeof(value));
                    MSG_WriteInt64(&msg, value);
                    break;
                }
                case 5: {
                    uint32_t finite_bits = raw & 0xff7fffff;
                    float value;
                    memcpy(&value, &finite_bits, sizeof(value));
                    MSG_WriteFloat(&msg, value);
                    break;
                }
                default: return 2;
            }
        }
        if (msg.overflowed) return 3;
        printf("%u ", seed);
        for (int i = 0; i < msg.cursize; ++i) printf("%02x", storage[i]);
        putchar('\n');
    }
    return 0;
}
"""


def extract_function(source: str, name: str) -> str:
    """Extract the exact balanced function body, refusing ambiguous matches."""
    pattern = rf"(?m)^void {re.escape(name)}\s*\([^)]*\)\s*\{{"
    matches = list(re.finditer(pattern, source))
    if len(matches) != 1:
        raise ValueError(f"Expected exactly one definition of {name}")
    match = matches[0]
    depth = 1
    end = match.end()
    # The pinned primitive bodies contain no braces inside strings or comments.
    while depth and end < len(source):
        depth += (source[end] == "{") - (source[end] == "}")
        end += 1
    if depth:
        raise ValueError(f"Unterminated function: {name}")
    return source[match.start():end]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "reference", type=Path, help="Pinned CoD4x_Server checkout"
    )
    parser.add_argument("--cc", default="gcc", help="Native C compiler")
    parser.add_argument(
        "--check", action="store_true", help="Compare without writing"
    )
    args = parser.parse_args()
    if sys.byteorder != "little":
        raise SystemExit("The upstream oracle requires a little-endian host")
    lock = json.loads((ROOT / "upstream.lock.json").read_text(encoding="utf-8"))
    revision = subprocess.check_output(
        ["git", "-C", str(args.reference), "rev-parse", "HEAD"], text=True
    ).strip()
    if revision != lock["revision"]:
        raise SystemExit("Reference checkout does not match upstream.lock.json")
    source_bytes = (args.reference / "src/msg.c").read_bytes()
    # Git on Windows may check out CRLF; the lock hashes the canonical Git blob.
    source_bytes = source_bytes.replace(b"\r\n", b"\n")
    digest = hashlib.sha256(source_bytes).hexdigest()
    if digest != lock["files"]["src/msg.c"]:
        raise SystemExit(
            "Reference msg.c hash does not match upstream.lock.json"
        )
    source = source_bytes.decode("utf-8")
    functions = "\n\n".join(
        extract_function(source, name) for name in FUNCTIONS
    )
    with tempfile.TemporaryDirectory(prefix="cod4x-message-oracle-") as scratch:
        scratch_path = Path(scratch)
        program = scratch_path / "oracle.c"
        executable_name = "oracle.exe" if sys.platform == "win32" else "oracle"
        executable = scratch_path / executable_name
        program.write_text(PREAMBLE + functions + DRIVER, encoding="utf-8")
        subprocess.run(
            [args.cc, "-std=c11", "-O0", "-fno-strict-aliasing", str(program),
             "-o", str(executable)], check=True
        )
        rows = subprocess.check_output([str(executable)])
        rows = rows.replace(b"\r\n", b"\n")
    header = (
        "# Generated by tools/generate_message_vectors.py; do not edit.\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {lock['revision']}; msg.c SHA256 {digest}\n"
        "# 64 seeds; 96 mixed byte/bit/integer/float operations per seed.\n"
    ).encode("utf-8")
    expected = header + rows
    output = ROOT / "crates/protocol/tests/fixtures/message_vectors.txt"
    if args.check:
        if output.read_bytes() != expected:
            raise SystemExit(
                "Differential fixtures differ from the upstream C oracle"
            )
    else:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_bytes(expected)
    print(f"Verified 64 upstream C vectors ({len(rows)} bytes of fixture text)")


if __name__ == "__main__":
    main()
