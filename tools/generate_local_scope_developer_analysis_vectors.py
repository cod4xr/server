# SPDX-License-Identifier: AGPL-3.0-or-later
"""Record twenty selected native developer local-name planning cases."""
import argparse
from pathlib import Path

from local_scope_developer_analysis_model import validate
from local_scope_developer_analysis_reference import execute
from reference_source import ReferenceSource

OUTPUT = Path(__file__).resolve().parents[1] / "crates/script/tests/fixtures/local_scope_developer_analysis_vectors.txt"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--assembler", default="nasm")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    output = execute(reference, args.cc, args.assembler)
    validate(output)
    expected = (
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
        "# Actual selected x86 CalcLocalVars helpers; driver O0/O2 agree.\n"
        "# Synthetic AST/IDs, bounded modeled allocator/globals; no parser/emitter/VM/freeing/unwinding.\n"
        "# D scenario later-direct-b; R parent/B allocated block/A auxiliary allocation bytes.\n"
        "# Block: abort created public total initialized-low initialized-high then n:name IDs.\n"
        "# 0 private b/1 b+c/2 common if/3 disjoint if/4 return/5 nested developer list;\n"
        "# 6 while break/7 while continue/8 for break/9 for continue, transfers inside developer list.\n"
        "# Native analysis does not reject nested developer lists; the selected emission diagnostic is separate.\n"
    ).encode("ascii") + output
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Native developer planning fixtures differ")
    else:
        OUTPUT.write_bytes(expected)
    print("Verified 20 native developer planning cases against selected independent joins at driver O0/O2")


if __name__ == "__main__":
    main()
