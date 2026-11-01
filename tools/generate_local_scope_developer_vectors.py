# SPDX-License-Identifier: AGPL-3.0-or-later
"""Record selected developer-list/slot and discarded function-name executions."""
import argparse
from pathlib import Path

from local_scope_developer_model import validate
from local_scope_developer_reference import execute
from reference_source import ReferenceSource

OUTPUT = Path(__file__).resolve().parents[1] / "crates/script/tests/fixtures/local_scope_developer_vectors.txt"


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
        "# Actual selected x86 EmitDeveloperStatementList/Scr_FindLocalVarIndex; driver O0/O2 agree.\n"
        "# Modeled planned scopes/AST/body assignment/abort/strings/writer/temp rewind/diagnostic return.\n"
        "# No native parser/local analysis/body dispatcher/return emitter/VM/freeing/error unwinding.\n"
        "# C scenario enabled initial-mode; 0 empty/1 new private/2 existing/3 planned later/4 modeled abort.\n"
        "# P parent/B child: abort created public total initialized-low initialized-high, then names.\n"
        "# G final-mode source-position diagnostics rewinds add-refs drop-refs canonical-transfers.\n"
        "# M body compilation modes; O opcode requests, including discarded; W retained modeled bytes.\n"
        "# F external borrowed-name flag; mode-two EmitFunction, no name/link services.\n"
        "# F rows: G mode unresolved-file-count diagnostics add-refs canonical-transfers; T modeled removed IDs; W bytes.\n"
    ).encode("ascii") + output
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Native developer-list fixtures differ")
    else:
        OUTPUT.write_bytes(expected)
    print("Verified 12 native developer-list and 4 discarded-function cases at driver O0/O2")


if __name__ == "__main__":
    main()
