# SPDX-License-Identifier: AGPL-3.0-or-later
"""Execute pinned local-name planning for synthetic developer lists and transfers."""
import argparse
from pathlib import Path
import sys

from local_scope_reference import run
from reference_source import ReferenceSource

DRIVERS = ("local_scope_platform.c", "local_scope_services.c", "local_scope_ast.c",
           "local_scope_loops.c", "local_scope_developer_analysis_cases.c")


def execute(reference: ReferenceSource, cc: str = "gcc", assembler: str = "nasm") -> bytes:
    return run(reference, cc, assembler, drivers=DRIVERS)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    sys.stdout.buffer.write(execute(ReferenceSource(args.reference)))


if __name__ == "__main__":
    main()
