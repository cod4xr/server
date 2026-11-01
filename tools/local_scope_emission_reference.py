# SPDX-License-Identifier: AGPL-3.0-or-later
"""Execute selected native scope emitters with modeled body/output services."""
import argparse
from pathlib import Path
import sys

from local_scope_reference import EXTERNALS, FUNCTIONS, run
from reference_source import ReferenceSource

EMITTERS = FUNCTIONS + (
    "Scr_FindLocalVarIndex", "EmitIfStatement", "EmitIfElseStatement",
    "EmitBreakStatement", "EmitContinueStatement",
)
SERVICES = EXTERNALS | {
    "EmitOpcode", "TempMalloc", "TempMallocAlign", "AddOpcodePos",
    "SL_AddRefToString", "SL_RemoveRefToString", "SL_TransferToCanonicalString",
    "SL_ConvertToString", "EmitOrEvalExpression", "EmitStatement", "EmitValue",
}
DRIVERS = ("local_scope_platform.c", "local_scope_services.c", "local_scope_ast.c",
           "local_scope_emission_services.c", "local_scope_emission_cases.c")


def execute(reference: ReferenceSource, cc: str = "gcc", assembler: str = "nasm") -> bytes:
    return run(reference, cc, assembler, functions=EMITTERS, externals=SERVICES, drivers=DRIVERS)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    sys.stdout.buffer.write(execute(ReferenceSource(args.reference)))


if __name__ == "__main__":
    main()
