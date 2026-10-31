# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard the archival integer instruction profile; not full native VM execution."""

import re


def verify_machine_profile(source: str) -> None:
    """Check exact arithmetic blocks before testing their instruction semantics."""
    profiles = {
        "_Z12Scr_EvalPlusP13VariableValueS0__60":
            ("mov eax, [esi]", "add eax, [edi]", "mov [esi], eax"),
        "_Z13Scr_EvalMinusP13VariableValueS0__20":
            ("mov eax, [ebx]", "sub eax, [esi]", "mov [ebx], eax"),
        "_Z16Scr_EvalMultiplyP13VariableValueS0__20":
            ("mov eax, [ebx]", "imul eax, [esi]", "mov [ebx], eax"),
        "_Z17Scr_EvalShiftLeftP13VariableValueS0__10":
            ("mov ecx, [eax]", "shl dword [edx], cl"),
        "_Z18Scr_EvalShiftRightP13VariableValueS0__10":
            ("mov ecx, [eax]", "sar dword [edx], cl"),
        "_Z11Scr_EvalModP13VariableValueS0__10":
            ("mov eax, [ecx]", "mov ebx, edx", "cdq", "idiv ebx", "mov [ecx], edx"),
    }
    for label, instructions in profiles.items():
        block = re.search(rf"(?ms)^{re.escape(label)}:\n(.*?)(?=^\w+:|\Z)", source)
        if not block:
            raise ValueError("Missing integer instruction block")
        text = block[1]
        sequence = "\n\t".join(instructions)
        if sequence not in text:
            raise ValueError(f"Integer instruction profile changed: {label}")
