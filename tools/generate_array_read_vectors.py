#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Record actual array reads with bounded modeled storage/ref/error services."""
import argparse
from pathlib import Path
import subprocess
import sys
import tempfile

import array_reference
from reference_source import ReferenceSource
import script_array_read_reference as reads


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/array_read_vectors.txt"


def run(reference: ReferenceSource, compiler: str) -> bytes:
    functions = reads.inputs(reference)
    reads.verify(functions)
    parts = array_reference.core(reference)
    parts.append((ROOT / "tools/oracles/array_read_services.c").read_text(encoding="utf-8"))
    parts.extend(functions[name] for name in ("FindArrayVariableIndex", "FindArrayVariable", "FindVariable",
                 "Scr_FindArrayIndex", "Scr_EvalVariable", "Scr_EvalArray", "Scr_EvalSizeValue"))
    parts.append((ROOT / "tools/oracles/array_read_cases.c").read_text(encoding="utf-8"))
    errors = [("0", str(tag), "33") for tag in (0, 1, 3, 4, 5, 7, 8, 9, 10, 11, 12, 13)]
    errors += [("0", "6", "0xffffffff"), ("0", "6", "0x800000"),
               ("0", "5", "0x7fc00000"), ("1", "6", "0")]
    outputs = []
    with tempfile.TemporaryDirectory(prefix="cod4x-array-read-oracle-") as scratch:
        directory = Path(scratch).resolve()
        if (not directory.is_relative_to(Path(tempfile.gettempdir()).resolve())
                or not directory.name.startswith("cod4x-array-read-oracle-")):
            raise ValueError("Oracle scratch must stay inside its owned temporary directory")
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        for optimization in ("-O0", "-O2"):
            subprocess.run([compiler, "-std=c11", optimization, "-Wall", "-Wextra", "-Werror",
                            "-msse2", str(source), "-o", str(executable)], check=True)
            rows = []
            for arguments in [(), *errors]:
                completed = subprocess.run([str(executable), *arguments], capture_output=True, timeout=60)
                if completed.returncode:
                    raise ValueError(f"Array read driver failed ({optimization}, {arguments}): "
                                     f"{completed.stderr.decode('ascii', errors='replace')}")
                rows.append(completed.stdout.replace(b"\r\n", b"\n"))
            outputs.append(b"".join(rows))
    if outputs[0] != outputs[1]:
        raise ValueError("Array reads differ between O0/O2")
    return outputs[0]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    rows = run(reference, args.cc)
    expected = (
        "# Actual selected C array lookup/read/size functions and declarations; O0/O2.\n"
        "# Bounded modeled pool/lookup/nonfreeing refs/error exit, not native allocator/interner/VM/unwinding.\n"
        "# Profile 0 has current primitive/shared-struct entries; 1=function word, 2=nested array are helper-only.\n"
        "# Vector payload reads are excluded on host pointer ABI; vector index-type errors remain covered.\n"
        "# Error rows record callback-entry state before external unwinding, then exit the process.\n"
        "# R profile rotation aliases key_type key_word found result_type result_word pointer_subtype string_adds vector_adds string_key_removals size /keys\n"
        "# Z rotation aliases integer_size parent_removals\n"
        "# E receiver_subtype key_type key_word category native_error_index parent_removals\n"
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
    ).encode("utf-8") + rows
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Array read traces differ")
    else:
        OUTPUT.write_bytes(expected)
    print(f"Verified {len(rows.splitlines())} selected native array read traces at O0/O2; helper evidence only")


if __name__ == "__main__":
    main()
