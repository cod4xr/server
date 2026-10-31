# SPDX-License-Identifier: AGPL-3.0-or-later
"""Ensure each script test leaf belongs to exactly one unconditional Cargo suite."""

from collections import Counter
from pathlib import Path
import re
import tomllib


def validate(root: Path) -> list[str]:
    """Reject missing/duplicate/gated test registration without invoking a build."""
    directory = root / "crates/script"
    manifest = tomllib.loads((directory / "Cargo.toml").read_text(encoding="utf-8"))
    expected = {("frontend", "tests/frontend.rs"), ("legacy", "tests/legacy.rs")}
    targets = {(target.get("name"), target.get("path")) for target in manifest.get("test", [])}
    failures = []
    if manifest["package"].get("autotests") is not False or targets != expected:
        failures.append("Script tests must use exactly the explicit frontend/legacy suites")
    declared = Counter()
    for name, path in sorted(expected):
        harness = directory / path
        if not harness.is_file():
            failures.append(f"Script test suite is absent: {name}")
            continue
        text = harness.read_text(encoding="utf-8")
        if re.search(r"#\[(?:cfg|cfg_attr|ignore|test)\b|\binclude!\s*\(", text):
            failures.append(f"Script test suite must declare unconditional modules only: {name}")
        for filename, module in re.findall(r'(?m)^#\[path = "([^"]+)"\]\s*\nmod ([a-z0-9_]+);$', text):
            relative = Path(filename)
            if (relative.is_absolute() or relative.drive or ".." in relative.parts
                    or relative.as_posix() != filename or relative.suffix != ".rs"
                    or relative.stem != module):
                failures.append(f"Script test module/path identity mismatch: {name}/{module}")
            declared[filename] += 1
    tests = directory / "tests"
    leaves = {path.relative_to(tests).as_posix() for path in tests.rglob("*.rs")
              if re.search(r"(?m)^#\[test\]\s*$", path.read_text(encoding="utf-8"))}
    missing = leaves - set(declared)
    extra = set(declared) - leaves
    repeated = {name for name, count in declared.items() if count != 1}
    if missing or extra or repeated:
        failures.append(
            f"Script test leaf registration differs: missing={sorted(missing)}, "
            f"extra={sorted(extra)}, repeated={sorted(repeated)}"
        )
    return failures
