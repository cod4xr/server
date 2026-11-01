# SPDX-License-Identifier: AGPL-3.0-or-later
"""Execute copied pinned C game-time functions with explicit modeled services."""
from pathlib import Path
import re
import subprocess
import sys
import tempfile

from reference_source import ReferenceSource

ROOT = Path(__file__).resolve().parents[1]


def execute(reference: ReferenceSource, cc: str = "gcc") -> bytes:
    matches = re.findall(r"(?m)^#define VAR_NAME_LOW_MASK 0x00FFFFFF$", reference.files["src/scr_vm.h"])
    if len(matches) != 1:
        raise ValueError("Pinned time mask declaration changed")
    parts = [matches[0], (ROOT / "tools/oracles/scheduler_phase_services.c").read_text(encoding="utf-8")]
    parts.extend(reference.function("src/cscr_vm.cpp", name)
                 for name in ("VM_SetTime", "Scr_RunCurrentThreads", "Scr_IncTime"))
    parts.append((ROOT / "tools/oracles/scheduler_phase_cases.c").read_text(encoding="utf-8"))
    with tempfile.TemporaryDirectory(prefix="cod4x-scheduler-phase-") as scratch:
        directory = Path(scratch)
        source = directory / "oracle.c"
        source.write_text("\n\n".join(parts), encoding="utf-8")
        executable = directory / ("oracle.exe" if sys.platform == "win32" else "oracle")
        outputs = []
        for optimization in ("-O0", "-O2"):
            compiled = subprocess.run([cc, "-std=gnu11", optimization, "-Wall", "-Wextra", "-Werror",
                                       str(source), "-o", str(executable)], capture_output=True, timeout=60)
            if compiled.returncode:
                raise ValueError(compiled.stderr.decode("utf-8", errors="replace")[:4000])
            flags = subprocess.CREATE_NO_WINDOW if sys.platform == "win32" else 0
            result = subprocess.run([str(executable)], check=True, capture_output=True,
                                    timeout=30, creationflags=flags)
            if len(result.stdout) > 65536:
                raise ValueError("Native scheduler phase trace exceeded bound")
            outputs.append(result.stdout.replace(b"\r\n", b"\n"))
        if outputs[0] != outputs[1]:
            raise ValueError("Native scheduler phase traces differ between O0/O2")
        return outputs[0]
