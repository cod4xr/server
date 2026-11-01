# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard selected resolved/deferred developer-reference roles, not native execution."""

import argparse
from pathlib import Path
import re

from reference_source import ReferenceSource
import script_update_reference as assembly

ROLES = {
    "EmitFunction_80": (
        "cmp edx, 0x7\n\tjz EmitFunction_210\n",
        "mov edx, [gScrCompilePub+0x20024]\n\ttest edx, edx\n\tjz EmitFunction_220\n",
    ),
    "EmitFunction_220": ("_cstring_normal_script_ca", "call CompileError\n"),
    "LinkThread_40": (
        "mov ecx, [ebp-0x28]\n\tmov edx, [ecx+0x4]\n\tcmp edx, 0xc\n\tjnz LinkThread_70\n",
        "cmp eax, 0x7\n\tjnz LinkThread_80\n",
    ),
    "LinkThread_130": ("_cstring_normal_script_ca", "call CompileError2\n"),
}
TYPES = (("VAR_CODEPOS", 7), ("VAR_DEVELOPER_CODEPOS", 12))
DIAGNOSTIC = 'db "normal script cannot reference a function in a /# ... #/ comment",0'


def verify(compiler: str, header: str) -> None:
    """Tie selected direct/deferred reference rejection to ordinary/developer types."""
    for name, value in TYPES:
        if not re.search(rf"\b{name}\s*=\s*(?i:0x{value:x})\b", header):
            raise ValueError("Local call code-position type identities changed")
    for label, needles in ROLES.items():
        if any(needle not in assembly.block(compiler, label) for needle in needles):
            raise ValueError("Local developer-reference rejection roles changed")
    if DIAGNOSTIC not in compiler:
        raise ValueError("Normal-to-developer reference diagnostic identity changed")


def verify_mutations(compiler: str, header: str) -> int:
    """Reject scoped in-memory compiler/type/diagnostic mutations without file changes."""
    cases = []
    for label, needles in ROLES.items():
        body = assembly.block(compiler, label)
        for needle in needles:
            if body.count(needle) != 1:
                raise ValueError("Call-link fault injection needs one scoped role")
            changed = body.replace(needle, "mutated_role\n", 1)
            cases.append((compiler.replace(f"{label}:\n{body}", f"{label}:\n{changed}", 1), header))
    for name, value in TYPES:
        match = re.search(rf"\b{name}\s*=\s*(?i:0x{value:x})\b", header)
        if not match:
            raise ValueError("Missing recorded code-position identity")
        changed = header[:match.start()] + f"{name} = 0x{value+1:x}" + header[match.end():]
        cases.append((compiler, changed))
    cases.append((compiler.replace(DIAGNOSTIC, 'db "changed diagnostic",0', 1), header))
    for changed, types in cases:
        try:
            verify(changed, types)
        except ValueError:
            continue
        raise ValueError("Call-link source guard accepted an injected role change")
    return len(cases)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    compiler = reference.files["src/asmsource/_scr_compiler.asm"]
    header = reference.files["src/scr_vm.h"]
    verify(compiler, header)
    print(f"Verified selected local-call visibility; rejected {verify_mutations(compiler, header)} mutations")


if __name__ == "__main__":
    main()
