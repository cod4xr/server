# SPDX-License-Identifier: AGPL-3.0-or-later
"""Ensure each script test leaf belongs to exactly one unconditional Cargo suite."""

from collections import Counter
from pathlib import Path, PurePosixPath, PureWindowsPath
import re
import tomllib


TEST = re.compile(r"(?m)^#\[test\]\s*$")
MODULE = re.compile(r"mod ([a-z_][a-z0-9_]*);(?:\s*//.*)?")
PATH = re.compile(r'#\[path = "([^"\n]+)"\](?:\s*//.*)?')


def explicit_path(filename: str, module: str) -> bool:
    """Accept canonical relative POSIX leaf paths on every host OS."""
    relative = PurePosixPath(filename)
    return (not relative.is_absolute() and not PureWindowsPath(filename).drive
            and "\\" not in filename and ".." not in relative.parts
            and relative.as_posix() == filename and relative.suffix == ".rs"
            and relative.stem == module)


def registrations(harness: Path, tests: Path, failures: list[str]) -> list[str]:
    """Parse the bounded module-only suite grammar, including ordinary Rust mods."""
    declared = []
    modules = set()
    pending = None
    for number, raw in enumerate(harness.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith("//"):
            continue
        attribute = PATH.fullmatch(line)
        if attribute:
            if pending is not None:
                failures.append(f"Repeated path attribute: {harness.name}:{number}")
            pending = attribute[1]
            continue
        declaration = MODULE.fullmatch(line)
        if not declaration:
            failures.append(f"Suite requires unconditional module declarations: {harness.name}:{number}")
            continue
        module = declaration[1]
        if module in modules:
            failures.append(f"Repeated suite module identity: {harness.name}/{module}")
        modules.add(module)
        if pending is not None:
            filename, pending = pending, None
            if not explicit_path(filename, module):
                failures.append(f"Script test module/path identity mismatch: {harness.name}/{module}")
                continue
            if not (tests / filename).is_file():
                failures.append(f"Declared script test leaf is absent: {filename}")
            declared.append(filename)
            continue
        leaf, helper = tests / f"{module}.rs", tests / module / "mod.rs"
        if leaf.is_file() == helper.is_file():
            failures.append(f"Script module is absent or ambiguous: {harness.name}/{module}")
        elif leaf.is_file():
            declared.append(leaf.relative_to(tests).as_posix())
        elif TEST.search(helper.read_text(encoding="utf-8")):
            failures.append(f"Helper tests require a separately registered leaf: {module}")
    if pending is not None:
        failures.append(f"Dangling suite path attribute: {harness.name}")
    return declared


def validate(root: Path) -> list[str]:
    """Reject missing/duplicate/gated test registration without invoking a build."""
    directory = root / "crates/script"
    manifest = tomllib.loads((directory / "Cargo.toml").read_text(encoding="utf-8"))
    expected = {("frontend", "tests/frontend.rs"), ("legacy", "tests/legacy.rs")}
    entries = manifest.get("test", [])
    targets = {(target.get("name"), target.get("path")) for target in entries}
    failures = []
    if (manifest["package"].get("autotests") is not False or targets != expected
            or len(entries) != len(expected)):
        failures.append("Script tests must use exactly the explicit frontend/legacy suites")
    declared = Counter()
    tests = directory / "tests"
    for name, path in sorted(expected):
        harness = directory / path
        if not harness.is_file():
            failures.append(f"Script test suite is absent: {name}")
            continue
        declared.update(registrations(harness, tests, failures))
    leaves = {path.relative_to(tests).as_posix() for path in tests.rglob("*.rs")
              if TEST.search(path.read_text(encoding="utf-8"))}
    missing = leaves - set(declared)
    extra = set(declared) - leaves
    repeated = {name for name, count in declared.items() if count != 1}
    if missing or extra or repeated:
        failures.append(
            f"Script test leaf registration differs: missing={sorted(missing)}, "
            f"extra={sorted(extra)}, repeated={sorted(repeated)}"
        )
    return failures
