#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Record valid legacy block offsets using the original pointer conversion function."""

import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

from reference_source import ReferenceSource


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/content/tests/fixtures/zone_pointer_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--cc", default="gcc")
    args = parser.parse_args()
    if sys.byteorder != "little":
        raise SystemExit("Original pointer oracle requires a little-endian host")
    reference = ReferenceSource(args.reference)
    parts = [
        "#include <stdint.h>\n#include <stddef.h>\n#include <stdio.h>\n"
        "#ifndef __cdecl\n#define __cdecl\n#endif\n"
        "typedef uint32_t DWORD;\ntypedef unsigned char byte;\n"
        "struct Block { byte *data; uint32_t size; };\n"
        "static struct Block g_streamBlocks[9];",
        reference.function("src/db_load.cpp", "DB_ConvertOffsetToPointer"),
        (ROOT / "tools/oracles/zone_pointer_driver.c").read_text(encoding="utf-8"),
    ]
    with tempfile.TemporaryDirectory(prefix="cod4x-zone-pointer-oracle-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        subprocess.run([args.cc, "-std=c11", "-O0", "-fno-strict-aliasing",
                        "-Wno-pointer-to-int-cast", str(source), "-o", str(executable)], check=True)
        rows = subprocess.check_output([str(executable)]).replace(b"\r\n", b"\n")
    expected = (
        "# Original DB_ConvertOffsetToPointer; valid allocated spans only.\n"
        "# Host pointers are narrowed by source; only relative u32 offsets are recorded.\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + rows
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Zone pointer fixtures differ from original function")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(rows.splitlines())} original C relative block offsets")


if __name__ == "__main__":
    main()
