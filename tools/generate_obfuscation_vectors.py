#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Generate byte-exact fixtures from both original channel XOR functions."""

import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

from reference_source import ReferenceSource


ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--cc", default="gcc")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    preamble = (ROOT / "tools/oracles/wire_preamble.h").read_text()
    context = r"""
#define MAX_RELIABLE_COMMANDS 128
typedef struct { char command[1024]; } reliable_command_t;
typedef struct {
    int challenge, serverId, messageAcknowledge, reliableAcknowledge;
    netchan_t netchan;
    char lastClientCommandString[1024];
    reliable_command_t reliableCommands[MAX_RELIABLE_COMMANDS];
} client_t;
"""
    parts = [preamble, context]
    for name in ("SV_Netchan_Decode", "SV_Netchan_Encode"):
        parts.append(reference.function("src/sv_net_chan.c", name))
    parts.append(
        (ROOT / "tools/oracles/obfuscation_driver.c").read_text()
    )
    with tempfile.TemporaryDirectory(prefix="cod4x-xor-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / (
            "oracle.exe" if sys.platform == "win32" else "oracle"
        )
        subprocess.run(
            [args.cc, "-std=c11", "-O0", str(source), "-o", str(executable)],
            check=True
        )
        rows = subprocess.check_output([str(executable)])
    result = (
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Generated from upstream {reference.revision}\n"
    ).encode("utf-8") + rows.replace(b"\r\n", b"\n")
    output = ROOT / "crates/protocol/tests/fixtures/obfuscation_vectors.txt"
    if args.check:
        if output.read_bytes() != result:
            raise SystemExit("Channel XOR fixtures differ from original C")
    else:
        output.write_bytes(result)
    print(f"Verified {len(rows.splitlines())} original channel XOR samples")


if __name__ == "__main__":
    main()
