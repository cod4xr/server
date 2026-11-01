# SPDX-License-Identifier: AGPL-3.0-or-later
"""Execute pinned developer-list/slot emitters with bounded modeled body services."""
import argparse
from pathlib import Path
import sys

from local_scope_reference import run
from reference_source import ReferenceSource

FUNCTIONS = ("EmitDeveloperStatementList", "Scr_FindLocalVarIndex", "EmitFunction")
EXTERNALS = {
    "Com_Error", "CompileError", "gScrVarPub", "gScrCompilePub", "scrCompileGlob",
    "EmitStatement", "EmitOpcode", "TempMalloc", "TempMallocAlign", "TempMemorySetPos",
    "SL_AddRefToString", "SL_RemoveRefToString", "SL_TransferToCanonicalString", "SL_ConvertToString",
    "AddOpcodePos", "FindVariable", "GetNewVariable", "GetValueType", "GetVariable",
    "SGetObjectA", "SL_TransferRefToUser", "Scr_CreateCanonicalFilename", "Scr_EvalVariableExtern",
    "SetNewVariableValue", "SetVariableValue",
}
DRIVERS = ("local_scope_platform.c", "local_scope_developer_services.c", "local_scope_developer_cases.c")


def execute(reference: ReferenceSource, cc: str = "gcc", assembler: str = "nasm") -> bytes:
    return run(reference, cc, assembler, functions=FUNCTIONS, externals=EXTERNALS, drivers=DRIVERS)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    sys.stdout.buffer.write(execute(ReferenceSource(args.reference)))


if __name__ == "__main__":
    main()
