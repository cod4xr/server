# SPDX-License-Identifier: AGPL-3.0-or-later
"""Execute pinned VM_Resume with bounded modeled queue/archive/execution services."""
from local_scope_reference import run
from reference_source import ReferenceSource

FUNCTIONS = ("VM_Resume",)
EXTERNALS = {
    "gScrVmGlob", "gScrVmPub", "gScrVarPub", "gFs", "thread_count",
    "AddRefToObject", "ClearVariableValue", "FindFirstSibling", "FindLastSibling",
    "FindPrevSibling", "GetParentLocalId", "GetVariableKeyObject", "GetVariableValueAddress",
    "MT_Free", "RemoveObjectVariable", "RemoveRefToObject", "RemoveRefToValue",
    "Scr_ClearWaitTime", "Scr_DecNumScriptThreads", "Scr_GetVarId", "Sys_Milliseconds",
    "VM_ExecuteInternal",
}
DRIVERS = ("local_scope_platform.c", "vm_resume_services.c", "vm_resume_cases.c")
GLOBALS = frozenset({"gScrVmGlob", "gScrVmPub", "gFs", "thread_count"})


def execute(reference: ReferenceSource, cc: str = "gcc", assembler: str = "nasm") -> bytes:
    return run(reference, cc, assembler, functions=FUNCTIONS, externals=EXTERNALS,
               drivers=DRIVERS, source_name="src/asmsource/_scr_vm.asm", modeled_globals=GLOBALS)
