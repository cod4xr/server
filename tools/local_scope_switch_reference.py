# SPDX-License-Identifier: AGPL-3.0-or-later
"""Execute selected native switch/slot/branch/transfer emitters with modeled services."""
import argparse
from pathlib import Path
import sys

from local_scope_emission_reference import EMITTERS, SERVICES
from local_scope_reference import run
from reference_source import ReferenceSource

FUNCTIONS = EMITTERS + ("EmitSwitchStatement", "CompareCaseInfo")
EXTERNALS = SERVICES | {"TempMallocAlignStrict", "IsValidArrayIndex", "GetInternalVariableIndex",
                        "SL_TransferRefToUser", "qsort", "va"}
DRIVERS = ("local_scope_switch_mode.h", "local_scope_platform.c", "local_scope_services.c",
           "local_scope_ast.c", "local_scope_emission_services.c", "local_scope_switch_services.c",
           "local_scope_switch_cases.c")


def execute(reference: ReferenceSource, cc: str = "gcc", assembler: str = "nasm") -> bytes:
    return run(reference, cc, assembler, functions=FUNCTIONS, externals=EXTERNALS, drivers=DRIVERS)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    sys.stdout.buffer.write(execute(ReferenceSource(args.reference)))


if __name__ == "__main__":
    main()
